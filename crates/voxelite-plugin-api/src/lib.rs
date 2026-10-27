//! Portable declarations and capability boundaries for Voxelite's plugins.
//!
//! Built-in plugins use this API to declare features without access to the app.
//! Contributions contain data only; the host interprets localization keys,
//! schemas, icons, menus and keybindings. Tools and command transforms return
//! typed commands through read-only capabilities; only the host may apply edits.
//! There is no GUI dependency, registry, enablement or lifecycle management here,
//! and no third-party code loading, WASM or scripting.
mod context;
mod data;
mod export;
mod id;
mod overlay;
mod palette;
mod registrar;
mod spec;
mod tool;
mod transform;

pub use context::{ActivateCx, CommandCx, ExportCx, ToolCx, TransformCx};
pub use data::*;
pub use export::*;
pub use id::{InvalidPluginId, PluginId};
pub use overlay::*;
pub use palette::*;
pub use registrar::{Contributions, Registrar, ValidationError, validate_contributions};
pub use semver::Version;
/// Portable JSON codecs for plugin-owned settings, without filesystem access.
pub use serde_json as json;
pub use serde_json::Value;
pub use spec::{
    ApplicationMenuSection, CommandContribution, CommandTransformContribution, Contribution,
    DataSectionContribution, FormatContent, FormatContribution, Manifest, PanelContribution,
    RenderPassContribution, RenderPassSlot, SettingContribution, SettingKind, SettingValue,
    ToolContribution,
};
pub use tool::{Tool, ToolError, ToolGesture, ToolHit, ToolInput, ToolLayer, ToolOutcome};
pub use transform::CommandTransform;
pub use voxelite_core::{
    Command, Error as CoreError, Mirror, Mode, Palette, PaletteEdit, PaletteSlot, Pos, Region,
    Shape, Snapshot, Swatch, Uuid, Voxel, validate_name,
};

/// Version of this Rust API contract, independent of plugin and `.voxl` versions.
///
/// Before 1.0, breaking changes bump the minor version; compatible changes bump
/// the patch version. From 1.0, breaking changes bump major, additions bump minor
/// and fixes bump patch. Compatibility checks belong to the host.
pub const API_VERSION: Version = Version::new(0, 6, 0);

/// A statically linked plugin that declares metadata and host contributions.
pub trait Plugin {
    /// Describe this plugin and its enablement policy.
    fn manifest(&self) -> Manifest;
    /// Contribute declarations; hosts bind implementations later by ID.
    fn contribute(&self, registrar: &mut Registrar);
    /// Forward migrations for a declared section, invoked only while enabled.
    fn data_migrations(&self, _section_name: &str) -> Vec<DataMigration> {
        vec![]
    }
    /// Validate or initialize this plugin's sections after all its migrations.
    /// Failure aborts the entire load operation without changing stored data.
    fn load_data(&self, _cx: &mut dyn DataCx) -> DataResult<()> {
        Ok(())
    }
    /// Bind a declared tool ID to its implementation. Hosts gate calls on enablement.
    fn tool(&self, _id: &str) -> Option<&dyn Tool> {
        None
    }
    /// Bind a declared command transform. Hosts gate calls on enablement.
    fn command_transform(&self, _id: &str) -> Option<&dyn CommandTransform> {
        None
    }
    /// Bind a declared render pass to portable overlay drawing and interaction.
    fn overlay(&self, _id: &str) -> Option<&dyn Overlay> {
        None
    }
    /// Bind a declared export format to a portable, stateless byte encoder.
    fn exporter(&self, _id: &str) -> Option<ExportCallback> {
        None
    }
    /// Bind portable palette behavior to a declared panel, without GUI access.
    fn palette_panel(&self, _id: &str) -> Option<&dyn PalettePanel> {
        None
    }
    /// Initialize plugin state using only capabilities supplied by the host.
    fn activate(&mut self, _cx: &mut dyn ActivateCx) {}
    /// Release plugin state when the host deactivates this plugin.
    fn deactivate(&mut self) {}
}

#[cfg(test)]
mod tests;
