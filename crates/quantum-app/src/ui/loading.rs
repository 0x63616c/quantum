//! Prepared preview pixels and ASCII grids. Disk I/O, resizing and sampling run
//! with the existing background jobs; render only clones shared handles.
use super::*;
use quantum_ui::{
    AsciiImageReveal, AsciiSkeleton,
    ascii_reveal::media::{MAX_MEDIA_EDGE, Media, MediaOptions},
};
use std::time::Instant;

pub(super) struct PreparedPreview {
    image: Arc<RenderImage>,
    media: [Media; 2],
}

impl PreparedPreview {
    pub fn new(
        mut rgba: image::RgbaImage,
        size: [u32; 2],
        backdrop: fn(Theme) -> Hsla,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            size.iter().all(|edge| (1..=MAX_MEDIA_EDGE).contains(edge))
                && rgba.width() > 0
                && rgba.height() > 0,
            "Invalid preview dimensions"
        );
        // Fit as the existing Contain image did, including transparent letterboxing.
        let scale =
            (size[0] as f32 / rgba.width() as f32).min(size[1] as f32 / rgba.height() as f32);
        let fitted = image::imageops::resize(
            &rgba,
            (rgba.width() as f32 * scale).round().max(1.) as u32,
            (rgba.height() as f32 * scale).round().max(1.) as u32,
            image::imageops::FilterType::Triangle,
        );
        let mut sample = image::RgbaImage::new(size[0], size[1]);
        image::imageops::overlay(
            &mut sample,
            &fitted,
            i64::from((size[0] - fitted.width()) / 2),
            i64::from((size[1] - fitted.height()) / 2),
        );
        // Prepare both small grids off-thread against the caller's actual surface
        // so changing appearance during a load needs no resampling in render.
        let media = [Theme::Dark, Theme::Light].map(|theme| {
            let color: Rgba = backdrop(theme).into();
            Media::new(
                size[0],
                size[1],
                sample.as_raw().as_chunks::<4>().0,
                MediaOptions {
                    background: [color.r, color.g, color.b].map(|c| (c * 255.).round() as u8),
                    ..Default::default()
                },
            )
        });
        let [dark, light] = media;
        let media = [dark?, light?];
        for pixel in rgba.as_mut().as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
        Ok(Self {
            image: Arc::new(RenderImage::new(vec![image::Frame::new(rgba)])),
            media,
        })
    }

    pub fn ready(self) -> Preview {
        Preview {
            image: self.image,
            media: self.media.map(Rc::new),
            arrived: Instant::now(),
        }
    }
}

pub(super) struct Preview {
    pub image: Arc<RenderImage>,
    media: [Rc<Media>; 2],
    arrived: Instant,
}

impl Preview {
    pub fn reveal(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        theme: Theme,
        reduced: bool,
    ) -> impl IntoElement {
        let [width, height] = self.media[0].size();
        self.reveal_with(
            id,
            label,
            theme,
            reduced,
            img(self.image.clone())
                .w(px(width as f32))
                .h(px(height as f32))
                .object_fit(ObjectFit::Contain),
        )
    }

    fn reveal_with(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        theme: Theme,
        reduced: bool,
        image: impl IntoElement,
    ) -> impl IntoElement {
        let media = &self.media[usize::from(theme == Theme::Light)];
        let reveal = AsciiImageReveal::new(id, media.clone(), image, label, theme)
            .started_at(self.arrived)
            .reduced_motion(reduced);
        #[cfg(feature = "ui-snapshots")]
        let reveal = reveal.elapsed(std::time::Duration::from_secs(60));
        reveal
    }
}

/// Read only our bounded RGBA thumbnail format; a broken cache entry becomes a
/// static fallback, never a forever-loading skeleton or an unbounded allocation.
pub(super) fn recent(path: &std::path::Path) -> anyhow::Result<PreparedPreview> {
    let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path)?));
    let mut reader = decoder.read_info()?;
    let info = reader.info();
    anyhow::ensure!(
        (1..=MAX_MEDIA_EDGE).contains(&info.width)
            && (1..=MAX_MEDIA_EDGE).contains(&info.height)
            && info.color_type == png::ColorType::Rgba
            && info.bit_depth == png::BitDepth::Eight,
        "Invalid cached preview"
    );
    let mut bytes = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or_else(|| anyhow::anyhow!("Preview too large"))?
    ];
    let info = reader.next_frame(&mut bytes)?;
    bytes.truncate(info.buffer_size());
    let rgba = image::RgbaImage::from_raw(info.width, info.height, bytes)
        .ok_or_else(|| anyhow::anyhow!("Invalid preview pixels"))?;
    PreparedPreview::new(
        rgba,
        [
            metrics::welcome::THUMB_WIDTH as u32,
            metrics::welcome::THUMB_HEIGHT as u32,
        ],
        |theme| theme.background().blend(theme.hover().opacity(0.5)),
    )
}

pub(super) fn recent_element(
    preview: Option<&Option<Preview>>,
    index: usize,
    label: String,
    theme: Theme,
    reduced: bool,
) -> AnyElement {
    match preview {
        Some(Some(preview)) => preview
            .reveal_with(
                ("recent-image", index),
                label,
                theme,
                reduced,
                div()
                    .relative()
                    .w(px(metrics::welcome::THUMB_WIDTH))
                    .h(px(metrics::welcome::THUMB_HEIGHT))
                    .child(
                        img(preview.image.clone())
                            .absolute()
                            .left(relative(-0.06))
                            .top(px(-metrics::welcome::THUMB_HEIGHT * 0.06))
                            .w(relative(1.12))
                            .h(px(metrics::welcome::THUMB_HEIGHT * 1.12))
                            .object_fit(ObjectFit::Contain),
                    ),
            )
            .into_any_element(),
        Some(None) => icon("cube").text_color(theme.muted()).into_any_element(),
        None => div()
            .w(px(metrics::welcome::THUMB_WIDTH))
            .h(px(metrics::welcome::THUMB_HEIGHT))
            .overflow_hidden()
            .child(
                AsciiSkeleton::new(("recent-loading", index), theme)
                    .rows(6)
                    .seed(index as i32)
                    .reduced_motion(reduced),
            )
            .into_any_element(),
    }
}

impl EditorView {
    pub(super) fn scene_loading(&self) -> Div {
        let t = self.theme;
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div().w(px(metrics::SCENE_LOADING_WIDTH)).child(
                    quantum_ui::AsciiSkeleton::new("scene-loading", t)
                        .rows(12)
                        .reduced_motion(self.reduced_motion),
                ),
            )
    }
    pub(super) fn scene_heading(&self) -> Div {
        let t = self.theme;
        let heading = quantum_ui::AsciiReveal::new(
            "scene-view",
            match self.camera_view {
                Some(Op::Front) => "Front",
                Some(Op::Right) => "Right",
                Some(Op::Top) => "Top",
                Some(Op::Frame) => "Isometric",
                _ => "Orthographic",
            },
            t,
        )
        .loading(!self.viewport.has_frame() && self.viewport.rendering())
        .started_at(self.viewport.reveal_started())
        .reduced_motion(self.reduced_motion);
        #[cfg(feature = "ui-snapshots")]
        let heading = heading.elapsed(std::time::Duration::from_secs(60));
        div()
            .absolute()
            .top(px(space::PANEL))
            .left(px(space::XXL))
            .child(
                row()
                    .gap(px(space::INSET))
                    .text_color(t.muted())
                    .child(heading),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_MEDIA_EDGE, PreparedPreview, Theme, recent};
    use gpui::Rgba;

    #[test]
    fn preview_validates_dimensions_before_resizing_and_samples_both_backdrops() {
        assert!(
            PreparedPreview::new(image::RgbaImage::new(0, 1), [64; 2], Theme::surface).is_err()
        );
        let rgba = image::RgbaImage::from_pixel(16, 8, image::Rgba([255, 0, 0, 255]));
        assert!(PreparedPreview::new(rgba.clone(), [0, 64], Theme::surface).is_err());
        assert!(
            PreparedPreview::new(rgba.clone(), [MAX_MEDIA_EDGE + 1, 64], Theme::surface).is_err()
        );
        let preview = PreparedPreview::new(rgba, [64; 2], Theme::surface)
            .unwrap()
            .ready();
        for (media, theme) in preview.media.iter().zip([Theme::Dark, Theme::Light]) {
            assert_eq!(media.size(), [64; 2]);
            let color: Rgba = theme.surface().into();
            assert_eq!(
                media.options().background,
                [color.r, color.g, color.b].map(|c| (c * 255.).round() as u8)
            );
            let frame = media.frame(
                media.duration_ms() * 0.5,
                quantum_ui::ascii_reveal::core::Direction::Reveal,
            );
            let cells: Vec<_> = frame.cells().collect();
            assert!(!cells.is_empty());
            assert!(cells.iter().all(|c| (10. ..54.).contains(&c.y)));
        }
        assert_eq!(&preview.image.as_bytes(0).unwrap()[..4], &[0, 0, 255, 255]);
        assert_ne!(
            preview.media[0].options().background,
            preview.media[1].options().background
        );
    }

    #[test]
    fn malformed_cached_preview_is_an_error() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"not a PNG").unwrap();
        assert!(recent(file.path()).is_err());
    }
    #[test]
    fn welcome_cache_pixels_fit_the_split_layout_in_both_themes() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut encoder = png::Encoder::new(file.reopen().unwrap(), 640, 480);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&vec![0; 640 * 480 * 4])
            .unwrap();
        let preview = recent(file.path()).unwrap().ready();
        for (media, theme) in preview.media.iter().zip([Theme::Dark, Theme::Light]) {
            assert_eq!(media.size(), [60, 46]);
            let color: Rgba = theme.background().blend(theme.hover().opacity(0.5)).into();
            assert_eq!(
                media.options().background,
                [color.r, color.g, color.b].map(|value| (value * 255.).round() as u8)
            );
        }
        assert_eq!(preview.image.as_bytes(0).unwrap().len(), 640 * 480 * 4);
    }
}
