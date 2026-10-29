//! Bounded host file I/O; the palette plugin owns parsing and serialization.
use super::*;
use quantum_app::editor::EditorError;
use quantum_app::local_settings::data_dir;

const MAX_IMPORT: usize = 64 * 1024;
fn import_action(filename: &str, text: String) -> PaletteAction {
    let path = std::path::Path::new(filename);
    PaletteAction::ImportLibrary {
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported palette")
            .chars()
            .take(100)
            .collect(),
        format: path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase(),
        text,
    }
}
impl EditorView {
    pub(super) fn palette_menu_action(&mut self, index: usize, cx: &mut Context<Self>) {
        let format = if index == 0 {
            None
        } else {
            let Some(format) = self
                .editor
                .palette_view()
                .and_then(|v| v.formats.get(index - 1).copied())
            else {
                return;
            };
            Some(format.id.to_owned())
        };
        self.palettes.menu = false;
        self.palette_file(format, cx);
    }
    fn palette_export_bytes(&mut self, format: String, cx: &mut Context<Self>) -> Option<Vec<u8>> {
        if !self.flush_palette_fields(cx)
            || self
                .manager_request(PaletteAction::CommitColor, cx)
                .is_none()
        {
            return None;
        }
        self.manager_request(PaletteAction::ExportLibrary { format }, cx)?
            .export
            .map(|export| export.text.into_bytes())
    }
    fn palette_file(&mut self, export: Option<String>, cx: &mut Context<Self>) {
        if !self.flush_palette_fields(cx) {
            return;
        }
        let bytes = if let Some(format) = &export {
            let Some(bytes) = self.palette_export_bytes(format.clone(), cx) else {
                return;
            };
            Some(bytes)
        } else {
            None
        };
        self.save_flow.begin_dialog();
        cx.spawn(async move |this, cx| {
            let path: anyhow::Result<Option<PathBuf>> = if let Some(ext) = &export {
                cx.update(|cx| cx.prompt_for_new_path(&data_dir(), Some(&format!("palette.{ext}"))))
                    .await
                    .map_err(anyhow::Error::from)
                    .and_then(|r| r)
            } else {
                cx.update(|cx| {
                    cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: false,
                        prompt: Some("Import HEX, GPL or JASC-PAL palette".into()),
                    })
                })
                .await
                .map_err(anyhow::Error::from)
                .and_then(|r| r)
                .map(|paths| paths.and_then(|paths| paths.into_iter().next()))
            };
            let result = match path {
                Ok(Some(path)) => {
                    cx.background_executor()
                        .spawn(async move {
                            if let Some(bytes) = bytes {
                                quantum_app::local_settings::write_local(&path, &bytes)?;
                                Ok(None)
                            } else {
                                read_import(&path).map(Some)
                            }
                        })
                        .await
                }
                Ok(None) => Ok(None),
                Err(error) => Err(EditorError::PaletteDialog(error)),
            };
            let _ = this.update(cx, |this, cx| {
                this.save_flow.finish_dialog();
                this.finish_palette_import(result, cx);
            });
        })
        .detach();
    }
    pub(super) fn palette_drop_file(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        if self.save_flow.dialog_open() || !self.flush_palette_fields(cx) {
            return;
        }
        if paths.paths().len() != 1 {
            self.palettes.error = Some("Drop one HEX, GPL or JASC-PAL file at a time.".into());
            cx.notify();
            return;
        }
        let path = paths.paths()[0].clone();
        self.save_flow.begin_dialog();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { read_import(&path).map(Some) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.save_flow.finish_dialog();
                this.finish_palette_import(result, cx);
            });
        })
        .detach();
    }
    fn finish_palette_import(
        &mut self,
        result: EditorResult<Option<PaletteAction>>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(Some(action)) => self.manager_action(action, cx),
            Ok(None) => {}
            Err(error) => self.palettes.error = Some(error.to_string()),
        }
        cx.notify();
    }
}
fn read_import(path: &std::path::Path) -> EditorResult<PaletteAction> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take((MAX_IMPORT + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_IMPORT {
        return Err(EditorError::PaletteImportTooLarge);
    }
    let text = String::from_utf8(bytes)?;
    Ok(import_action(&path.to_string_lossy(), text))
}

#[cfg(test)]
mod tests {
    use super::{EditorError, MAX_IMPORT, PaletteAction, read_import};

    #[test]
    fn palette_import_accepts_the_limit_and_rejects_one_extra_byte() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Boundary.HEX");
        let text = "a".repeat(MAX_IMPORT);
        std::fs::write(&path, &text).unwrap();
        let PaletteAction::ImportLibrary {
            name,
            format,
            text: actual,
        } = read_import(&path).unwrap()
        else {
            panic!("expected a palette import proposal");
        };
        assert_eq!(name, "Boundary");
        assert_eq!(format, "hex");
        assert_eq!(actual, text);
        std::fs::write(&path, vec![b'a'; MAX_IMPORT + 1]).unwrap();
        let error = read_import(&path).unwrap_err();
        assert!(matches!(error, EditorError::PaletteImportTooLarge));
        assert_eq!(
            error.to_string(),
            "Palette files must be 64 KiB or smaller."
        );
    }

    #[test]
    fn palette_import_preserves_utf8_and_io_failures() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid.hex");
        assert!(matches!(
            read_import(&path),
            Err(EditorError::Io(cause)) if cause.kind() == std::io::ErrorKind::NotFound
        ));
        std::fs::write(&path, [255]).unwrap();
        let error = read_import(&path).unwrap_err();
        assert!(matches!(&error, EditorError::PaletteUtf8(cause)
            if cause.as_bytes() == [255]));
        assert_eq!(
            error.to_string(),
            String::from_utf8(vec![255]).unwrap_err().to_string()
        );
    }
}
