//! Structural feed validation; cryptographic verification belongs to Sparkle.
use super::{DOWNLOAD, files, invalid, read, stable_version};
use crate::{Error, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use roxmltree::{Document, Node};
use std::{collections::BTreeMap, path::Path};
const SPARKLE: &str = "http://www.andymatuschak.org/xml-namespaces/sparkle";

#[derive(Debug, PartialEq, Eq)]
struct Content {
    name: (String, String),
    attributes: BTreeMap<(String, String), String>,
    text: String,
    children: Vec<Content>,
}
fn content(node: Node<'_, '_>) -> Content {
    Content {
        name: (
            node.tag_name().namespace().unwrap_or_default().into(),
            node.tag_name().name().into(),
        ),
        attributes: node
            .attributes()
            .map(|a| {
                (
                    (a.namespace().unwrap_or_default().into(), a.name().into()),
                    a.value().into(),
                )
            })
            .collect(),
        text: node.text().unwrap_or_default().trim().into(),
        children: node
            .children()
            .filter(Node::is_element)
            .map(content)
            .collect(),
    }
}
/// Validated update metadata. The original signed XML is never rewritten.
#[derive(Debug, PartialEq, Eq)]
pub struct Update {
    /// Stable bundle version.
    pub version: String,
    /// Human-visible version, which must equal the bundle version.
    pub short_version: String,
    /// Minimum macOS version.
    pub minimum_system: String,
    /// Full DMG size.
    pub dmg_length: u64,
    /// Signed release notes size.
    pub notes_length: u64,
    content: Content,
}
fn child<'a, 'i>(node: Node<'a, 'i>, namespace: Option<&str>, name: &str) -> Option<Node<'a, 'i>> {
    node.children().find(|n| {
        n.is_element() && n.tag_name().namespace() == namespace && n.tag_name().name() == name
    })
}
fn unqualified(node: Node<'_, '_>, name: &str) -> bool {
    // roxmltree's has_tag_name("name") matches every namespace. RSS tags
    // were unqualified in ElementTree, and must remain so after the port.
    node.is_element() && node.tag_name().namespace().is_none() && node.tag_name().name() == name
}
fn field(node: Node<'_, '_>, name: &str) -> String {
    child(node, Some(SPARKLE), name)
        .and_then(|n| n.text())
        .unwrap_or_default()
        .into()
}
fn signature_and_length(node: Node<'_, '_>) -> Result<u64> {
    let encoded = node.attribute((SPARKLE, "edSignature")).unwrap_or_default();
    let signature = STANDARD
        .decode(encoded)
        .map_err(|_| invalid("Invalid Ed25519 signature encoding"))?;
    if signature.len() != 64 || STANDARD.encode(&signature) != encoded {
        return Err(invalid("Expected a base64 64-byte Ed25519 signature"));
    }
    let length = if node.has_tag_name((SPARKLE, "releaseNotesLink")) {
        node.attribute((SPARKLE, "length"))
    } else {
        node.attribute("length")
    }
    .unwrap_or_default();
    if length.starts_with('0') || length.is_empty() || !length.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("Expected a positive asset length"));
    }
    length
        .parse()
        .map_err(|_| invalid("Expected a positive asset length"))
}

/// Validate every item, its canonical signatures, asset URLs, lengths and order.
/// If sizes are provided, they must cover every DMG and notes URL in the feed.
/// This does not replace Sparkle's verification of cryptographic signatures.
pub fn validate(
    path: &Path,
    allow_empty: bool,
    sizes: Option<&BTreeMap<String, u64>>,
) -> Result<Vec<Update>> {
    let data = read(path)?;
    let text = std::str::from_utf8(&data).map_err(|_| invalid("Appcast is not UTF-8"))?;
    if text.contains("<!DOCTYPE") || text.contains("<!ENTITY") {
        return Err(invalid("Appcasts must not declare a DTD or entities"));
    }
    let doc = Document::parse(text).map_err(|e| Error::Invalid(e.to_string()))?;
    let root = doc.root_element();
    let channels: Vec<_> = root
        .children()
        .filter(|n| unqualified(*n, "channel"))
        .collect();
    if !unqualified(root, "rss") || root.attribute("version") != Some("2.0") || channels.len() != 1
    {
        return Err(invalid("Expected one RSS 2.0 channel"));
    }
    let mut updates = Vec::new();
    let mut last = None;
    for item in channels[0].children().filter(|n| unqualified(*n, "item")) {
        let version = field(item, "version");
        let parsed = stable_version(&version)?;
        let short_version = field(item, "shortVersionString");
        if short_version != version {
            return Err(invalid("shortVersionString must match version"));
        }
        let minimum_system = field(item, "minimumSystemVersion");
        let parts: Vec<_> = minimum_system.split('.').collect();
        if !(2..=3).contains(&parts.len())
            || !parts
                .iter()
                .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(invalid("Missing or invalid minimumSystemVersion"));
        }
        if item
            .descendants()
            .filter(|n| unqualified(*n, "enclosure"))
            .count()
            != 1
        {
            return Err(invalid("Expected exactly one full-update enclosure"));
        }
        let prefix = format!("{DOWNLOAD}v{version}/");
        let dmg_url = format!("{prefix}Voxelite.dmg");
        let notes_url = format!("{prefix}Voxelite.html");
        let enclosure = child(item, None, "enclosure")
            .filter(|n| n.attribute("url") == Some(dmg_url.as_str()))
            .ok_or_else(|| invalid("Enclosure must reference this version's GitHub release DMG"))?;
        let notes = child(item, Some(SPARKLE), "releaseNotesLink")
            .filter(|n| n.text() == Some(notes_url.as_str()))
            .ok_or_else(|| invalid("Missing or incorrect GitHub release-notes link"))?;
        let dmg_length = signature_and_length(enclosure)?;
        let notes_length = signature_and_length(notes)?;
        for (url, length) in [(dmg_url, dmg_length), (notes_url, notes_length)] {
            if sizes.is_some_and(|sizes| sizes.get(&url) != Some(&length)) {
                return Err(invalid(
                    "Appcast asset length does not match the release asset",
                ));
            }
        }
        if last.as_ref().is_some_and(|last| last <= &parsed) {
            return Err(invalid(
                "Appcast versions must be unique and ordered newest first",
            ));
        }
        last = Some(parsed);
        updates.push(Update {
            version,
            short_version,
            minimum_system,
            dmg_length,
            notes_length,
            content: content(item),
        });
    }
    if updates.is_empty() && !allow_empty {
        return Err(invalid("Expected a signed update item"));
    }
    Ok(updates)
}

/// Check the newest update against the bundle and local assets, and require all
/// historical item content to survive unchanged (namespace prefix/spacing aside).
pub fn validate_release(path: &Path, dist: &Path, previous: &Path) -> Result<()> {
    let items = validate(path, false, None)?;
    let old_items = validate(previous, true, None)?;
    let info = files::read_plist(&dist.join("Voxelite.app/Contents/Info.plist"))?;
    let newest = &items[0];
    for (actual, key, message) in [
        (
            &newest.version,
            "CFBundleVersion",
            "Latest appcast version does not match the bundle",
        ),
        (
            &newest.short_version,
            "CFBundleShortVersionString",
            "Appcast short version does not match the bundle",
        ),
        (
            &newest.minimum_system,
            "LSMinimumSystemVersion",
            "Appcast minimum system version does not match the bundle",
        ),
    ] {
        if actual != files::plist_string(&info, key)? {
            return Err(invalid(message));
        }
    }
    if newest.dmg_length != files::file_size(&dist.join("Voxelite.dmg"))? {
        return Err(invalid("Appcast DMG length does not match"));
    }
    if newest.notes_length != files::file_size(&dist.join("Voxelite.html"))? {
        return Err(invalid("Appcast release-notes length does not match"));
    }
    if items[1..] != old_items {
        return Err(invalid("Appcast history was changed or discarded"));
    }
    Ok(())
}
