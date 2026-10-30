//! Read-only export callbacks and their portable errors.
use voxelite_mesh::MeshError;

/// Export validation, encoding or output failure.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    /// Geometry generation failed.
    #[error(transparent)]
    Geometry(#[from] MeshError),
    /// PNG encoder error.
    #[error(transparent)]
    Png(#[from] png::EncodingError),
    /// I/O failed (the caller should stage file replacements).
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Output or pixel limits were exceeded.
    #[error("export limit or input error: {0}")]
    Invalid(String),
}
/// Result of a checked export.
pub type ExportResult<T> = Result<T, ExportError>;

/// Captured RGBA8 pixels for a host-selected view or contact sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportImage {
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// Row-major, top-to-bottom straight-alpha sRGB pixels.
    pub rgba: Vec<u8>,
}

/// Stateless export implementation. Returns bytes; only the host can write them.
/// Function pointers can run on a worker without sending plugin instances or GUI state.
pub type ExportCallback = fn(&dyn crate::ExportCx) -> ExportResult<Vec<u8>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_errors_preserve_variants_and_messages() {
        let geometry = ExportError::from(MeshError::Limit("too many quads".into()));
        let png = ExportError::from(png::EncodingError::from(std::io::Error::other(
            "png output",
        )));
        let io = ExportError::from(std::io::Error::other("disk full"));
        let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let json_message = json.to_string();
        let json = ExportError::from(json);
        assert!(matches!(geometry, ExportError::Geometry(_)));
        assert!(matches!(png, ExportError::Png(_)));
        assert!(matches!(io, ExportError::Io(_)));
        assert!(matches!(json, ExportError::Json(_)));
        for (error, expected) in [
            (geometry, "render limit exceeded: too many quads"),
            (png, "png output"),
            (io, "disk full"),
            (json, json_message.as_str()),
            (
                ExportError::Invalid("bad pixels".into()),
                "export limit or input error: bad pixels",
            ),
        ] {
            assert_eq!(error.to_string(), expected);
        }
    }
}
