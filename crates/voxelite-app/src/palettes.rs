//! Persistent local palette libraries. Choosing a palette never recolors a project.
use crate::editor::{EditorError, EditorResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use voxelite_core::{Palette, Swatch, Uuid, validate_name};

/// A user-owned palette, independent of the document's stable slots.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryPalette {
    /// Stable local identity.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Ordered opaque swatches.
    pub palette: Palette,
}
/// Bounded custom library. Built-in presets are never mutated.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PaletteLibrary {
    /// Custom palettes in display order.
    pub entries: Vec<LibraryPalette>,
}
/// App-local settings directory, overridable for isolated tests and captures.
pub fn data_dir() -> PathBuf {
    std::env::var_os("VOXELITE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
                })
                .join("voxelite")
        })
}
/// Replace a local settings file atomically.
#[cfg(not(target_family = "wasm"))]
pub fn write_local(path: &Path, bytes: &[u8]) -> EditorResult<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or_else(|| EditorError::Gesture("Invalid settings path".into()))?;
    std::fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
impl PaletteLibrary {
    /// Read and validate a bounded library; a missing file starts empty.
    pub fn load(path: &Path) -> EditorResult<Self> {
        let Some(bytes) = read_local(path)? else {
            return Ok(Self::default());
        };
        let library: Self =
            serde_json::from_slice(&bytes).map_err(|e| EditorError::Gesture(e.to_string()))?;
        library.validate()?;
        Ok(library)
    }
    /// Check library limits, names, IDs, and every palette.
    pub fn validate(&self) -> EditorResult<()> {
        if self.entries.len() > 128 {
            return Err(EditorError::Gesture("128 custom palettes maximum".into()));
        }
        let mut ids = std::collections::BTreeSet::new();
        for entry in &self.entries {
            validate_name(&entry.name)?;
            entry.palette.validate()?;
            if !ids.insert(entry.id) {
                return Err(EditorError::Gesture("Duplicate palette identity".into()));
            }
        }
        Ok(())
    }
    /// Validate and persist before installing an edit, so I/O failures retain the old library.
    pub fn update(
        &mut self,
        path: &Path,
        edit: impl FnOnce(&mut Self) -> EditorResult<()>,
    ) -> EditorResult<()> {
        let mut staged = self.clone();
        edit(&mut staged)?;
        staged.validate()?;
        let bytes =
            serde_json::to_vec_pretty(&staged).map_err(|e| EditorError::Gesture(e.to_string()))?;
        write_local(path, &bytes)?;
        *self = staged;
        Ok(())
    }
    /// Add a custom palette or a copy of a preset.
    pub fn create(&mut self, name: String, palette: Palette) -> EditorResult<Uuid> {
        validate_name(&name)?;
        palette.validate()?;
        if self.entries.len() == 128 {
            return Err(EditorError::Gesture("128 custom palettes maximum".into()));
        }
        let id = Uuid::new_v4();
        self.entries.push(LibraryPalette { id, name, palette });
        Ok(id)
    }
    /// Change a custom palette while preserving display order and duplicate colors.
    pub fn set_colors(&mut self, index: usize, colors: Vec<Swatch>) -> EditorResult<()> {
        let palette = Palette::from_colors(colors)?;
        self.entries
            .get_mut(index)
            .ok_or_else(|| EditorError::Gesture("Palette no longer exists".into()))?
            .palette = palette;
        Ok(())
    }
}
/// Parse exactly one opaque hexadecimal swatch.
pub fn parse_color(text: &str) -> EditorResult<Swatch> {
    let palette = Palette::from_hex(text)?;
    if palette.slots.len() != 1 {
        return Err(EditorError::Gesture("Enter one six-digit hex color".into()));
    }
    Ok(palette.slots[0].color.clone())
}
/// Persist browser settings in origin-local storage.
#[cfg(all(feature = "web", target_family = "wasm"))]
pub fn write_local(path: &Path, bytes: &[u8]) -> EditorResult<()> {
    crate::web::store(&path.to_string_lossy(), bytes)
}

/// Read bounded settings bytes, or `None` if no settings have been saved.
pub fn read_local(path: &Path) -> EditorResult<Option<Vec<u8>>> {
    #[cfg(not(target_family = "wasm"))]
    let bytes = {
        if !path.exists() {
            return Ok(None);
        }
        if std::fs::metadata(path)?.len() > 4 * 1024 * 1024 {
            return Err(EditorError::Gesture("Settings file too large".into()));
        }
        Some(std::fs::read(path)?)
    };
    #[cfg(target_family = "wasm")]
    let bytes = crate::web::load(&path.to_string_lossy())?;
    if bytes.as_ref().is_some_and(|b| b.len() > 4 * 1024 * 1024) {
        return Err(EditorError::Gesture("Settings file too large".into()));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atomic_library_roundtrip_and_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("palettes.json");
        let mut lib = PaletteLibrary::default();
        lib.update(&path, |l| {
            l.create("Studio".into(), voxelite_core::Preset::Sweetie16.palette())?;
            Ok(())
        })
        .unwrap();
        let id = lib.entries[0].id;
        assert!(
            lib.update(&path, |l| {
                l.entries[0].name.clear();
                Ok(())
            })
            .is_err()
        );
        assert_eq!(PaletteLibrary::load(&path).unwrap().entries[0].id, id);
        assert_eq!(lib.entries[0].name, "Studio");
        assert!(
            lib.update(&dir.path().join("missing/file/child"), |l| l
                .set_colors(0, vec![]))
                .is_err()
        );
        assert_eq!(lib.entries[0].palette.slots.len(), 16);
    }
    #[test]
    fn duplicate_edit_delete_and_io_failure_are_isolated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("palettes.json");
        let mut lib = PaletteLibrary::default();
        let original = voxelite_core::Preset::Pico8.palette();
        lib.update(&path, |l| {
            l.create("Original".into(), original.clone())?;
            l.create("Copy".into(), original.clone())?;
            Ok(())
        })
        .unwrap();
        let mut colors = vec![Swatch::rgb(1, 2, 3), Swatch::rgb(4, 5, 6)];
        lib.update(&path, |l| l.set_colors(1, colors.clone()))
            .unwrap();
        colors.swap(0, 1);
        lib.update(&path, |l| l.set_colors(1, colors)).unwrap();
        assert_eq!(lib.entries[0].palette, original);
        assert_eq!(lib.entries[1].palette.slots[0].color.rgba, [4, 5, 6, 255]);
        assert!(
            lib.update(dir.path(), |l| {
                l.entries[1].name = "Should not install".into();
                Ok(())
            })
            .is_err()
        );
        assert_eq!(lib.entries[1].name, "Copy");
        lib.update(&path, |l| {
            l.entries.remove(1);
            Ok(())
        })
        .unwrap();
        assert_eq!(PaletteLibrary::load(&path).unwrap().entries.len(), 1);
    }
    #[test]
    fn colors_and_interchange_preserve_duplicates() {
        assert!(parse_color("112233\n445566").is_err());
        assert!(parse_color("#xyz000").is_err());
        let c = parse_color("#A0b1C2").unwrap();
        let p = Palette::from_colors(vec![c.clone(), c]).unwrap();
        assert_eq!(Palette::from_hex(&p.to_hex()).unwrap(), p);
        assert_eq!(Palette::from_gpl(&p.to_gpl("Studio").unwrap()).unwrap(), p);
    }
}
