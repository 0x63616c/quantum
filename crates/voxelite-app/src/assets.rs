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
        Ok(match path {
            "welcome-mark.png" => Some(Cow::Borrowed(include_bytes!("../assets/welcome-mark.png"))),
            "app-icon-256.png" => Some(Cow::Borrowed(include_bytes!("../assets/app-icon-256.png"))),
            "icons/arrow-left.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/arrow-left.svg"
            ))),
            "icons/arrow-right.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/arrow-right.svg"
            ))),
            "icons/arrow-up.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/arrow-up.svg"
            ))),
            "icons/box.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/box.svg"))),
            "icons/camera.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/camera.svg"))),
            "icons/check.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/check.svg"))),
            "icons/check-check.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/check-check.svg"
            ))),
            "icons/chevron.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/chevron.svg")))
            }
            "icons/chevron-right.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/chevron-right.svg"
            ))),
            "icons/chevron-left.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/chevron-left.svg"
            ))),
            "icons/circle.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/circle.svg"))),
            "icons/close.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/close.svg"))),
            "icons/command.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/command.svg")))
            }
            "icons/comment.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/comment.svg")))
            }
            "icons/copy.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/copy.svg"))),
            "icons/cube.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/cube.svg"))),
            "icons/diamond.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/diamond.svg")))
            }
            "icons/download.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/download.svg"
            ))),
            "icons/ellipsis.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/ellipsis.svg"
            ))),
            "icons/erase.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/erase.svg"))),
            "icons/eye.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/eye.svg"))),
            "icons/eye-off.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/eye-off.svg")))
            }
            "icons/file-output.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/file-output.svg"
            ))),
            "icons/file-plus.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/file-plus.svg"
            ))),
            "icons/fill.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/fill.svg"))),
            "icons/fit.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/fit.svg"))),
            "icons/flip-horizontal-2.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/flip-horizontal-2.svg"
            ))),
            "icons/flip-vertical-2.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/flip-vertical-2.svg"
            ))),
            "icons/folder-open.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/folder-open.svg"
            ))),
            "icons/git-fork.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/git-fork.svg"
            ))),
            "icons/grid.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/grid.svg"))),
            "icons/history.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/history.svg")))
            }
            "icons/keyboard.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/keyboard.svg"
            ))),
            "icons/layer.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/layer.svg"))),
            "icons/link.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/link.svg"))),
            "icons/lock.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/lock.svg"))),
            "icons/log-out.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/log-out.svg")))
            }
            "icons/moon.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/moon.svg"))),
            "icons/mouse.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/mouse.svg"))),
            "icons/mouse-pointer-2.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/mouse-pointer-2.svg"
            ))),
            "icons/mouse-pointer-click.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/mouse-pointer-click.svg"
            ))),
            "icons/move.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/move.svg"))),
            "icons/orbit.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/orbit.svg"))),
            "icons/paint.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/paint.svg"))),
            "icons/palette.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/palette.svg")))
            }
            "icons/panel-left.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/panel-left.svg"
            ))),
            "icons/panel-top.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/panel-top.svg"
            ))),
            "icons/pause.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/pause.svg"))),
            "icons/picker.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/picker.svg"))),
            "icons/place.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/place.svg"))),
            "icons/play.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/play.svg"))),
            "icons/plug.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/plug.svg"))),
            "icons/plus.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/plus.svg"))),
            "icons/puzzle.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/puzzle.svg"))),
            "icons/redo.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/redo.svg"))),
            "icons/rotate-ccw.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/rotate-ccw.svg"
            ))),
            "icons/save.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/save.svg"))),
            "icons/search.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/search.svg"))),
            "icons/settings.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/settings.svg"
            ))),
            "icons/send.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/send.svg"))),
            "icons/sliders-horizontal.svg" => Some(Cow::Borrowed(include_bytes!(
                "../assets/icons/sliders-horizontal.svg"
            ))),
            "icons/spark.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/spark.svg"))),
            "icons/square.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/square.svg"))),
            "icons/sun.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/sun.svg"))),
            "icons/trash-2.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/trash-2.svg")))
            }
            "icons/undo.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/undo.svg"))),
            "icons/unlock.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/unlock.svg"))),
            "icons/upload.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/upload.svg"))),
            "icons/user.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/user.svg"))),
            "icons/users.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/users.svg"))),
            "icons/view.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/view.svg"))),
            "icons/wrench.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/wrench.svg"))),
            "icons/zoom-in.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/zoom-in.svg")))
            }
            _ => crate::shortcuts::ShortcutSymbol::ALL
                .iter()
                .find(|symbol| symbol.path() == path)
                .map(|symbol| Cow::Borrowed(symbol.data())),
        })
    }
    fn list(&self, _path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
