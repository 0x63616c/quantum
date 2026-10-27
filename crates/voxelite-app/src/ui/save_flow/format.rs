//! Core project saving plus formats declared by enabled plugins.
use voxelite_plugin_api::FormatContribution;
use voxelite_plugin_host::Host;

/// One choice in the Save/Export dialog.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SaveFormat {
    /// Native persistence is always available, independent of plugins.
    #[default]
    Project,
    /// Metadata captured from the enabled registry when the dialog opens.
    Export(FormatContribution),
}

impl SaveFormat {
    /// Current choices in registry order, with native project saving first.
    pub fn available(host: &Host) -> Vec<Self> {
        std::iter::once(Self::Project)
            .chain(
                host.registry()
                    .formats()
                    .filter(|(_, format)| {
                        format.export
                            && format.content == voxelite_plugin_api::FormatContent::Document
                            && !format.extensions.is_empty()
                    })
                    .map(|(_, format)| Self::Export(format.clone())),
            )
            .collect()
    }

    /// Preferred extension without the leading dot.
    pub fn extension(&self) -> &str {
        match self {
            Self::Project => "voxl",
            Self::Export(format) => &format.extensions[0],
        }
    }

    /// Display label supplied by the format's owner.
    pub fn label(&self) -> &str {
        match self {
            Self::Project => "Voxelite Project (.voxl)",
            Self::Export(format) => &format.label_key,
        }
    }

    /// Match only offered extensions, preserving native saving as the fallback.
    pub fn from_extension(formats: &[Self], extension: &str) -> Self {
        formats
            .iter()
            .find(|format| match format {
                Self::Project => extension.eq_ignore_ascii_case("voxl"),
                Self::Export(format) => format
                    .extensions
                    .iter()
                    .any(|ext| extension.eq_ignore_ascii_case(ext)),
            })
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_follow_enabled_registry_and_project_save_survives_disable() {
        let mut host = crate::app_host();
        let formats = SaveFormat::available(&host);
        assert_eq!(
            formats
                .iter()
                .map(|f| (f.extension(), f.label()))
                .collect::<Vec<_>>(),
            [
                ("voxl", "Voxelite Project (.voxl)"),
                ("glb", "glTF Binary (.glb)"),
                ("png", "PNG Image (.png)"),
            ]
        );
        for format in &formats {
            assert_eq!(
                SaveFormat::from_extension(&formats, &format.extension().to_uppercase()),
                *format
            );
        }
        for extension in ["", "json", "jpeg", "gltf"] {
            assert_eq!(
                SaveFormat::from_extension(&formats, extension),
                SaveFormat::Project
            );
        }
        host.disable("dev.voxelite.exporters").unwrap();
        assert_eq!(SaveFormat::available(&host), [SaveFormat::Project]);
        host.enable("dev.voxelite.exporters").unwrap();
        assert_eq!(SaveFormat::available(&host), formats);
    }
}
