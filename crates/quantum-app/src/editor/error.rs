//! Typed shell failures retain their underlying causes and user-facing messages.
use quantum_format::session::SessionError;
use quantum_plugin_api::{PaletteError, ToolError};
use quantum_plugin_host::HostError;
use std::path::PathBuf;

/// Failures presented by the shell without dropping the current document.
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    /// Core validation or edit conflict.
    #[error(transparent)]
    Core(#[from] quantum_core::Error),
    /// Portable format error.
    #[error(transparent)]
    Format(#[from] quantum_format::FormatError),
    /// Filesystem error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A project could not be loaded; retains its path and underlying cause.
    #[error("Could not open {path}: {source}")]
    Open {
        /// Requested project path.
        path: PathBuf,
        /// Loading or editor initialization failure.
        source: Box<EditorError>,
    },
    /// Document persistence policy or durability failure.
    #[error(transparent)]
    Session(SessionError),
    /// Plugin dispatch, validation or quarantine failure.
    #[error(transparent)]
    Host(HostError),
    /// Palette input or library failure.
    #[error(transparent)]
    Palette(PaletteError),
    /// Local JSON serialization or parsing failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Preview rendering failed.
    #[error(transparent)]
    Render(#[from] quantum_render::RenderError),
    /// Preview PNG encoding failed.
    #[error(transparent)]
    Png(#[from] png::EncodingError),
    /// A native palette file dialog failed.
    #[error("{0}")]
    PaletteDialog(#[source] anyhow::Error),
    /// Imported palette text is not UTF-8.
    #[error(transparent)]
    PaletteUtf8(#[from] std::string::FromUtf8Error),
    /// Requested exporter is absent or disabled.
    #[error("Export format is unavailable")]
    ExportUnavailable,
    /// Comments plugin is absent or disabled.
    #[error("Comments plugin is unavailable")]
    CommentsUnavailable,
    /// Editing is forbidden while inspecting a history preview.
    #[error("History preview is read-only. Return to live or Restore.")]
    HistoryReadOnly,
    /// Requested named version was removed.
    #[error("Version no longer exists")]
    VersionMissing,
    /// Restore requires a selected preview.
    #[error("Choose a history state first")]
    HistorySelectionRequired,
    /// A palette proposal cannot atomically edit both durable authorities.
    #[error("Palette request mixes library and document edits")]
    MixedPaletteEdits,
    /// Settings bytes exceeded the host's read budget.
    #[error("Settings file too large")]
    SettingsTooLarge,
    /// A recent-project entry must reference a regular file.
    #[error("Project is not a file")]
    ProjectNotFile,
    /// The project changed before its generated preview could be cached.
    #[error("Project changed while creating its preview")]
    PreviewChanged,
    /// Palette imports exceeded the host's read budget.
    #[error("Palette files must be 64 KiB or smaller.")]
    PaletteImportTooLarge,
}
impl From<SessionError> for EditorError {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::Format(e) => Self::Format(e),
            SessionError::Io(e) => Self::Io(e),
            e => Self::Session(e),
        }
    }
}
impl From<HostError> for EditorError {
    fn from(error: HostError) -> Self {
        match error {
            HostError::Tool(ToolError::Core(error)) => Self::Core(error),
            HostError::Palette(error) => error.into(),
            error => Self::Host(error),
        }
    }
}
/// Result of an editor operation.
pub type EditorResult<T> = std::result::Result<T, EditorError>;

impl From<PaletteError> for EditorError {
    fn from(error: PaletteError) -> Self {
        match error {
            PaletteError::Core(error) => Self::Core(error),
            error => Self::Palette(error),
        }
    }
}

#[cfg(test)]
mod tests;
