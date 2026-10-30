//! Browser file operations attached to the shared editor actions.
use super::*;
impl EditorView {
    pub(super) fn open_dialog(&mut self, cx: &mut Context<Self>) {
        let upload =
            match crate::web::upload(".voxl,application/json", voxelite_format::MAX_FILE_BYTES) {
                Ok(upload) => upload,
                Err(error) => {
                    self.report::<()>(Err(error), "", cx);
                    return;
                }
            };
        self.dialog_open = true;
        self.drag = None;
        cx.spawn(async move |this, cx| {
            let result = upload.await.and_then(|selection| {
                selection
                    .map(|(_, bytes)| Editor::new(voxelite_format::read(bytes.as_slice())?, None))
                    .transpose()
            });
            let _ = this.update(cx, |this, cx| {
                this.dialog_open = false;
                match result {
                    Ok(Some(editor)) => {
                        this.replace_editor(editor, cx);
                        this.status = "Project opened".into();
                        this.error = false;
                    }
                    Ok(None) => {}
                    Err(error) => this.report::<()>(Err(error), "", cx),
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub(super) fn save(&mut self, _: bool, cx: &mut Context<Self>) {
        self.choose_save_path(false, cx);
    }
    pub(super) fn choose_save_path(&mut self, copy: bool, cx: &mut Context<Self>) {
        let result = self
            .editor
            .project()
            .to_bytes()
            .map_err(Into::into)
            .and_then(|bytes| {
                crate::web::download(
                    &format!("{}.voxl", self.editor.snapshot().name),
                    "application/json",
                    &bytes,
                )
            });
        let success = result.is_ok();
        self.report(result, "Project download started", cx);
        if success && !copy {
            self.editor.engine.mark_saved();
        }
        let next = self.after_save.take();
        if success && let Some(op) = next {
            self.run(op, cx);
        }
    }
    pub(super) fn export_browser(&mut self, png: bool, cx: &mut Context<Self>) {
        let result: EditorResult<()> = (|| {
            let bytes = if png {
                let frame = voxelite_render::render(
                    self.editor.snapshot(),
                    &RenderOptions {
                        width: 1024,
                        height: 768,
                        view: View::Camera(self.editor.camera.camera()),
                        background: if self.theme == Theme::Dark {
                            [0, 0, 0, 255]
                        } else {
                            [255; 4]
                        },
                        ..Default::default()
                    },
                )
                .map_err(|e| voxelite_app::editor::EditorError::Gesture(e.to_string()))?;
                voxelite_export::png_rgba(frame.width, frame.height, &frame.rgba)
            } else {
                voxelite_export::glb(self.editor.snapshot())
            }
            .map_err(|e| voxelite_app::editor::EditorError::Gesture(e.to_string()))?;
            let (ext, mime) = if png {
                ("png", "image/png")
            } else {
                ("glb", "model/gltf-binary")
            };
            crate::web::download(
                &format!("{}.{}", self.editor.snapshot().name, ext),
                mime,
                &bytes,
            )
        })();
        self.report(result, "Export download started", cx);
    }
}
