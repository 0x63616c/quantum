use gpui::{AssetSource, SharedString};
use std::borrow::Cow;

pub struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../quantum-app/assets")
            .join(path);
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    fn list(&self, _: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
pub fn fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(include_bytes!(
            "../../../quantum-app/assets/fonts/Geist.ttf"
        )),
        Cow::Borrowed(include_bytes!(
            "../../../quantum-app/assets/fonts/Geist-Medium.ttf"
        )),
        Cow::Borrowed(include_bytes!(
            "../../../quantum-app/assets/fonts/Geist-SemiBold.ttf"
        )),
        Cow::Borrowed(include_bytes!(
            "../../../quantum-app/assets/fonts/GeistMono.ttf"
        )),
    ]
}
