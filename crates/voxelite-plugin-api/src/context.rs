/// Host-provided capabilities for executing a command.
///
/// This trait is a stub. Future operations will mediate validated document edits
/// and command feedback without exposing the app or its UI implementation.
pub trait CommandCx {}

/// Host-provided capabilities for an interactive viewport tool.
///
/// This trait is a stub. Input, picking and edit operations will be added when
/// tools are integrated; plugins will not receive the app or a GUI context.
pub trait ToolCx {}

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
