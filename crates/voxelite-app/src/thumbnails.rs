//! CPU-rendered previews of saved documents. All methods perform blocking work;
//! desktop callers run them on a background executor, never on the window thread.
use crate::{
    editor::{EditorError, EditorResult},
    local_settings,
};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};
use voxelite_format::session::fingerprint;

// Bound peak memory when multiple windows request previews of large documents.
static RENDER: Mutex<()> = Mutex::new(());
const SIZE: u32 = 192;

#[derive(PartialEq, Eq, Serialize)]
struct Stamp {
    version: u32,
    modified: SystemTime,
    length: u64,
}
impl Stamp {
    fn read(path: &Path) -> EditorResult<Self> {
        let metadata = path.metadata()?;
        Ok(Self {
            version: 1,
            modified: metadata.modified()?,
            length: metadata.len(),
        })
    }
}

/// A cache rooted in the application's settings directory.
#[derive(Clone)]
pub struct ThumbnailCache {
    directory: PathBuf,
}
impl ThumbnailCache {
    /// Create a cache handle without touching disk.
    pub fn new(settings_directory: &Path) -> Self {
        Self {
            directory: settings_directory.join("thumbnails"),
        }
    }

    fn destination(&self, path: &Path, stamp: &Stamp) -> PathBuf {
        // Preserve non-Unicode path identity. The second hash also avoids stale
        // GPUI image-cache hits after the project is saved again.
        self.directory
            .join(fingerprint(path.as_os_str().as_encoded_bytes()))
            .join(format!(
                "{}.png",
                fingerprint(&serde_json::to_vec(stamp).expect("serializable timestamp"))
            ))
    }

    /// Return a valid cache entry for the current file revision, without rendering.
    pub fn cached(&self, path: &Path) -> EditorResult<Option<PathBuf>> {
        let path = path.canonicalize()?;
        let destination = self.destination(&path, &Stamp::read(&path)?);
        Ok(destination.is_file().then_some(destination))
    }

    /// Reuse or generate a preview from the saved document, without replaying history.
    /// If the file changes during rendering, discard the result and let a later
    /// request retry. Failed previews never affect opening or saving the project.
    pub fn ensure(&self, path: &Path) -> EditorResult<PathBuf> {
        let _guard = RENDER.lock().unwrap_or_else(|poison| poison.into_inner());
        let path = path.canonicalize()?;
        let stamp = Stamp::read(&path)?;
        let destination = self.destination(&path, &stamp);
        if destination.is_file() {
            return Ok(destination);
        }
        let bytes = voxelite_format::session::read(&path)?;
        let project = voxelite_format::from_bytes(&bytes)?;
        let image = voxelite_render::render(
            &project.document.content,
            &voxelite_render::RenderOptions {
                width: SIZE,
                height: SIZE,
                ..Default::default()
            },
        )
        .map_err(|e| EditorError::Gesture(e.to_string()))?;
        let mut png = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png, SIZE, SIZE);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .map_err(|e| EditorError::Gesture(e.to_string()))?;
            writer
                .write_image_data(&image.rgba)
                .map_err(|e| EditorError::Gesture(e.to_string()))?;
        }
        if Stamp::read(&path)? != stamp {
            return Err(EditorError::Gesture(
                "Project changed while creating its preview".into(),
            ));
        }
        local_settings::write_local(&destination, &png)?;
        // Each project keeps only its current revision. Other paths are untouched.
        if let Some(parent) = destination.parent() {
            for entry in std::fs::read_dir(parent)?.flatten() {
                let old = entry.path();
                if old != destination && old.extension().is_some_and(|ext| ext == "png") {
                    let _ = std::fs::remove_file(old);
                }
            }
        }
        Ok(destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::Duration};

    #[test]
    fn lazy_preview_is_png_reused_until_mtime_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut editor = crate::editor::Editor::blank().unwrap();
        editor.apply_at([2, 2, 2]).unwrap();
        editor.save(&path).unwrap();
        let cache = ThumbnailCache::new(dir.path());
        assert!(cache.cached(&path).unwrap().is_none());
        let first = cache.ensure(&path).unwrap();
        let bytes = fs::read(&first).unwrap();
        let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
        let mut reader = decoder.read_info().unwrap();
        assert_eq!((reader.info().width, reader.info().height), (SIZE, SIZE));
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut pixels).unwrap();
        assert!(pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
        let cached_mtime = first.metadata().unwrap().modified().unwrap();
        assert_eq!(cache.ensure(&path).unwrap(), first);
        assert_eq!(first.metadata().unwrap().modified().unwrap(), cached_mtime);
        let modified = path.metadata().unwrap().modified().unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(modified + Duration::from_secs(1))
            .unwrap();
        assert!(cache.cached(&path).unwrap().is_none());
        let second = cache.ensure(&path).unwrap();
        assert_ne!(first, second);
        assert!(!first.exists());
        assert_eq!(fs::read(&second).unwrap(), bytes);
        editor.apply_at([3, 2, 2]).unwrap();
        editor.save(&path).unwrap();
        let changed = cache.ensure(&path).unwrap();
        assert_ne!(changed, second);
        assert_ne!(fs::read(changed).unwrap(), bytes);
    }

    #[test]
    fn path_hashes_separate_identical_names_and_aliases_share_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ThumbnailCache::new(dir.path());
        for name in ["a", "b"] {
            fs::create_dir(dir.path().join(name)).unwrap();
        }
        let a = dir.path().join("a/project.voxl");
        let b = dir.path().join("b/project.voxl");
        crate::editor::Editor::blank().unwrap().save(&a).unwrap();
        fs::copy(&a, &b).unwrap();
        assert_ne!(
            cache.ensure(&a).unwrap().parent(),
            cache.ensure(&b).unwrap().parent()
        );
        assert_eq!(
            cache.ensure(&a).unwrap(),
            cache.ensure(&dir.path().join("a/./project.voxl")).unwrap()
        );
    }

    #[test]
    fn missing_or_corrupt_projects_never_produce_cache_entries() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ThumbnailCache::new(dir.path());
        let path = dir.path().join("bad.voxl");
        assert!(cache.ensure(&path).is_err());
        fs::write(&path, b"invalid").unwrap();
        assert!(cache.ensure(&path).is_err());
        assert!(cache.cached(&path).unwrap().is_none());
        assert!(!cache.directory.exists());
    }
}
