//! Palette manager and native interchange dialogs.
use super::*;
use voxelite_app::palettes::{PaletteLibrary, data_dir, parse_color};
use voxelite_core::{Swatch, Uuid};

pub(super) struct PaletteManager {
    pub library: PaletteLibrary,
    pub custom: Option<Uuid>,
    pub open: bool,
    pub slot: usize,
    pub name: Entity<crate::text_field::TextField>,
    pub hex: Entity<crate::text_field::TextField>,
}
impl PaletteManager {
    pub fn new(cx: &mut App) -> Self {
        Self {
            library: PaletteLibrary::default(),
            custom: None,
            open: false,
            slot: 0,
            name: crate::text_field::TextField::new("Palette name", cx),
            hex: crate::text_field::TextField::new("#RRGGBB", cx),
        }
    }
}
impl EditorView {
    pub(super) fn palette_name(&self) -> String {
        self.palettes
            .custom
            .and_then(|id| self.palettes.library.entries.iter().find(|e| e.id == id))
            .map(|e| e.name.clone())
            .unwrap_or_else(|| PALETTES[self.palette_index].0.into())
    }
    fn sync_palette_fields(&mut self, cx: &mut Context<Self>) {
        let p = self.active_palette();
        self.palettes.slot = self.palettes.slot.min(p.order.len() - 1);
        let color = p
            .get(p.order[self.palettes.slot])
            .expect("validated palette")
            .rgba;
        let name = self.palette_name();
        self.palettes.name.update(cx, |f, cx| f.set(name, cx));
        self.palettes.hex.update(cx, |f, cx| {
            f.set(
                format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]),
                cx,
            )
        });
    }
    pub(super) fn manage_palettes(&mut self, cx: &mut Context<Self>) {
        self.palettes.open = true;
        self.sync_palette_fields(cx);
    }
    fn palette_edit(&mut self, operation: &'static str, cx: &mut Context<Self>) {
        let current = self.active_palette();
        let name = self.palettes.name.read(cx).text.clone();
        let color = self.palettes.hex.read(cx).text.clone();
        let slot = self.palettes.slot;
        let index = self.palettes.custom.and_then(|id| {
            self.palettes
                .library
                .entries
                .iter()
                .position(|p| p.id == id)
        });
        let mut selected = self.palettes.custom;
        let result = self
            .palettes
            .library
            .update(&data_dir().join("palettes.json"), |library| {
                if matches!(operation, "Create" | "Duplicate") {
                    selected = Some(library.create(
                        if operation == "Create" {
                            "Untitled palette".into()
                        } else {
                            format!("{} copy", name.chars().take(100).collect::<String>())
                        },
                        if operation == "Create" {
                            Palette::default()
                        } else {
                            current.clone()
                        },
                    )?);
                    return Ok(());
                }
                let i = index.ok_or_else(|| {
                    voxelite_app::editor::EditorError::Gesture(
                        "Duplicate this palette to customize it".into(),
                    )
                })?;
                if operation == "Delete" {
                    library.entries.remove(i);
                    selected = None;
                    return Ok(());
                }
                if operation == "Rename" {
                    library.entries[i].name = name;
                    return Ok(());
                }
                let mut colors: Vec<Swatch> = current
                    .order
                    .iter()
                    .map(|s| current.get(*s).unwrap().clone())
                    .collect();
                match operation {
                    "Apply color" => colors[slot] = parse_color(&color)?,
                    "Add slot" => colors.push(parse_color(&color)?),
                    "Remove slot" => {
                        colors.remove(slot);
                    }
                    "Move left" if slot > 0 => colors.swap(slot, slot - 1),
                    "Move right" if slot + 1 < colors.len() => colors.swap(slot, slot + 1),
                    _ => {}
                }
                library.set_colors(i, colors)
            });
        if result.is_ok() {
            self.palettes.custom = selected;
            self.sync_palette_fields(cx);
        }
        self.report(result, "Palette library saved", cx);
    }
    pub(super) fn control(
        &self,
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        cx: &Context<Self>,
        callback: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        let t = self.theme;
        row()
            .id(id)
            .h(px(34.0))
            .px(px(10.0))
            .gap(px(6.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(t.line())
            .cursor_pointer()
            .hover(move |d| d.bg(t.hover()))
            .child(text.into())
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, window, cx| callback(this, window, cx)))
    }
    fn palette_file(&mut self, export: Option<&'static str>, cx: &mut Context<Self>) {
        self.dialog_open = true;
        let palette = self.active_palette();
        let name = self.palette_name();
        cx.spawn(async move |this, cx| {
            let path: anyhow::Result<Option<PathBuf>> = if let Some(ext) = export {
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
                    cx.background_executor()
                        .spawn(async move {
                            if let Some(ext) = export {
                                let text = if ext == "gpl" {
                                    palette.to_gpl(&name)?
                                } else {
                                    palette.to_hex()
                                };
                                voxelite_app::palettes::write_local(&path, text.as_bytes())?;
                                Ok(None)
                            } else {
                                if std::fs::metadata(&path)?.len() > 256 * 1024 {
                                    return Err(voxelite_app::editor::EditorError::Gesture(
                                        "Palette file too large".into(),
                                    ));
                                }
                                let text = std::fs::read_to_string(&path)?;
                                let p = match path
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .map(str::to_ascii_lowercase)
                                    .as_deref()
                                {
                                    Some("gpl") => Palette::from_gpl(&text)?,
                                    Some("hex") => Palette::from_hex(&text)?,
                                    _ => {
                                        return Err(voxelite_app::editor::EditorError::Gesture(
                                            "Choose a .hex or .gpl palette".into(),
                                        ));
                                    }
                                };
                                let name = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("Imported palette")
                                    .chars()
                                    .take(100)
                                    .collect();
                                Ok(Some((name, p)))
                            }
                        })
                        .await
                }
                Ok(None) => Ok(None),
                Err(e) => Err(voxelite_app::editor::EditorError::Gesture(e.to_string())),
            };
            let _ = this.update(cx, |this, cx| {
                this.dialog_open = false;
                let result = result.and_then(|import| {
                    if let Some((name, palette)) = import {
                        let mut id = None;
                        this.palettes
                            .library
                            .update(&data_dir().join("palettes.json"), |lib| {
                                id = Some(lib.create(name, palette)?);
                                Ok(())
                            })?;
                        this.palettes.custom = id;
                        this.sync_palette_fields(cx);
                    }
                    Ok(())
                });
                this.report(result, "Palette file operation complete", cx);
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
        let palette = self.active_palette();
        let custom = self.palettes.custom.is_some();
        let left =
            div()
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
                        .children(PALETTES.iter().enumerate().map(|(i, (name, _))| {
                            self.control(("library-preset", i), *name, cx, move |this, _, cx| {
                                this.palette_index = i;
                                this.palettes.custom = None;
                                this.sync_palette_fields(cx);
                                cx.notify();
                            })
                            .w_full()
                            .mb(px(4.0))
                            .when(!custom && i == self.palette_index, |d| d.bg(t.selected()))
                        }))
                        .children(self.palettes.library.entries.iter().enumerate().map(
                            |(i, e)| {
                                let id = e.id;
                                self.control(
                                    ("library-custom", i),
                                    e.name.clone(),
                                    cx,
                                    move |this, _, cx| {
                                        this.palettes.custom = Some(id);
                                        this.sync_palette_fields(cx);
                                        cx.notify();
                                    },
                                )
                                .w_full()
                                .mb(px(4.0))
                                .when(self.palettes.custom == Some(id), |d| d.bg(t.selected()))
                            },
                        )),
                )
                .child(
                    self.control("create-palette", "+ Create palette", cx, |this, _, cx| {
                        this.palette_edit("Create", cx)
                    }),
                );
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
                            d.child(self.control("rename-palette", "Rename", cx, |this, _, cx| {
                                this.palette_edit("Rename", cx)
                            }))
                        }),
                )
                .child(
                    row()
                        .gap(px(8.0))
                        .child(
                            self.control("duplicate-palette", "Duplicate", cx, |this, _, cx| {
                                this.palette_edit("Duplicate", cx)
                            }),
                        )
                        .when(custom, |d| {
                            d.child(
                                self.control("delete-palette", "Delete", cx, |this, _, cx| {
                                    this.palette_edit("Delete", cx)
                                })
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
                                row()
                                    .id(("manager-swatch", i))
                                    .size(px(32.0))
                                    .justify_center()
                                    .rounded(px(5.0))
                                    .bg(color_fill(c))
                                    .border_2()
                                    .border_color(if i == self.palettes.slot {
                                        t.text()
                                    } else {
                                        t.swatch_edge()
                                    })
                                    .cursor_pointer()
                                    .when(i == self.palettes.slot, |d| {
                                        d.child(icon("check").text_color(swatch_ink(c)))
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.palettes.slot = i;
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
                                "Apply color",
                                cx,
                                |this, _, cx| this.palette_edit("Apply color", cx),
                            ))
                        }),
                )
                .when(custom, |d| {
                    d.child(
                        row().gap(px(6.0)).children(
                            ["Add slot", "Remove slot", "Move left", "Move right"]
                                .into_iter()
                                .enumerate()
                                .map(|(i, label)| {
                                    self.control(("slot-op", i), label, cx, move |this, _, cx| {
                                        this.palette_edit(label, cx)
                                    })
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
                        .child(
                            self.control("import-palette", "Import…", cx, |this, _, cx| {
                                this.palette_file(None, cx)
                            }),
                        )
                        .child(
                            self.control("export-hex", "Export HEX…", cx, |this, _, cx| {
                                this.palette_file(Some("hex"), cx)
                            }),
                        )
                        .child(
                            self.control("export-gpl", "Export GPL…", cx, |this, _, cx| {
                                this.palette_file(Some("gpl"), cx)
                            }),
                        ),
                )
                .child(div().text_size(px(12.0)).text_color(t.muted()).child(
                    "Palette selection applies to new voxels. Existing colors stay intact.",
                ));
        div()
            .absolute()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000060))
            .child(
                self.floating()
                    .w(px(780.0))
                    .h(px(620.0))
                    .max_h(relative(0.95))
                    .flex()
                    .flex_col()
                    .p(px(0.0))
                    .child(
                        row()
                            .px(px(24.0))
                            .py(px(16.0))
                            .justify_between()
                            .border_b_1()
                            .border_color(t.line())
                            .child(section_title("Palettes"))
                            .child(self.control(
                                "close-palettes",
                                "Done",
                                cx,
                                |this, window, cx| {
                                    this.palettes.open = false;
                                    window.focus(&this.focus, cx);
                                    cx.notify();
                                },
                            )),
                    )
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
