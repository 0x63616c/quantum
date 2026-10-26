//! Versioned canonical `.voxl` JSON with bounded streaming voxel storage.
//!
//! Runtime revisions never enter this format. Optional build recipes replay real
//! core commands to reconstruct a timeline. Unknown optional metadata
//! is inert, bounded, preserved, and never interpreted as a URL or executable.
mod build_history;
mod stream;
pub use build_history::{BuildHistory, BuildStep};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    sync::Arc,
};
use voxelite_core::*;

/// Maximum encoded portable project size.
pub const MAX_FILE_BYTES: usize = 128 * 1024 * 1024;
/// Maximum metadata retained during parsing, independent of voxel storage.
pub const MAX_METADATA_BYTES: usize = 8 * 1024 * 1024;
/// `.voxl` media type.
pub const MIME_TYPE: &str = "application/vnd.voxelite.project+json";
/// A syntax, compatibility, resource, or document-invariant error.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Invalid JSON or wire schema.
    #[error("invalid .voxl JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Document integrity failed.
    #[error(transparent)]
    Core(#[from] voxelite_core::Error),
    /// Input or output failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Incompatible format semantics or exceeded resource bounds.
    #[error("unsupported or invalid .voxl: {0}")]
    Invalid(String),
}
/// Result of a portable format operation.
pub type FormatResult<T> = std::result::Result<T, FormatError>;
/// Version declaration. New minor versions retain their declaration on save.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    /// Major format family; only 1 is supported.
    pub major: u16,
    /// Backward-compatible minor; required features still must be understood.
    pub minor: u16,
}
impl Default for Version {
    fn default() -> Self {
        Self { major: 1, minor: 0 }
    }
}
/// Validated project plus preserved optional fields and portable view.
#[derive(Clone, Debug)]
pub struct Project {
    /// Core content, with no session revision or raw history.
    pub document: Document,
    /// Original supported format declaration.
    pub version: Version,
    /// Optional portable default view. Excluded from semantic content hashes.
    pub view: Option<Value>,
    template: Value,
    build_history: Option<BuildHistory>,
    built_document: Option<Document>,
}
#[derive(Serialize, Deserialize)]
struct Header {
    id: Uuid,
    name: String,
    bounds: Region,
}
#[derive(Serialize, Deserialize)]
struct Wire {
    format: String,
    version: Version,
    required_features: Vec<String>,
    document: Header,
    palette: Palette,
    layers: Vec<Layer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    comments: Vec<Comment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    versions: Vec<NamedVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lineage: Option<Lineage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    view: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build_history: Option<BuildHistory>,
    #[serde(flatten)]
    extra: Extra,
}
impl Project {
    /// Wrap already validated core content in the current format version.
    pub fn new(document: Document) -> FormatResult<Self> {
        document.validate()?;
        Ok(Self {
            document,
            version: Version::default(),
            view: None,
            template: json!({}),
            build_history: None,
            built_document: None,
        })
    }
    /// Access retained input metadata (voxel arrays are deliberately empty here).
    pub fn metadata(&self) -> &Value {
        &self.template
    }
    /// Canonical bytes, bounded to the portable file cap.
    pub fn to_bytes(&self) -> FormatResult<Vec<u8>> {
        let mut bytes = Vec::new();
        self.write(&mut bytes)?;
        Ok(bytes)
    }
    /// Stream canonical output. Only one layer's sorted voxel list is staged.
    pub fn write(&self, writer: impl Write) -> FormatResult<()> {
        self.write_inner(writer, false)
    }
    fn write_inner(&self, writer: impl Write, semantic: bool) -> FormatResult<()> {
        self.document.validate()?;
        if self.version.major != 1 {
            return Err(FormatError::Invalid("unknown major version".into()));
        }
        let (mut metadata, volumes) = self.output_tree()?;
        if semantic {
            metadata
                .as_object_mut()
                .expect("root object")
                .remove("view");
            metadata
                .as_object_mut()
                .expect("root object")
                .remove("build_history");
        }
        validate_metadata(&metadata)?;
        let mut writer = stream::BoundedWriter {
            inner: writer,
            written: 0,
            limit: MAX_FILE_BYTES,
        };
        stream::write(&mut writer, &metadata, "", 0, &volumes)?;
        writer.write_all(b"\n")?;
        Ok(())
    }
    /// SHA-256 of canonical content, excluding `view` and optional `build_history`.
    pub fn semantic_hash(&self) -> FormatResult<String> {
        struct HashWriter(Sha256);
        impl Write for HashWriter {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                self.0.update(b);
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut h = HashWriter(Sha256::new());
        self.write_inner(&mut h, true)?;
        Ok(format!("{:x}", h.0.finalize()))
    }
    fn output_tree(&self) -> FormatResult<(Value, BTreeMap<String, &Volume>)> {
        let s = &self.document.content;
        let mut volumes = BTreeMap::new();
        let mut layers = s.layers.clone();
        for (i, l) in layers.iter_mut().enumerate() {
            volumes.insert(format!("/layers/{i}/voxels"), &s.layers[i].voxels);
            l.voxels = Volume::default();
        }
        let mut versions: Vec<_> = self.document.versions.values().cloned().collect();
        for (i, v) in versions.iter_mut().enumerate() {
            let original = &self.document.versions[&v.id];
            let snapshot = Arc::make_mut(&mut v.snapshot);
            snapshot.palette.slots.sort_by_key(|s| s.index);
            snapshot.comments.sort_by_key(|c| c.id);
            for (j, l) in snapshot.layers.iter_mut().enumerate() {
                volumes.insert(
                    format!("/versions/{i}/snapshot/layers/{j}/voxels"),
                    &original.snapshot.layers[j].voxels,
                );
                l.voxels = Volume::default();
            }
        }
        let mut palette = s.palette.clone();
        palette.slots.sort_by_key(|s| s.index);
        let mut comments = s.comments.clone();
        comments.sort_by_key(|c| c.id);
        let wire = Wire {
            format: "voxelite".into(),
            version: self.version,
            required_features: vec![],
            document: Header {
                id: self.document.id,
                name: s.name.clone(),
                bounds: s.bounds,
            },
            palette,
            layers,
            comments,
            versions,
            lineage: self.document.lineage.clone(),
            view: self.view.clone(),
            extra: s.extra.clone(),
            build_history: self.build_history().cloned(),
        };
        let mut metadata = serde_json::to_value(wire)?;
        preserve_unknown(&mut metadata, &self.template, "");
        Ok((metadata, volumes))
    }
}
/// Read bounded UTF-8 JSON directly into staged chunks and a small metadata tree.
/// Duplicate object keys are rejected at every level, before installation.
pub fn read(reader: impl Read) -> FormatResult<Project> {
    let (metadata, mut volumes) = stream::read(reader)?;
    validate_metadata(&metadata)?;
    let mut wire: Wire = serde_json::from_value(metadata.clone())?;
    if wire.format != "voxelite" || wire.version.major != 1 {
        return Err(FormatError::Invalid(
            "expected voxelite major version 1".into(),
        ));
    }
    if !wire.required_features.is_empty() {
        return Err(FormatError::Invalid(format!(
            "unsupported required features {:?}",
            wire.required_features
        )));
    }
    for (i, l) in wire.layers.iter_mut().enumerate() {
        l.voxels = volumes
            .remove(&format!("/layers/{i}/voxels"))
            .ok_or_else(|| FormatError::Invalid("missing voxels array".into()))?;
    }
    let mut versions = BTreeMap::new();
    for (i, mut v) in wire.versions.into_iter().enumerate() {
        let snapshot = Arc::make_mut(&mut v.snapshot);
        for (j, l) in snapshot.layers.iter_mut().enumerate() {
            l.voxels = volumes
                .remove(&format!("/versions/{i}/snapshot/layers/{j}/voxels"))
                .ok_or_else(|| FormatError::Invalid("missing snapshot voxels".into()))?;
        }
        if versions.insert(v.id, v).is_some() {
            return Err(FormatError::Invalid("duplicate version ID".into()));
        }
    }
    if !volumes.is_empty() {
        return Err(FormatError::Invalid("unexpected voxel payload".into()));
    }
    let document = Document {
        id: wire.document.id,
        content: Snapshot {
            name: wire.document.name,
            bounds: wire.document.bounds,
            palette: wire.palette,
            layers: wire.layers,
            comments: wire.comments,
            extra: wire.extra,
        },
        versions,
        lineage: wire.lineage,
    };
    document.validate()?;
    let mut project = Project {
        document,
        version: wire.version,
        view: wire.view,
        template: metadata,
        build_history: None,
        built_document: None,
    };
    if let Some(history) = wire.build_history {
        // Recipes are executable typed data, unlike inert optional metadata.
        // Reject unknown command fields and noncanonical UUID spellings rather
        // than silently dropping or reinterpreting them on a subsequent save.
        if serde_json::to_value(&history)? != project.template["build_history"] {
            return Err(FormatError::Invalid(
                "noncanonical build command fields".into(),
            ));
        }
        project.set_build_history(history)?;
    }
    Ok(project)
}
/// Parse an in-memory input without copying its voxel text.
pub fn from_bytes(bytes: &[u8]) -> FormatResult<Project> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(FormatError::Invalid("128 MiB input limit".into()));
    }
    read(bytes)
}

fn known(path: &str) -> Option<&'static [&'static str]> {
    let last = path.rsplit('/').next().unwrap_or("");
    if path.is_empty() {
        return Some(&[
            "format",
            "version",
            "required_features",
            "document",
            "palette",
            "layers",
            "comments",
            "versions",
            "lineage",
            "view",
            "build_history",
        ]);
    }
    match last {
        "document" => Some(&["id", "name", "bounds"]),
        "bounds" => Some(&["min", "max"]),
        "palette" => Some(&["order", "slots"]),
        "version" => Some(&["major", "minor"]),
        "lineage" => Some(&["document", "version"]),
        "snapshot" => Some(&["name", "bounds", "palette", "layers", "comments"]),
        "anchor" => Some(&["kind", "at", "min", "max"]),
        _ => {
            let parent = path.rsplit('/').nth(1).unwrap_or("");
            match parent {
                "layers" => Some(&["id", "name", "visible", "locked", "voxels"]),
                "slots" => Some(&["index", "rgba", "name"]),
                "comments" => Some(&["id", "anchor", "layer", "resolved", "messages"]),
                "messages" => Some(&["id", "origin", "text", "tool"]),
                "versions" => Some(&["id", "name", "parent", "origin", "content_hash", "snapshot"]),
                _ => None,
            }
        }
    }
}
// Known fields come from live content. Unknown fields are merged by stable IDs,
// never by array position, so a layer reorder cannot move extension ownership.
fn preserve_unknown(current: &mut Value, old: &Value, path: &str) {
    if path == "/build_history" {
        return;
    }
    match (current, old) {
        (Value::Object(new), Value::Object(old)) => {
            let keys = known(path).unwrap_or(&[]);
            for (k, v) in old {
                if !keys.contains(&k.as_str()) && !new.contains_key(k) {
                    new.insert(k.clone(), v.clone());
                }
            }
            for (k, v) in new.iter_mut() {
                if let Some(before) = old.get(k)
                    && keys.contains(&k.as_str())
                {
                    preserve_unknown(v, before, &format!("{path}/{k}"));
                }
            }
        }
        (Value::Array(new), Value::Array(old)) => {
            for (i, v) in new.iter_mut().enumerate() {
                let identity = v
                    .get("id")
                    .map(|id| ("id", id))
                    .or_else(|| v.get("index").map(|id| ("index", id)));
                let before =
                    identity.and_then(|(key, id)| old.iter().find(|v| v.get(key) == Some(id)));
                if let Some(before) = before {
                    preserve_unknown(v, before, &format!("{path}/{i}"));
                }
            }
        }
        _ => {}
    }
}
fn validate_metadata(value: &Value) -> FormatResult<()> {
    fn inert(v: &Value, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        match v {
            Value::Number(n) => n.as_i64().is_some(),
            Value::Array(a) => a.iter().all(|v| inert(v, depth + 1)),
            Value::Object(o) => o.values().all(|v| inert(v, depth + 1)),
            _ => true,
        }
    }
    fn walk(v: &Value, path: &str) -> FormatResult<()> {
        match v {
            Value::Object(o) => {
                if let Some(keys) = known(path) {
                    for (k, v) in o {
                        if keys.contains(&k.as_str()) {
                            if ["id", "parent", "layer"].contains(&k.as_str()) && !v.is_null() {
                                let text = v.as_str().ok_or_else(|| {
                                    FormatError::Invalid("UUID must be a string".into())
                                })?;
                                let id = Uuid::parse_str(text)
                                    .map_err(|_| FormatError::Invalid("invalid UUID".into()))?;
                                if id.to_string() != text {
                                    return Err(FormatError::Invalid(
                                        "UUID must be lowercase hyphenated".into(),
                                    ));
                                }
                            }
                            if k != "view" {
                                walk(v, &format!("{path}/{k}"))?;
                            }
                        } else if !inert(v, 0) {
                            return Err(FormatError::Invalid(format!(
                                "unsupported optional metadata at {path}/{k}"
                            )));
                        }
                    }
                }
            }
            Value::Array(a) => {
                for (i, v) in a.iter().enumerate() {
                    walk(v, &format!("{path}/{i}"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    walk(value, "")?;
    let bytes = serde_json::to_vec(value)?.len();
    if bytes > MAX_METADATA_BYTES {
        return Err(FormatError::Invalid("8 MiB metadata budget".into()));
    }
    let comments = serde_json::to_vec(value.get("comments").unwrap_or(&Value::Null))?.len();
    if comments > 4 * 1024 * 1024 || bytes.saturating_sub(comments) > 4 * 1024 * 1024 {
        return Err(FormatError::Invalid("comment/other metadata budget".into()));
    }
    Ok(())
}
