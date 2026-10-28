//! Keep the branding inventory aligned with source, manifests and packaging.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    Error, Result,
    check::{Context, Outcome},
    workspace::Workspace,
};

type Inventory = BTreeMap<String, BTreeMap<String, Entry>>;

#[derive(Deserialize)]
struct Entry {
    current: Value,
    stable: bool,
    next: Option<Value>,
}

pub(crate) fn run(context: &Context<'_>) -> Outcome {
    super::outcome(check(context.workspace))
}

fn read_toml<T: DeserializeOwned>(workspace: &Workspace, path: &str) -> Result<T> {
    toml::from_str(&workspace.read(path)?)
        .map_err(|error| Error::Invalid(format!("{path}: {error}")))
}

struct Audit<'a> {
    workspace: &'a Workspace,
    inventory: Inventory,
    errors: Vec<String>,
}

impl Audit<'_> {
    fn current(&self, group: &str, name: &str) -> Result<Value> {
        self.inventory
            .get(group)
            .and_then(|entries| entries.get(name))
            .map(|entry| entry.current.clone())
            .ok_or_else(|| Error::Invalid(format!("brand.toml: missing {group}.{name}.current")))
    }

    fn text(&self, group: &str, name: &str) -> Result<String> {
        self.current(group, name)?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "brand.toml: {group}.{name}.current must be a string"
                ))
            })
    }

    fn identifier(&self, name: &str) -> Result<String> {
        self.text("identifiers", name)
    }

    fn require(&mut self, condition: bool, message: impl Into<String>) {
        if !condition {
            self.errors.push(message.into());
        }
    }

    fn equal(&mut self, label: &str, actual: impl Into<Value>, expected: impl Into<Value>) {
        let (actual, expected) = (actual.into(), expected.into());
        self.require(
            actual == expected,
            format!("{label}: expected {expected}, found {actual}"),
        );
    }

    fn capture(&mut self, path: &str, pattern: &str, expected: &str) -> Result<()> {
        let values = super::captures(&self.workspace.read(path)?, pattern)?;
        self.equal(path, values, vec![expected]);
        Ok(())
    }

    fn stability(&mut self) -> Result<()> {
        for (group, entries) in &self.inventory {
            for (name, entry) in entries {
                if entry.stable
                    && entry
                        .next
                        .as_ref()
                        .is_some_and(|next| next != &entry.current)
                {
                    self.errors.push(format!(
                        "brand.toml: stable {group}.{name} must keep its current value"
                    ));
                }
            }
        }
        Ok(())
    }

    fn bundle(&mut self) -> Result<()> {
        let path = "packaging/macos/Info.plist";
        let bytes = std::fs::read(self.workspace.path(path)).map_err(|source| Error::Io {
            path: self.workspace.path(path),
            source,
        })?;
        let info: Bundle = plist::from_bytes(&bytes)
            .map_err(|error| Error::Invalid(format!("{path}: {error}")))?;
        for (key, actual, name) in [
            ("CFBundleIdentifier", &info.identifier, "bundle_id"),
            ("CFBundleName", &info.name, "bundle_name"),
            (
                "CFBundleDisplayName",
                &info.display_name,
                "bundle_display_name",
            ),
            ("CFBundleExecutable", &info.executable, "bundle_executable"),
            ("CFBundleIconFile", &info.icon, "bundle_icon"),
        ] {
            self.equal(
                &format!("{path}: {key}"),
                actual.as_str(),
                self.identifier(name)?,
            );
        }
        self.equal(
            "bundle display name",
            info.display_name,
            self.text("product", "display_name")?,
        );
        self.equal(
            "exported UTIs",
            info.exported
                .iter()
                .map(|item| item.identifier.as_str())
                .collect::<Vec<_>>(),
            self.current("identifiers", "exported_utis")?,
        );
        self.equal(
            "imported UTIs",
            info.imported
                .iter()
                .map(|item| item.identifier.as_str())
                .collect::<Vec<_>>(),
            self.current("identifiers", "imported_utis")?,
        );
        self.equal(
            "document type IDs",
            info.documents
                .into_iter()
                .flat_map(|item| item.content_types)
                .collect::<Vec<_>>(),
            self.current("identifiers", "document_type_ids")?,
        );
        Ok(())
    }

    fn windows(&mut self) -> Result<()> {
        self.capture(
            "crates/quantum-app/src/lib.rs",
            r#"pub\s+const\s+APP_ID\s*:\s*&str\s*=\s*"([^"]+)""#,
            &self.identifier("window_app_id")?,
        )?;
        self.equal(
            "window app ID",
            self.identifier("window_app_id")?,
            self.identifier("bundle_id")?,
        );
        let hardcoded = regex::Regex::new(r#"app_id\s*:\s*Some\(\s*""#)
            .map_err(|error| Error::Invalid(error.to_string()))?;
        for name in ["welcome", "settings"] {
            let path = format!("crates/quantum-app/src/ui/{name}.rs");
            let source = self.workspace.read(&path)?;
            self.require(
                source.contains("app_id: Some(quantum_app::APP_ID.into())"),
                format!("{path}: expected shared window APP_ID"),
            );
            self.require(
                !hardcoded.is_match(&source),
                format!("{path}: window app_id must not be a string literal"),
            );
        }
        Ok(())
    }

    fn packages(&mut self) -> Result<()> {
        let root: RootManifest = read_toml(self.workspace, "Cargo.toml")?;
        let manifests = member_manifests(self.workspace, &root.workspace)?;
        self.require(
            !manifests.is_empty(),
            "Cargo.toml: expected workspace member manifests",
        );
        let prefix = self.identifier("crate_name_prefix")?;
        for path in manifests {
            if path == self.workspace.path("xtask/Cargo.toml") {
                continue;
            }
            let relative = path
                .strip_prefix(self.workspace.root())
                .unwrap_or(&path)
                .to_string_lossy();
            let manifest: PackageManifest = read_toml(self.workspace, &relative)?;
            self.require(
                manifest.package.name.starts_with(&prefix),
                format!(
                    "{relative}: package {:?} must start with {prefix:?}",
                    manifest.package.name
                ),
            );
        }
        let app: PackageManifest = read_toml(self.workspace, "crates/quantum-app/Cargo.toml")?;
        let binary = self.identifier("app_binary")?;
        self.require(
            app.bin.iter().any(|bin| bin.name == binary),
            format!("crates/quantum-app/Cargo.toml: missing binary {binary:?}"),
        );
        let mcp: PackageManifest = read_toml(self.workspace, "crates/quantum-mcp/Cargo.toml")?;
        self.equal(
            "MCP package name",
            mcp.package.name,
            self.identifier("mcp_binary")?,
        );
        self.require(
            self.workspace
                .path("crates/quantum-mcp/src/main.rs")
                .is_file(),
            "crates/quantum-mcp/src/main.rs: missing MCP binary entry point",
        );
        Ok(())
    }

    fn mcp(&mut self) -> Result<()> {
        for (path, pattern, name) in [
            (
                "catalog",
                r#"format!\(\s*"([^"]+)\{name\}"\s*\)"#,
                "mcp_tool_prefix",
            ),
            (
                "session",
                r#"\.strip_prefix\(\s*"([^"]+)"\s*\)"#,
                "mcp_tool_prefix",
            ),
            (
                "lib",
                r#"const\s+API_VERSION\s*:\s*&str\s*=\s*"([^"]+)""#,
                "mcp_api_version",
            ),
        ] {
            self.capture(
                &format!("crates/quantum-mcp/src/{path}.rs"),
                pattern,
                &self.identifier(name)?,
            )?;
        }
        Ok(())
    }

    fn site(&mut self) -> Result<()> {
        let domain = self.identifier("site_domain")?;
        self.capture(
            "crates/quantum-site/src/main.rs",
            r#"const\s+SITE_BASE_URL\s*:\s*&str\s*=\s*"([^"]+)""#,
            &format!("https://{domain}"),
        )?;
        self.capture(
            "crates/quantum-site/templates/github-link.html",
            r#"href="([^"]+)""#,
            &self.identifier("github_repo_url")?,
        )?;
        let appcast =
            url::Url::parse(&self.identifier("sparkle_appcast_url")?).map_err(|error| {
                Error::Invalid(format!("brand.toml: invalid sparkle_appcast_url: {error}"))
            })?;
        self.equal("appcast hostname", json!(appcast.host_str()), domain);
        let tagline = self.text("product", "tagline")?;
        self.require(
            self.workspace
                .read("README.md")?
                .lines()
                .any(|line| line == tagline),
            "README.md: missing product tagline line",
        );
        Ok(())
    }

    fn project_format(&mut self) -> Result<()> {
        let path = format!("{}/schema.json", self.identifier("format_spec")?);
        let schema: Value =
            serde_json::from_str(&self.workspace.read(&path)?).map_err(|source| Error::Json {
                context: path.clone(),
                source,
            })?;
        self.equal(
            &format!("{path}: format const"),
            schema["properties"]["format"]["const"].clone(),
            self.identifier("format_magic")?,
        );
        for (pattern, name) in [
            (r#"format:\s*"([^"]+)"\.into\(\)"#, "format_magic"),
            (r#"wire\.format\s*!=\s*"([^"]+)""#, "format_magic"),
            (
                r#"const\s+MIME_TYPE\s*:\s*&str\s*=\s*"([^"]+)""#,
                "format_mime_type",
            ),
        ] {
            self.capture(
                "crates/quantum-format/src/lib.rs",
                pattern,
                &self.identifier(name)?,
            )?;
        }
        let path = "crates/quantum-app/src/ui/save_flow/format.rs";
        let labels = super::captures(
            &self.workspace.read(path)?,
            r#"Self::Project\s*=>\s*"([^"]+)""#,
        )?;
        let extension = self.identifier("file_extension")?;
        self.equal(
            path,
            labels,
            vec![
                extension.strip_prefix('.').unwrap_or(&extension).to_owned(),
                self.identifier("document_label")?,
            ],
        );
        Ok(())
    }

    fn storage(&mut self) -> Result<()> {
        let path = "crates/quantum-app/src/local_settings.rs";
        let source = self.workspace.read(path)?;
        self.storage_source(&source)?;
        let documents = self.text("storage", "documents_directory")?;
        self.capture(
            "crates/quantum-app/src/attachment.rs",
            r#"home\.join\(\s*"([^"]+)"\s*\)"#,
            documents.strip_prefix("~/").unwrap_or(&documents),
        )?;
        Ok(())
    }

    // Match only the two path-selection functions, excluding migration and test code.
    fn storage_source(&mut self, source: &str) -> Result<()> {
        let data_dir = super::captures(source, r"(?ms)^pub fn data_dir\(\) -> PathBuf \{(.*?)^\}")?;
        let settings_dir = super::captures(
            source,
            r"(?ms)^fn settings_dir\([^\n]*\) -> PathBuf \{(.*?)^\}",
        )?;
        self.equal("data_dir function count", data_dir.len(), 1);
        self.equal("settings_dir function count", settings_dir.len(), 1);
        self.equal(
            "data_dir: environment variables",
            super::captures(&data_dir.join(""), r#"var_os\(\s*"([^"]+)"\s*\)"#)?,
            vec![
                self.text("storage", "data_directory_env")?,
                "HOME".into(),
                "XDG_DATA_HOME".into(),
            ],
        );
        let parts = super::captures(&settings_dir.join(""), r#"\.join\(\s*"([^"]+)"\s*\)"#)?;
        let directory = self.text("storage", "data_directory_name")?;
        self.equal(
            "settings_dir: path components",
            parts,
            vec![
                self.text("storage", "macos_data_directory")?
                    .strip_prefix("~/")
                    .ok_or_else(|| {
                        Error::Invalid("macos_data_directory must start with ~/".into())
                    })?
                    .to_owned(),
                ".local/share".to_owned(),
                directory.clone(),
            ],
        );
        self.equal(
            "fallback data directory",
            self.text("storage", "fallback_data_directory")?,
            format!("~/.local/share/{directory}"),
        );
        self.equal(
            "XDG data directory",
            self.text("storage", "xdg_data_directory")?,
            format!("$XDG_DATA_HOME/{directory}"),
        );
        Ok(())
    }

    fn skills(&mut self) -> Result<()> {
        let skills = super::paths(self.workspace, "skills/*/SKILL.md")?;
        self.require(!skills.is_empty(), "skills: expected at least one SKILL.md");
        let prefix = self.identifier("skills_name_prefix")?;
        for path in skills {
            let name = path
                .parent()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .ok_or_else(|| Error::Invalid(format!("invalid skill path: {}", path.display())))?;
            self.require(
                name.starts_with(&prefix),
                format!("{}: skill name must start with {prefix:?}", path.display()),
            );
            let names = super::captures(&self.workspace.read(&path)?, r"(?m)^name:\s*(\S+)\s*$")?;
            self.equal(&path.display().to_string(), names, vec![name]);
        }
        Ok(())
    }
}

fn check(workspace: &Workspace) -> Result<Vec<String>> {
    let mut audit = Audit {
        workspace,
        inventory: read_toml(workspace, "brand.toml")?,
        errors: Vec::new(),
    };
    for check in [
        Audit::stability,
        Audit::bundle,
        Audit::windows,
        Audit::packages,
        Audit::mcp,
        Audit::site,
        Audit::project_format,
        Audit::storage,
        Audit::skills,
    ] {
        if let Err(error) = check(&mut audit) {
            audit.errors.push(error.to_string());
        }
    }
    Ok(audit.errors)
}

#[derive(Deserialize)]
struct Bundle {
    #[serde(rename = "CFBundleIdentifier")]
    identifier: String,
    #[serde(rename = "CFBundleName")]
    name: String,
    #[serde(rename = "CFBundleDisplayName")]
    display_name: String,
    #[serde(rename = "CFBundleExecutable")]
    executable: String,
    #[serde(rename = "CFBundleIconFile")]
    icon: String,
    #[serde(default, rename = "UTExportedTypeDeclarations")]
    exported: Vec<TypeDeclaration>,
    #[serde(default, rename = "UTImportedTypeDeclarations")]
    imported: Vec<TypeDeclaration>,
    #[serde(default, rename = "CFBundleDocumentTypes")]
    documents: Vec<DocumentType>,
}

#[derive(Deserialize)]
struct TypeDeclaration {
    #[serde(rename = "UTTypeIdentifier")]
    identifier: String,
}

#[derive(Deserialize)]
struct DocumentType {
    #[serde(default, rename = "LSItemContentTypes")]
    content_types: Vec<String>,
}

#[derive(Deserialize)]
struct RootManifest {
    workspace: Members,
}

#[derive(Deserialize)]
struct Members {
    members: Vec<String>,
    exclude: Vec<String>,
}

fn member_manifests(
    workspace: &Workspace,
    members: &Members,
) -> Result<BTreeSet<std::path::PathBuf>> {
    let mut manifests = BTreeSet::new();
    for member in &members.members {
        manifests.extend(super::paths(workspace, &format!("{member}/Cargo.toml"))?);
    }
    for exclude in &members.exclude {
        manifests.remove(&workspace.path(exclude));
    }
    Ok(manifests)
}

#[derive(Deserialize)]
struct PackageManifest {
    package: Named,
    #[serde(default)]
    bin: Vec<Named>,
}

#[derive(Deserialize)]
struct Named {
    name: String,
}

#[cfg(test)]
mod tests;
