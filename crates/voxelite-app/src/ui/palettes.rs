//! Palette manager and native interchange dialogs.
use super::*;
#[cfg(not(target_family = "wasm"))]
use voxelite_app::local_settings::data_dir;
use voxelite_core::Swatch;
use voxelite_plugin_api::{PaletteAction, PaletteOperation, PaletteOutcome, PalettePresentation};
use voxelite_plugin_palette::parse_color;

pub(super) struct PaletteManager {
    pub open: bool,
    pub name: Entity<crate::text_field::TextField>,
    pub hex: Entity<crate::text_field::TextField>,
}
impl PaletteManager {
    pub fn new(cx: &mut App) -> Self {
        let hex = crate::text_field::TextField::new("#RRGGBB", cx);
        hex.update(cx, |field, _| field.numeric(7));
        Self {
            open: false,
            name: crate::text_field::TextField::new("Palette name", cx),
            hex,
        }
    }
}
impl EditorView {
    pub(super) fn reconcile_palette_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editor.palette_enabled() {
            let focused = self.palette_focus.is_focused(window)
                || self.palettes.name.read(cx).focused(window)
                || self.palettes.hex.read(cx).focused(window);
            self.palette_open = false;
            self.palettes.open = false;
            if focused {
                window.focus(&self.focus, cx);
            }
        }
    }
    pub(super) fn palette_name(&self) -> String {
        self.editor
            .palette_view()
            .map(|v| v.name)
            .unwrap_or_default()
    }
    pub(super) fn palette_index(&self) -> usize {
        self.editor
            .palette_view()
            .map(|v| v.preset)
            .unwrap_or_default()
    }
    pub(super) fn palette_choices(&self) -> Vec<voxelite_plugin_api::PaletteChoice> {
        self.editor
            .palette_view()
            .map(|v| v.choices)
            .unwrap_or_default()
    }
    pub(super) fn palette_request(
        &mut self,
        action: PaletteAction,
        cx: &mut Context<Self>,
    ) -> Option<PaletteOutcome> {
        match self.editor.palette_action(action) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.report::<()>(Err(error), "", cx);
                None
            }
        }
    }
    fn sync_palette_fields(&mut self, cx: &mut Context<Self>) {
        let Some(view) = self.editor.palette_view() else {
            return;
        };
        // Clamp the manager cursor when its fields are synchronized, as before.
        self.palette_request(PaletteAction::SelectSlot(view.state.slot), cx);
        let color = view
            .palette
            .get(view.palette.order[view.state.slot])
            .expect("validated palette")
            .rgba;
        self.palettes.name.update(cx, |f, cx| f.set(view.name, cx));
        self.palettes.hex.update(cx, |f, cx| {
            f.set(
                format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]),
                cx,
            )
        });
    }
    pub(super) fn manage_palettes(&mut self, cx: &mut Context<Self>) {
        if self
            .palette_request(
                PaletteAction::Command(Op::ManagePalettes.command_id().into()),
                cx,
            )
            .is_some_and(|out| out.presentation == Some(PalettePresentation::Manage))
        {
            self.palettes.open = true;
            self.sync_palette_fields(cx);
        }
    }
    fn palette_edit(&mut self, operation: PaletteOperation, cx: &mut Context<Self>) {
        let name = self.palettes.name.read(cx).text.clone();
        let color = self.palettes.hex.read(cx).text.clone();
        let result = self.editor.palette_action(PaletteAction::Edit {
            operation,
            name,
            color,
        });
        if result.as_ref().is_ok_and(|out| out.is_some()) {
            self.sync_palette_fields(cx);
        }
        self.report(result, "Palette library saved", cx);
    }
    pub(super) fn control(
        &self,
        id: impl Into<ElementId>,
        name: &'static str,
        text: impl Into<SharedString>,
        cx: &Context<Self>,
        callback: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        let t = self.theme;
        let mut button = Button::new(id, text, t).variant(if name == "send" {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Ghost
        });
        if !name.is_empty() {
            button = button.icon(name);
        }
        button
            .build()
            .border_1()
            .border_color(t.line())
            .on_click(cx.listener(move |this, _, window, cx| {
                window.focus(&this.focus, cx);
                callback(this, window, cx)
            }))
    }
    fn palette_export(
        &self,
        extension: &str,
        palette: Palette,
        name: String,
    ) -> EditorResult<Vec<u8>> {
        if !self.editor.palette_enabled() {
            return Err(voxelite_app::editor::EditorError::Gesture(
                "Palette panel is unavailable".into(),
            ));
        }
        let format = self.editor.export_format(extension)?;
        let job = self.editor.prepare_export(&format.id)?;
        let mut snapshot = self.editor.snapshot().clone();
        snapshot.palette = palette;
        snapshot.name = name;
        self.editor
            .finish_export(job.run(&voxelite_plugin_host::SnapshotExport {
                snapshot: &snapshot,
                options: &Default::default(),
            }))
    }
    #[cfg(not(target_family = "wasm"))]
    fn palette_file(&mut self, export: Option<String>, cx: &mut Context<Self>) {
        self.save_flow.begin_dialog();
        let palette = self.active_palette();
        let name = self.palette_name();
        cx.spawn(async move |this, cx| {
            let path: anyhow::Result<Option<PathBuf>> = if let Some(ext) = &export {
                let receiver = cx.update(|cx| {
                    cx.prompt_for_new_path(&data_dir(), Some(&format!("palette.{ext}")))
                });
                receiver.await.map_err(anyhow::Error::from).and_then(|r| r)
            } else {
                let receiver = cx.update(|cx| {
                    cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: false,
                        prompt: Some("Import .hex or .gpl palette".into()),
                    })
                });
                receiver
                    .await
                    .map_err(anyhow::Error::from)
                    .and_then(|r| r)
                    .map(|p| p.and_then(|p| p.into_iter().next()))
            };
            let result = match path {
                Ok(Some(path)) => {
                    async {
                        let bytes = if let Some(ext) = export {
                            Some(
                                this.update(cx, |this, _| this.palette_export(&ext, palette, name))
                                    .map_err(|error| {
                                        voxelite_app::editor::EditorError::Gesture(
                                            error.to_string(),
                                        )
                                    })??,
                            )
                        } else {
                            None
                        };
                        cx.background_executor()
                            .spawn(async move {
                                if let Some(bytes) = bytes {
                                    voxelite_app::local_settings::write_local(&path, &bytes)?;
                                    Ok(None)
                                } else {
                                    if std::fs::metadata(&path)?.len() > 256 * 1024 {
                                        return Err(voxelite_app::editor::EditorError::Gesture(
                                            "Palette file too large".into(),
                                        ));
                                    }
                                    let text = std::fs::read_to_string(&path)?;
                                    let format = path
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .map(str::to_ascii_lowercase)
                                        .unwrap_or_default();
                                    let name = path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("Imported palette")
                                        .chars()
                                        .take(100)
                                        .collect();
                                    Ok(Some((name, format, text)))
                                }
                            })
                            .await
                    }
                    .await
                }
                Ok(None) => Ok(None),
                Err(e) => Err(voxelite_app::editor::EditorError::Gesture(e.to_string())),
            };
            let _ = this.update(cx, |this, cx| {
                this.save_flow.finish_dialog();
                let result = result.and_then(|import| {
                    if let Some((name, format, text)) = import {
                        this.editor.palette_action(PaletteAction::ImportLibrary {
                            name,
                            format,
                            text,
                        })?;
                        this.sync_palette_fields(cx);
                    }
                    Ok(())
                });
                this.report(result, "Palette file operation complete", cx);
            });
        })
        .detach();
    }
    #[cfg(target_family = "wasm")]
    fn palette_file(&mut self, export: Option<String>, cx: &mut Context<Self>) {
        if let Some(ext) = export {
            let result = self
                .palette_export(&ext, self.active_palette(), self.palette_name())
                .and_then(|bytes| {
                    crate::web::download(&format!("palette.{ext}"), "text/plain", &bytes)
                });
            self.report(result, "Palette download started", cx);
            return;
        }
        let upload = match crate::web::upload(".hex,.gpl", 256 * 1024) {
            Ok(upload) => upload,
            Err(error) => {
                self.report::<()>(Err(error), "", cx);
                return;
            }
        };
        self.save_flow.begin_dialog();
        cx.spawn(async move |this, cx| {
            let result = upload.await.and_then(|selection| {
                selection
                    .map(|(name, bytes)| {
                        let text = std::str::from_utf8(&bytes).map_err(|e| {
                            voxelite_app::editor::EditorError::Gesture(e.to_string())
                        })?;
                        let format = if name.to_ascii_lowercase().ends_with(".gpl") {
                            "gpl"
                        } else {
                            "hex"
                        }
                        .into();
                        let name = name
                            .rsplit_once('.')
                            .map(|(stem, _)| stem)
                            .unwrap_or(&name)
                            .chars()
                            .take(100)
                            .collect::<String>();
                        Ok((name, format, text.to_owned()))
                    })
                    .transpose()
            });
            let _ = this.update(cx, |this, cx| {
                this.save_flow.finish_dialog();
                let result = result.and_then(|selection| {
                    if let Some((name, format, text)) = selection {
                        this.editor.palette_action(PaletteAction::ImportLibrary {
                            name,
                            format,
                            text,
                        })?;
                        this.sync_palette_fields(cx);
                    }
                    Ok(())
                });
                this.report(result, "Palette imported", cx);
            });
        })
        .detach();
    }
    fn color_channel(&self, channel: usize, cx: &Context<Self>) -> Div {
        let color = parse_color(&self.palettes.hex.read(cx).text)
            .unwrap_or_else(|_| Swatch::rgb(128, 128, 128))
            .rgba;
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let paint_bounds = bounds.clone();
        let drag_bounds = bounds.clone();
        let update = move |this: &mut Self,
                           position: Point<Pixels>,
                           b: Bounds<Pixels>,
                           cx: &mut Context<Self>| {
            let mut c = parse_color(&this.palettes.hex.read(cx).text)
                .unwrap_or_else(|_| Swatch::rgb(128, 128, 128))
                .rgba;
            c[channel] = (f32::from(position.x - b.left()) / f32::from(b.size.width).max(1.0)
                * 255.0)
                .clamp(0.0, 255.0)
                .round() as u8;
            this.palettes.hex.update(cx, |f, cx| {
                f.set(format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]), cx)
            });
            cx.notify();
        };
        row()
            .gap(px(8.0))
            .child(mono(["R", "G", "B"][channel]).w(px(16.0)))
            .child(
                div()
                    .id(("color-channel", channel))
                    .relative()
                    .h(px(12.0))
                    .flex_1()
                    .rounded(px(4.0))
                    .overflow_hidden()
                    .cursor_pointer()
                    .child(
                        canvas(
                            move |b, _, _| paint_bounds.set(b),
                            move |b, _, window, _| {
                                for i in 0..64 {
                                    let mut c = color;
                                    c[channel] = (i * 255 / 63) as u8;
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(
                                                b.left() + b.size.width * (i as f32 / 64.0),
                                                b.top(),
                                            ),
                                            size(b.size.width / 64.0 + px(1.0), b.size.height),
                                        ),
                                        color_fill(c),
                                    ));
                                }
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(
                                            b.left()
                                                + b.size.width
                                                    * (f32::from(color[channel]) / 255.0),
                                            b.top(),
                                        ),
                                        size(px(2.0), b.size.height),
                                    ),
                                    rgb(0xffffff),
                                ));
                            },
                        )
                        .size_full(),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                            update(this, e.position, bounds.get(), cx);
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
                        if e.pressed_button == Some(MouseButton::Left) {
                            update(this, e.position, drag_bounds.get(), cx);
                        }
                    })),
            )
            .child(mono(color[channel].to_string()).w(px(28.0)))
    }
    pub(super) fn palette_manager(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        let Some(view) = self.editor.palette_view() else {
            return div().into_any_element();
        };
        let palette = &view.palette;
        let state = &view.state;
        let custom = state.custom.is_some();
        let left = div()
            .w(px(200.0))
            .flex_shrink_0()
            .border_r_1()
            .border_color(t.line())
            .p(px(16.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(section_title("Library"))
            .child(
                div()
                    .id("library-scroll")
                    .overflow_y_scroll()
                    .h(px(350.0))
                    .children(view.choices.iter().enumerate().map(|(i, choice)| {
                        self.control(
                            ("library-preset", i),
                            "palette",
                            choice.name.clone(),
                            cx,
                            move |this, _, cx| {
                                this.palette_request(PaletteAction::ChoosePreset(i), cx);
                                this.sync_palette_fields(cx);
                                cx.notify();
                            },
                        )
                        .w_full()
                        .mb(px(4.0))
                        .when(!custom && i == view.preset, |d| d.bg(t.selected()))
                    }))
                    .children(state.library.entries.iter().enumerate().map(|(i, e)| {
                        let id = e.id;
                        self.control(
                            ("library-custom", i),
                            "palette",
                            e.name.clone(),
                            cx,
                            move |this, _, cx| {
                                this.palette_request(PaletteAction::ChooseCustom(id), cx);
                                this.sync_palette_fields(cx);
                                cx.notify();
                            },
                        )
                        .w_full()
                        .mb(px(4.0))
                        .when(state.custom == Some(id), |d| d.bg(t.selected()))
                    })),
            )
            .child(self.control(
                "create-palette",
                "plus",
                "Create palette",
                cx,
                |this, _, cx| this.palette_edit(PaletteOperation::Create, cx),
            ));
        let right =
            div()
                .flex_1()
                .p(px(24.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(
                    row()
                        .gap(px(8.0))
                        .child(div().flex_1().child(self.palettes.name.clone()))
                        .when(custom, |d| {
                            d.child(self.control(
                                "rename-palette",
                                "place",
                                "Rename",
                                cx,
                                |this, _, cx| this.palette_edit(PaletteOperation::Rename, cx),
                            ))
                        }),
                )
                .child(
                    row()
                        .gap(px(8.0))
                        .child(self.control(
                            "duplicate-palette",
                            "copy",
                            "Duplicate",
                            cx,
                            |this, _, cx| this.palette_edit(PaletteOperation::Duplicate, cx),
                        ))
                        .when(custom, |d| {
                            d.child(
                                self.control(
                                    "delete-palette",
                                    "trash-2",
                                    "Delete",
                                    cx,
                                    |this, _, cx| this.palette_edit(PaletteOperation::Delete, cx),
                                )
                                .text_color(t.danger()),
                            )
                        })
                        .child(
                            div()
                                .text_color(t.muted())
                                .text_size(px(12.0))
                                .child(if custom {
                                    "Custom palette · saved on this device"
                                } else {
                                    "Original palette · duplicate to customize"
                                }),
                        ),
                )
                .child(
                    div()
                        .id("manager-swatches")
                        .max_h(px(152.0))
                        .overflow_y_scroll()
                        .child(div().flex().flex_wrap().gap(px(6.0)).children(
                            palette.order.iter().enumerate().map(|(i, s)| {
                                let c = palette.get(*s).unwrap().rgba;
                                voxelite_ui::Swatch::new(t, color_fill(c))
                                    .selected(i == state.slot)
                                    .build()
                                    .id(("manager-swatch", i))
                                    .role(Role::Button)
                                    .aria_label(format!("Color {}", i + 1))
                                    .aria_selected(i == state.slot)
                                    .tab_index(0)
                                    .focus_visible(move |d| d.shadow(focus_ring(t)))
                                    .cursor_pointer()
                                    .when(i == state.slot, |d| {
                                        d.child(icon("check").text_color(swatch_ink(c)))
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.palette_request(PaletteAction::SelectSlot(i), cx);
                                        this.sync_palette_fields(cx);
                                        cx.notify();
                                    }))
                            }),
                        )),
                )
                .child(
                    row()
                        .gap(px(8.0))
                        .child(div().w(px(160.0)).child(self.palettes.hex.clone()))
                        .when(custom, |d| {
                            d.child(self.control(
                                "apply-color",
                                "check",
                                "Apply color",
                                cx,
                                |this, _, cx| this.palette_edit(PaletteOperation::ApplyColor, cx),
                            ))
                        }),
                )
                .when(custom, |d| {
                    d.child(
                        row().gap(px(6.0)).children(
                            [
                                ("Add slot", PaletteOperation::AddSlot),
                                ("Remove slot", PaletteOperation::RemoveSlot),
                                ("Move left", PaletteOperation::MoveLeft),
                                ("Move right", PaletteOperation::MoveRight),
                            ]
                            .into_iter()
                            .enumerate()
                            .map(|(i, (label, operation))| {
                                self.control(
                                    ("slot-op", i),
                                    ["plus", "trash-2", "arrow-left", "arrow-right"][i],
                                    label,
                                    cx,
                                    move |this, _, cx| this.palette_edit(operation, cx),
                                )
                            }),
                        ),
                    )
                })
                .child(
                    div()
                        .text_color(t.muted())
                        .text_size(px(12.0))
                        .child("Color picker"),
                )
                .child(div().flex().gap(px(3.0)).children((0usize..24).map(|i| {
                    let h = i as f32 / 24.0;
                    let c = Hsla {
                        h,
                        s: 0.65,
                        l: 0.55,
                        a: 1.0,
                    };
                    let rgba: Rgba = c.into();
                    div()
                        .id(("color-choice", i))
                        .flex_1()
                        .h(px(24.0))
                        .rounded(px(3.0))
                        .bg(c)
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.palettes.hex.update(cx, |f, cx| {
                                f.set(
                                    format!(
                                        "#{:02X}{:02X}{:02X}",
                                        (rgba.r * 255.0).round() as u8,
                                        (rgba.g * 255.0).round() as u8,
                                        (rgba.b * 255.0).round() as u8
                                    ),
                                    cx,
                                )
                            });
                            cx.notify();
                        }))
                })))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .children((0..3).map(|i| self.color_channel(i, cx))),
                )
                .child(
                    row()
                        .gap(px(8.0))
                        .child(self.control(
                            "import-palette",
                            "download",
                            "Import…",
                            cx,
                            |this, _, cx| this.palette_file(None, cx),
                        ))
                        .children(
                            self.editor
                                .plugin_host()
                                .registry()
                                .formats()
                                .filter(|(_, format)| {
                                    format.export
                                        && format.content
                                            == voxelite_plugin_api::FormatContent::Palette
                                })
                                .filter_map(|(_, format)| format.extensions.first().cloned())
                                .map(|ext| {
                                    self.control(
                                        SharedString::from(format!("export-{ext}")),
                                        "upload",
                                        format!("Export {}…", ext.to_uppercase()),
                                        cx,
                                        move |this, _, cx| this.palette_file(Some(ext.clone()), cx),
                                    )
                                }),
                        ),
                )
                .child(div().text_size(px(12.0)).text_color(t.muted()).child(
                    "Palette selection applies to new voxels. Existing colors stay intact.",
                ));
        Dialog::scrim(t, false)
            .child(
                Dialog::panel("palette-manager", "Palettes", t, &self.modal_focus)
                    .w(px(780.0))
                    .h(px(620.0))
                    .max_h(relative(0.95))
                    .flex()
                    .flex_col()
                    .p(px(0.0))
                    .child(Dialog::title("Palettes", t).child(self.control(
                        "close-palettes",
                        "check",
                        "Done",
                        cx,
                        |this, window, cx| {
                            this.palettes.open = false;
                            window.focus(&this.focus, cx);
                            cx.notify();
                        },
                    )))
                    .child(
                        row()
                            .flex_1()
                            .min_h(px(0.0))
                            .items_stretch()
                            .child(left)
                            .child(right),
                    ),
            )
            .into_any_element()
    }
}
