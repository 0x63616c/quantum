//! CPU-rendered previews of saved documents. All methods perform blocking work;
//! desktop callers run them on a background executor, never on the window thread.
use crate::{
    editor::{EditorError, EditorResult},
    local_settings,
};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};
use voxelite_format::session::fingerprint;

// Bound peak memory when multiple windows request previews of large documents.
static RENDER: Mutex<()> = Mutex::new(());
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const VERSION: u32 = 3;

#[derive(PartialEq, Eq)]
struct Stamp {
    version: u32,
    modified: SystemTime,
    length: u64,
}
impl Stamp {
    fn read(path: &Path) -> EditorResult<Self> {
        let metadata = path.metadata()?;
        Ok(Self {
            version: VERSION,
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
        // SystemTime's serde encoding rejects pre-epoch file mtimes. Encode a
        // sign and magnitude instead; both sides of the epoch are valid inputs.
        let (before_epoch, elapsed) = match stamp.modified.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(elapsed) => (false, elapsed),
            Err(error) => (true, error.duration()),
        };
        let mut key = Vec::with_capacity(25);
        key.extend(stamp.version.to_le_bytes());
        key.extend(stamp.length.to_le_bytes());
        key.push(u8::from(before_epoch));
        key.extend(elapsed.as_secs().to_le_bytes());
        key.extend(elapsed.subsec_nanos().to_le_bytes());
        self.directory
            .join(fingerprint(path.as_os_str().as_encoded_bytes()))
            .join(format!("{}.png", fingerprint(&key)))
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
        let png = render_png(&project.document.content)?;
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
    /// Render bundled artwork using the same pipeline and a content-addressed cache.
    /// Counts are derived from the document, never from presentation fixtures.
    pub fn sample(&self, sample: crate::samples::Sample) -> EditorResult<SamplePreview> {
        let _guard = RENDER.lock().unwrap_or_else(|poison| poison.into_inner());
        let bytes = sample.bytes();
        let destination = self
            .directory
            .join("samples")
            .join(format!("{VERSION}-{}.png", fingerprint(bytes)));
        let project = voxelite_format::from_bytes(bytes)?;
        let content = &project.document.content;
        if !destination.is_file() {
            local_settings::write_local(&destination, &render_png(content)?)?;
        }
        Ok(SamplePreview {
            path: destination,
            voxels: content.composite().len(),
            layers: content.layers.len(),
        })
    }
}

/// Cached sample artwork and verified document metadata.
#[derive(Clone)]
pub struct SamplePreview {
    /// PNG produced by the same renderer as recent projects.
    pub path: PathBuf,
    /// Visible composite voxels, counting overlapping layers once.
    pub voxels: usize,
    /// Number of document layers.
    pub layers: usize,
}

fn render_png(snapshot: &voxelite_core::Snapshot) -> EditorResult<Vec<u8>> {
    let image = voxelite_render::render(
        snapshot,
        &voxelite_render::RenderOptions {
            width: WIDTH,
            height: HEIGHT,
            supersampling: 2,
            thumbnail: true,
            ..Default::default()
        },
    )
    .map_err(|e| EditorError::Gesture(e.to_string()))?;
    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| EditorError::Gesture(e.to_string()))?;
        writer
            .write_image_data(&image.rgba)
            .map_err(|e| EditorError::Gesture(e.to_string()))?;
    }
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::Duration};

    #[test]
    fn pre_epoch_mtime_is_cacheable_and_distinct_from_positive_time() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old.voxl");
        crate::editor::Editor::blank().unwrap().save(&path).unwrap();
        let file = fs::File::options().write(true).open(&path).unwrap();
        let offset = Duration::from_secs(86400);
        file.set_modified(SystemTime::UNIX_EPOCH - offset).unwrap();
        let cache = ThumbnailCache::new(dir.path());
        let first = cache.ensure(&path).unwrap();
        assert_eq!(cache.cached(&path).unwrap(), Some(first.clone()));
        file.set_modified(SystemTime::UNIX_EPOCH + offset).unwrap();
        assert!(cache.cached(&path).unwrap().is_none());
        assert_ne!(cache.ensure(&path).unwrap(), first);
    }

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
        assert_eq!((reader.info().width, reader.info().height), (WIDTH, HEIGHT));
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut pixels).unwrap();
        assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[3] > 0));
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
    fn sample_cache_reuses_artwork_and_counts_overlapping_layers_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ThumbnailCache::new(dir.path());
        let first = cache.sample(crate::samples::Sample::Autumn).unwrap();
        assert_eq!((first.voxels, first.layers), (27_817, 4));
        let modified = first.path.metadata().unwrap().modified().unwrap();
        let second = cache.sample(crate::samples::Sample::Autumn).unwrap();
        assert_eq!(first.path, second.path);
        assert_eq!(
            second.path.metadata().unwrap().modified().unwrap(),
            modified
        );
    }

    #[test]
    fn file_length_invalidates_even_when_mtime_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        crate::editor::Editor::blank().unwrap().save(&path).unwrap();
        let cache = ThumbnailCache::new(dir.path());
        let first = cache.ensure(&path).unwrap();
        let modified = path.metadata().unwrap().modified().unwrap();
        use std::io::Write;
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b" ").unwrap();
        file.set_modified(modified).unwrap();
        assert!(cache.cached(&path).unwrap().is_none());
        assert_ne!(cache.ensure(&path).unwrap(), first);
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
