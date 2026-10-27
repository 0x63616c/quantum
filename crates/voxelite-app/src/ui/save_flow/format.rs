//! Formats offered by the native save flow, without platform dependencies.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SaveFormat {
    #[default]
    Project,
    Glb,
    Png,
}

impl SaveFormat {
    pub(super) const ALL: [Self; 3] = [Self::Project, Self::Glb, Self::Png];

    pub(super) fn extension(self) -> &'static str {
        match self {
            Self::Project => "voxl",
            Self::Glb => "glb",
            Self::Png => "png",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Project => "Voxelite Project (.voxl)",
            Self::Glb => "glTF Binary (.glb)",
            Self::Png => "PNG Image (.png)",
        }
    }

    pub(super) fn from_extension(extension: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|format| extension.eq_ignore_ascii_case(format.extension()))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::SaveFormat;

    #[test]
    fn supported_formats_have_labels_and_case_insensitive_extensions() {
        for (format, extension, label) in [
            (SaveFormat::Project, "voxl", "Voxelite Project (.voxl)"),
            (SaveFormat::Glb, "glb", "glTF Binary (.glb)"),
            (SaveFormat::Png, "png", "PNG Image (.png)"),
        ] {
            assert_eq!(format.extension(), extension);
            assert_eq!(format.label(), label);
            assert_eq!(SaveFormat::from_extension(extension), format);
            assert_eq!(
                SaveFormat::from_extension(&extension.to_uppercase()),
                format
            );
        }
    }

    #[test]
    fn missing_or_unknown_extension_defaults_to_project() {
        assert_eq!(SaveFormat::default(), SaveFormat::Project);
        assert_eq!(SaveFormat::ALL[0], SaveFormat::Project);
        for extension in ["", "json", "jpeg", "gltf"] {
            assert_eq!(SaveFormat::from_extension(extension), SaveFormat::Project);
        }
    }
}
