//! Portable declarations and capability boundaries for Voxelite's plugins.
//!
//! Built-in plugins use this API to declare features without access to the app.
//! Contributions contain data only; the host interprets localization keys,
//! schemas, icons, menus and keybindings. Capability contexts are host-implemented
//! trait stubs, with operations to be added as features move behind this boundary.
//! There is no GUI dependency, registry, enablement or lifecycle management here,
//! and no third-party code loading, WASM or scripting.
mod context;
mod id;
mod registrar;
mod spec;

pub use context::{ActivateCx, CommandCx, ExportCx, ToolCx};
pub use id::{InvalidPluginId, PluginId};
pub use registrar::{Contributions, Registrar, ValidationError, validate_contributions};
pub use semver::Version;
pub use serde_json::Value;
pub use spec::{
    CommandContribution, Contribution, DataSectionContribution, FormatContribution, Manifest,
    PanelContribution, RenderPassContribution, RenderPassSlot, SettingContribution, SettingKind,
    SettingValue, ToolContribution,
};

/// Version of this Rust API contract, independent of plugin and `.voxl` versions.
///
/// Before 1.0, breaking changes bump the minor version; compatible changes bump
/// the patch version. From 1.0, breaking changes bump major, additions bump minor
/// and fixes bump patch. Compatibility checks belong to the host.
pub const API_VERSION: Version = Version::new(0, 1, 0);

/// A statically linked plugin that declares metadata and host contributions.
pub trait Plugin {
    /// Describe this plugin and its enablement policy.
    fn manifest(&self) -> Manifest;
    /// Contribute declarations; hosts bind implementations later by ID.
    fn contribute(&self, registrar: &mut Registrar);
    /// Initialize plugin state using only capabilities supplied by the host.
    fn activate(&mut self, _cx: &mut dyn ActivateCx) {}
    /// Release plugin state when the host deactivates this plugin.
    fn deactivate(&mut self) {}
}

#[cfg(test)]
mod tests;
