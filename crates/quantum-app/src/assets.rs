//! Bundled Lucide vector assets; see assets/icons/LICENSE for attribution.
use gpui::{AssetSource, SharedString};
use std::borrow::Cow;

pub fn fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist.ttf")),
        // Linux's font matcher needs concrete weight faces.
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-SemiBold.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono.ttf")),
    ]
}

pub struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        Ok(RASTER_AND_ICONS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes))
            .or_else(|| {
                crate::shortcuts::ShortcutSymbol::ALL
                    .iter()
                    .find(|symbol| symbol.path() == path)
                    .map(|symbol| Cow::Borrowed(symbol.data()))
            }))
    }
    fn list(&self, _path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

const RASTER_AND_ICONS: &[(&str, &[u8])] = &[
    (
        "welcome-mark.png",
        include_bytes!("../assets/welcome-mark.png"),
    ),
    (
        "app-icon-256.png",
        include_bytes!("../assets/app-icon-256.png"),
    ),
    (
        "icons/arrow-left.svg",
        include_bytes!("../assets/icons/arrow-left.svg"),
    ),
    (
        "icons/arrow-right.svg",
        include_bytes!("../assets/icons/arrow-right.svg"),
    ),
    (
        "icons/arrow-up.svg",
        include_bytes!("../assets/icons/arrow-up.svg"),
    ),
    ("icons/box.svg", include_bytes!("../assets/icons/box.svg")),
    (
        "icons/camera.svg",
        include_bytes!("../assets/icons/camera.svg"),
    ),
    (
        "icons/check.svg",
        include_bytes!("../assets/icons/check.svg"),
    ),
    (
        "icons/check-check.svg",
        include_bytes!("../assets/icons/check-check.svg"),
    ),
    (
        "icons/chevron.svg",
        include_bytes!("../assets/icons/chevron.svg"),
    ),
    (
        "icons/chevron-right.svg",
        include_bytes!("../assets/icons/chevron-right.svg"),
    ),
    (
        "icons/chevron-left.svg",
        include_bytes!("../assets/icons/chevron-left.svg"),
    ),
    (
        "icons/circle.svg",
        include_bytes!("../assets/icons/circle.svg"),
    ),
    (
        "icons/close.svg",
        include_bytes!("../assets/icons/close.svg"),
    ),
    (
        "icons/command.svg",
        include_bytes!("../assets/icons/command.svg"),
    ),
    (
        "icons/comment.svg",
        include_bytes!("../assets/icons/comment.svg"),
    ),
    ("icons/copy.svg", include_bytes!("../assets/icons/copy.svg")),
    ("icons/cube.svg", include_bytes!("../assets/icons/cube.svg")),
    (
        "icons/diamond.svg",
        include_bytes!("../assets/icons/diamond.svg"),
    ),
    (
        "icons/download.svg",
        include_bytes!("../assets/icons/download.svg"),
    ),
    (
        "icons/ellipsis.svg",
        include_bytes!("../assets/icons/ellipsis.svg"),
    ),
    (
        "icons/erase.svg",
        include_bytes!("../assets/icons/erase.svg"),
    ),
    ("icons/eye.svg", include_bytes!("../assets/icons/eye.svg")),
    (
        "icons/eye-off.svg",
        include_bytes!("../assets/icons/eye-off.svg"),
    ),
    (
        "icons/file-output.svg",
        include_bytes!("../assets/icons/file-output.svg"),
    ),
    (
        "icons/file-plus.svg",
        include_bytes!("../assets/icons/file-plus.svg"),
    ),
    ("icons/fill.svg", include_bytes!("../assets/icons/fill.svg")),
    ("icons/fit.svg", include_bytes!("../assets/icons/fit.svg")),
    (
        "icons/flip-horizontal-2.svg",
        include_bytes!("../assets/icons/flip-horizontal-2.svg"),
    ),
    (
        "icons/flip-vertical-2.svg",
        include_bytes!("../assets/icons/flip-vertical-2.svg"),
    ),
    (
        "icons/folder-open.svg",
        include_bytes!("../assets/icons/folder-open.svg"),
    ),
    (
        "icons/git-fork.svg",
        include_bytes!("../assets/icons/git-fork.svg"),
    ),
    ("icons/grid.svg", include_bytes!("../assets/icons/grid.svg")),
    (
        "icons/history.svg",
        include_bytes!("../assets/icons/history.svg"),
    ),
    (
        "icons/keyboard.svg",
        include_bytes!("../assets/icons/keyboard.svg"),
    ),
    (
        "icons/layer.svg",
        include_bytes!("../assets/icons/layer.svg"),
    ),
    ("icons/link.svg", include_bytes!("../assets/icons/link.svg")),
    ("icons/lock.svg", include_bytes!("../assets/icons/lock.svg")),
    (
        "icons/log-out.svg",
        include_bytes!("../assets/icons/log-out.svg"),
    ),
    ("icons/moon.svg", include_bytes!("../assets/icons/moon.svg")),
    (
        "icons/mouse.svg",
        include_bytes!("../assets/icons/mouse.svg"),
    ),
    (
        "icons/mouse-pointer-2.svg",
        include_bytes!("../assets/icons/mouse-pointer-2.svg"),
    ),
    (
        "icons/mouse-pointer-click.svg",
        include_bytes!("../assets/icons/mouse-pointer-click.svg"),
    ),
    ("icons/move.svg", include_bytes!("../assets/icons/move.svg")),
    (
        "icons/orbit.svg",
        include_bytes!("../assets/icons/orbit.svg"),
    ),
    (
        "icons/paint.svg",
        include_bytes!("../assets/icons/paint.svg"),
    ),
    (
        "icons/palette.svg",
        include_bytes!("../assets/icons/palette.svg"),
    ),
    (
        "icons/panel-left.svg",
        include_bytes!("../assets/icons/panel-left.svg"),
    ),
    (
        "icons/panel-top.svg",
        include_bytes!("../assets/icons/panel-top.svg"),
    ),
    (
        "icons/pause.svg",
        include_bytes!("../assets/icons/pause.svg"),
    ),
    (
        "icons/picker.svg",
        include_bytes!("../assets/icons/picker.svg"),
    ),
    (
        "icons/place.svg",
        include_bytes!("../assets/icons/place.svg"),
    ),
    ("icons/play.svg", include_bytes!("../assets/icons/play.svg")),
    ("icons/plug.svg", include_bytes!("../assets/icons/plug.svg")),
    ("icons/plus.svg", include_bytes!("../assets/icons/plus.svg")),
    (
        "icons/puzzle.svg",
        include_bytes!("../assets/icons/puzzle.svg"),
    ),
    ("icons/redo.svg", include_bytes!("../assets/icons/redo.svg")),
    (
        "icons/rotate-ccw.svg",
        include_bytes!("../assets/icons/rotate-ccw.svg"),
    ),
    ("icons/save.svg", include_bytes!("../assets/icons/save.svg")),
    (
        "icons/search.svg",
        include_bytes!("../assets/icons/search.svg"),
    ),
    (
        "icons/settings.svg",
        include_bytes!("../assets/icons/settings.svg"),
    ),
    ("icons/send.svg", include_bytes!("../assets/icons/send.svg")),
    (
        "icons/sliders-horizontal.svg",
        include_bytes!("../assets/icons/sliders-horizontal.svg"),
    ),
    (
        "icons/spark.svg",
        include_bytes!("../assets/icons/spark.svg"),
    ),
    (
        "icons/square.svg",
        include_bytes!("../assets/icons/square.svg"),
    ),
    ("icons/sun.svg", include_bytes!("../assets/icons/sun.svg")),
    (
        "icons/trash-2.svg",
        include_bytes!("../assets/icons/trash-2.svg"),
    ),
    ("icons/undo.svg", include_bytes!("../assets/icons/undo.svg")),
    (
        "icons/unlock.svg",
        include_bytes!("../assets/icons/unlock.svg"),
    ),
    (
        "icons/upload.svg",
        include_bytes!("../assets/icons/upload.svg"),
    ),
    ("icons/user.svg", include_bytes!("../assets/icons/user.svg")),
    (
        "icons/users.svg",
        include_bytes!("../assets/icons/users.svg"),
    ),
    ("icons/view.svg", include_bytes!("../assets/icons/view.svg")),
    (
        "icons/wrench.svg",
        include_bytes!("../assets/icons/wrench.svg"),
    ),
    (
        "icons/zoom-in.svg",
        include_bytes!("../assets/icons/zoom-in.svg"),
    ),
];
