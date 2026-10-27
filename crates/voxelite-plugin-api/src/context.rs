/// Host-provided capabilities for executing a command.
///
/// This trait is a stub. Future operations will mediate validated document edits
/// and command feedback without exposing the app or its UI implementation.
pub trait CommandCx {}

/// Read-only document and picking capabilities for a tool invocation.
///
/// No engine, app, GUI, mirror settings or mutation capabilities are exposed.
/// Palette slot zero denotes air; queries use the live document, not a history preview.
pub trait ToolCx {
    /// Nearest visible voxel and face for this pointer, if any.
    fn hovered_hit(&self) -> Option<crate::ToolHit>;
    /// Pointer intersection with the bottom work plane, already bounded by the host.
    fn ground_target(&self) -> Option<crate::Pos>;
    /// Visible composite sample at an explicit coordinate (palette slot, layer).
    fn sample(&self, at: crate::Pos) -> Option<(u16, crate::Uuid)>;
    /// Palette slot in a specific layer, including zero for empty space.
    fn voxel(&self, layer: crate::Uuid, at: crate::Pos) -> Result<u16, crate::CoreError>;
    /// Stable palette slots and their original colors.
    fn palette(&self) -> &crate::Palette;
    /// Selected brush color, which may not yet belong to the document palette.
    fn active_color(&self) -> &crate::Swatch;
    /// Selected layer identity.
    fn active_layer(&self) -> crate::Uuid;
    /// Visibility and lock state of a layer, without granting access to its storage.
    fn layer(&self, id: crate::Uuid) -> Result<crate::ToolLayer, crate::CoreError>;
    /// Finite, half-open document bounds.
    fn bounds(&self) -> crate::Region;
}

/// Read-only capabilities for transforming pending tool commands.
///
/// Settings are scoped to the declaring plugin, including its enabled mirror
/// axes. No other plugin's settings, document mutations or GUI are exposed.
pub trait TransformCx {
    /// Finite, half-open bounds of the live document.
    fn bounds(&self) -> crate::Region;
    /// Current value of one of this plugin's declared settings.
    fn setting(&self, key: &str) -> Option<&crate::SettingValue>;
}

/// Host-provided capabilities for exporting document content.
///
/// This trait is a stub. Snapshot and output capabilities will be added with
/// exporters, without granting general app or filesystem access.
pub trait ExportCx {}

/// Host-provided capabilities available during plugin activation.
///
/// This trait is a stub, intentionally separate from command, tool and export
/// capabilities. The host owns lifecycle ordering and capability grants.
pub trait ActivateCx {}
