//! Build-time product metadata shared by repository tools.
use serde::Deserialize;

/// A current branding value; stability is audited by xtask.
#[derive(Deserialize)]
pub struct Entry<T> {
    /// Current published value.
    pub current: T,
}

/// Product copy.
#[derive(Deserialize)]
pub struct Product {
    /// Display name.
    pub display_name: Entry<String>,
    /// Short product description.
    pub tagline: Entry<String>,
}

/// Identifiers used by site and installation examples.
#[derive(Deserialize)]
pub struct Identifiers {
    /// Production hostname.
    pub site_domain: Entry<String>,
    /// Canonical repository URL.
    pub github_repo_url: Entry<String>,
    /// Installed app bundle filename.
    pub app_bundle: Entry<String>,
    /// Downloaded release filename.
    pub release_asset: Entry<String>,
    /// MCP executable filename.
    pub mcp_binary: Entry<String>,
    /// MCP server configuration key.
    pub mcp_server_name: Entry<String>,
}

/// Parsed brand inventory and workspace version.
#[derive(Deserialize)]
pub struct Brand {
    /// Human-facing copy.
    pub product: Product,
    /// Stable installation and URL identifiers.
    pub identifiers: Identifiers,
    /// Version from the workspace manifest, never duplicated in brand.toml.
    #[serde(skip)]
    pub version: String,
}

#[derive(Deserialize)]
struct Manifest {
    workspace: Workspace,
}
#[derive(Deserialize)]
struct Workspace {
    package: Package,
}
#[derive(Deserialize)]
struct Package {
    version: String,
}

impl Brand {
    /// Parse the inventory and workspace manifest, rejecting missing metadata.
    pub fn parse(inventory: &str, manifest: &str) -> Result<Self, toml::de::Error> {
        let mut brand: Self = toml::from_str(inventory)?;
        let manifest: Manifest = toml::from_str(manifest)?;
        brand.version = manifest.workspace.package.version;
        Ok(brand)
    }

    /// Canonical HTTPS origin.
    pub fn site_url(&self) -> String {
        format!("https://{}", self.identifiers.site_domain.current)
    }

    /// Latest release asset endpoint, derived from repository and filename.
    pub fn download_url(&self) -> String {
        format!(
            "{}/releases/latest/download/{}",
            self.identifiers.github_repo_url.current, self.identifiers.release_asset.current
        )
    }

    /// Absolute executable path in the standard macOS installation.
    pub fn mcp_command(&self) -> String {
        format!(
            "/Applications/{}/Contents/MacOS/{}",
            self.identifiers.app_bundle.current, self.identifiers.mcp_binary.current
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_links_installation_and_version_from_inputs() {
        let source = include_str!("../../../brand.toml")
            .replace("Quantum", "Example")
            .replace("quantum.worldwidewebb.co", "example.test")
            .replace("0x63616c/quantum", "owner/example");
        let brand = Brand::parse(&source, "[workspace.package]\nversion = '9.8.7'\n").unwrap();
        assert_eq!(brand.product.display_name.current, "Example");
        assert_eq!(brand.version, "9.8.7");
        assert_eq!(brand.site_url(), "https://example.test");
        assert_eq!(
            brand.download_url(),
            "https://github.com/owner/example/releases/latest/download/Example.dmg"
        );
        assert_eq!(
            brand.mcp_command(),
            "/Applications/Example.app/Contents/MacOS/quantum-mcp"
        );
    }

    #[test]
    fn missing_inventory_or_workspace_version_is_an_error() {
        let source = include_str!("../../../brand.toml");
        assert!(Brand::parse("[product]", "[workspace.package]\nversion = '1.0.0'").is_err());
        assert!(Brand::parse(source, "[workspace.package]").is_err());
    }
}
