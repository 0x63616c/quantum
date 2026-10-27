//! Portable declarations and capability boundaries for Voxelite's plugins.
//!
//! Built-in plugins use this API to declare features without access to the app.
//! Contributions contain data only; the host interprets localization keys,
//! schemas, icons, menus and keybindings. Tools and command transforms return
//! typed commands through read-only capabilities; only the host may apply edits.
//! There is no GUI dependency, registry, enablement or lifecycle management here,
//! and no third-party code loading, WASM or scripting.
mod context;
mod id;
mod registrar;
mod spec;
mod tool;
mod transform;

pub use context::{ActivateCx, CommandCx, ExportCx, ToolCx, TransformCx};
pub use id::{InvalidPluginId, PluginId};
pub use registrar::{Contributions, Registrar, ValidationError, validate_contributions};
pub use semver::Version;
pub use serde_json::Value;
pub use spec::{
    ApplicationMenuSection, CommandContribution, CommandTransformContribution, Contribution,
    DataSectionContribution, FormatContribution, Manifest, PanelContribution,
    RenderPassContribution, RenderPassSlot, SettingContribution, SettingKind, SettingValue,
    ToolContribution,
};
pub use tool::{Tool, ToolError, ToolGesture, ToolHit, ToolInput, ToolLayer, ToolOutcome};
pub use transform::CommandTransform;
pub use voxelite_core::{
    Command, Error as CoreError, Mirror, Mode, Palette, PaletteEdit, PaletteSlot, Pos, Region,
    Shape, Swatch, Uuid, Voxel,
};

/// Version of this Rust API contract, independent of plugin and `.voxl` versions.
///
/// Before 1.0, breaking changes bump the minor version; compatible changes bump
/// the patch version. From 1.0, breaking changes bump major, additions bump minor
/// and fixes bump patch. Compatibility checks belong to the host.
pub const API_VERSION: Version = Version::new(0, 4, 0);

/// A statically linked plugin that declares metadata and host contributions.
pub trait Plugin {
    /// Describe this plugin and its enablement policy.
    fn manifest(&self) -> Manifest;
    /// Contribute declarations; hosts bind implementations later by ID.
    fn contribute(&self, registrar: &mut Registrar);
    /// Bind a declared tool ID to its implementation. Hosts gate calls on enablement.
    fn tool(&self, _id: &str) -> Option<&dyn Tool> {
        None
    }
    /// Bind a declared command transform. Hosts gate calls on enablement.
    fn command_transform(&self, _id: &str) -> Option<&dyn CommandTransform> {
        None
    }
    /// Initialize plugin state using only capabilities supplied by the host.
    fn activate(&mut self, _cx: &mut dyn ActivateCx) {}
    /// Release plugin state when the host deactivates this plugin.
    fn deactivate(&mut self) {}
}

#[cfg(test)]
mod tests;
