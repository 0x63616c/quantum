//! Bundled vector UI assets.
use gpui::{AssetSource, SharedString};
use std::borrow::Cow;
pub struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        Ok(match path {
            "icons/cube.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/cube.svg"))),
            "icons/place.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/place.svg"))),
            "icons/erase.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/erase.svg"))),
            "icons/paint.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/paint.svg"))),
            "icons/fill.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/fill.svg"))),
            "icons/box.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/box.svg"))),
            "icons/picker.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/picker.svg"))),
            "icons/undo.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/undo.svg"))),
            "icons/redo.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/redo.svg"))),
            "icons/fit.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/fit.svg"))),
            "icons/sun.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/sun.svg"))),
            "icons/moon.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/moon.svg"))),
            "icons/layer.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/layer.svg"))),
            "icons/eye.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/eye.svg"))),
            "icons/eye-off.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/eye-off.svg")))
            }
            "icons/lock.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/lock.svg"))),
            "icons/unlock.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/unlock.svg"))),
            "icons/plus.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/plus.svg"))),
            "icons/chevron.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/chevron.svg")))
            }
            "icons/search.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/search.svg"))),
            "icons/check.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/check.svg"))),
            "icons/close.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/close.svg"))),
            "icons/history.svg" => {
                Some(Cow::Borrowed(include_bytes!("../assets/icons/history.svg")))
            }
            "icons/spark.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/spark.svg"))),
            "icons/save.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/save.svg"))),
            "icons/grid.svg" => Some(Cow::Borrowed(include_bytes!("../assets/icons/grid.svg"))),
            _ => None,
        })
    }
    fn list(&self, _path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
