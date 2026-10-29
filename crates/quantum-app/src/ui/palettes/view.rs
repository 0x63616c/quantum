//! Grouped palette library, swatch grid and color editor presentation.
use super::*;
use crate::ui::palette_layout::{self as layout, CELL, COLUMNS, GAP};
use quantum_plugin_api::{PaletteKind, RecentColorTarget};
use quantum_ui::ColorToken;
use quantum_ui::Radius;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;
use quantum_ui::tokens::geometry;
use quantum_ui::{
    Badge, Card, ColorSquare, HueSlider, Menu, Segment, SegmentedControl, SidebarSection, Swatch,
};

#[derive(Clone)]
struct SwatchDrag {
    palette: PaletteId,
    from: usize,
    rgb: [u8; 3],
    theme: Theme,
}
impl Render for SwatchDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Swatch::design(self.theme, rgb_fill(self.rgb))
            .selected(true)
            .build()
    }
}
fn rgb_fill(rgb: [u8; 3]) -> Hsla {
    color_fill([rgb[0], rgb[1], rgb[2], 255])
}
fn caption(text: impl Into<SharedString>, t: Theme) -> Div {
    div()
        .text_style(TextSize::Small)
        .text_color(t.color(ColorToken::TextSecondary))
        .child(text.into())
}
fn thumbnail(palette: &Palette, t: Theme) -> Div {
    let colors: Vec<_> = layout::thumbnail_indices(palette.order.len())
        .iter()
        .filter_map(|i| palette.get(palette.order[*i]).map(|s| color_fill(s.rgba)))
        .collect();
    div()
        .w(px(metrics::THUMBNAIL_WIDTH))
        .h(px(Space::S7.px()))
        .flex_shrink_0()
        .rounded(px(Radius::Xs.px()))
        .overflow_hidden()
        .border_1()
        .border_color(t.swatch_edge())
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let columns = colors.len().clamp(1, 8);
                    let rows = colors.len().div_ceil(columns).max(1);
                    for (i, color) in colors.iter().enumerate() {
                        let width = bounds.size.width / columns as f32;
                        let height = bounds.size.height / rows as f32;
                        window.paint_quad(fill(
                            Bounds::new(
                                point(
                                    bounds.left() + width * (i % columns) as f32,
                                    bounds.top() + height * (i / columns) as f32,
                                ),
                                size(
                                    width + px(geometry::COLOR_CELL_OVERLAP),
                                    height + px(geometry::COLOR_CELL_OVERLAP),
                                ),
                            ),
                            *color,
                        ));
                    }
                },
            )
            .size_full(),
        )
}
impl EditorView {
    fn palette_sidebar(&self, view: &PaletteView, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let mut choices = div()
            .id("palette-library-scroll")
            .flex_1()
            .min_h(px(Space::S0.px()))
            .overflow_y_scroll()
            .track_scroll(&self.palettes.sidebar_scroll)
            .px(px(Space::S4.px()));
        for (kind, label) in [
            (PaletteKind::Project, "Project"),
            (PaletteKind::BuiltIn, "Built-in"),
            (PaletteKind::Custom, "Custom"),
        ] {
            choices = choices.child(SidebarSection::header(label, t));
            let mut count = 0;
            for (index, choice) in view
                .choices
                .iter()
                .enumerate()
                .filter(|(_, c)| c.kind == kind)
            {
                count += 1;
                let id = choice.id;
                let selected = view.selected == id;
                let color_count = format!(
                    "{} {}",
                    choice.palette.order.len(),
                    if choice.palette.order.len() == 1 {
                        "color"
                    } else {
                        "colors"
                    }
                );
                choices = choices.child(
                    quantum_ui::ListRow::new(("palette-choice", index), choice.name.clone(), t)
                        .meta(color_count)
                        .leading(thumbnail(&choice.palette, t))
                        .selected(selected)
                        .navigation(true)
                        .trailing(div().when(choice.read_only, |d| {
                            d.child(icon("lock").text_color(t.color(ColorToken::TextSecondary)))
                        }))
                        .build()
                        .tab_stop(false)
                        .h(px(metrics::COMMAND_SEARCH_HEIGHT))
                        .mb(px(Space::S1.px()))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.manager_action(PaletteAction::ChoosePalette(id), cx);
                            this.palettes
                                .grid_scroll
                                .set_offset(point(px(Space::S0.px()), px(Space::S0.px())));
                            window.focus(&this.palettes.sidebar_focus, cx);
                        })),
                );
            }
            if count == 0 {
                choices = choices.child(
                    caption("Your palettes appear here.", t)
                        .px(px(Space::S4.px()))
                        .py(px(Space::S3.px())),
                );
            }
        }
        Panel::section(t)
            .w(px(metrics::PALETTE_SIDEBAR_WIDTH))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(t.color(ColorToken::Surface))
            // This background meets the dialog's bottom-left border. GPUI's
            // overflow mask is rectangular, so it needs its own inner corner.
            .rounded_bl(px(radius::concentric(Radius::Lg.px(), metrics::EDGE)))
            .border_r_1()
            .border_color(t.color(ColorToken::Border))
            .child(
                quantum_ui::FocusGroup::frame(
                    "palette-library",
                    "Palettes",
                    t,
                    &self.palettes.sidebar_focus,
                )
                .flex_1()
                .min_h(px(Space::S0.px()))
                .flex()
                .flex_col()
                .border_1()
                .border_color(transparent_black())
                .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| {
                    if !matches!(e.keystroke.key.as_str(), "up" | "down" | "home" | "end") {
                        return;
                    }
                    if let Some(view) = this.editor.palette_view() {
                        let current = view
                            .choices
                            .iter()
                            .position(|c| c.id == view.selected)
                            .unwrap_or(0);
                        let next =
                            layout::navigate(current, view.choices.len(), 1, &e.keystroke.key);
                        if let Some(choice) = view.choices.get(next) {
                            this.manager_action(PaletteAction::ChoosePalette(choice.id), cx);
                        }
                    }
                    cx.stop_propagation();
                }))
                .child(choices),
            )
            .child(
                div()
                    .p(px(Space::S4.px()))
                    .border_t_1()
                    .border_color(t.color(ColorToken::Border))
                    .flex()
                    .flex_col()
                    .gap(px(Space::S3.px()))
                    .child(
                        Button::design("new-palette", "New palette", t)
                            .icon("plus")
                            .build()
                            .w_full()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.palette_edit(PaletteOperation::Create, cx)
                            })),
                    )
                    .child(
                        Button::design("duplicate-palette", "Duplicate palette", t)
                            .style(quantum_ui::ButtonStyle::Ghost)
                            .icon("copy")
                            .build()
                            .w_full()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.palette_edit(PaletteOperation::Duplicate, cx)
                            })),
                    )
                    .when(view.kind == PaletteKind::Custom, |d| {
                        d.child(
                            Button::design("delete-palette", "Delete palette…", t)
                                .style(quantum_ui::ButtonStyle::Ghost)
                                .icon("trash-2")
                                .build()
                                .w_full()
                                .text_color(t.color(ColorToken::Danger))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.palettes.delete_confirmation = true;
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        caption("Editing a read-only palette makes an editable copy.", t)
                            .mt(px(Space::S3.px())),
                    ),
            )
    }
    fn palette_grid(&self, view: &PaletteView, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let count = view.palette.order.len();
        let id = view.selected;
        let bounds = Rc::new(Cell::new(Bounds::default()));
        let painted = bounds.clone();
        let mut grid = div()
            .id("palette-grid-cells")
            .relative()
            .flex()
            .flex_wrap()
            .gap(px(GAP))
            .w(px(COLUMNS as f32 * (CELL + GAP) - GAP))
            .p(px(Space::S0.px()))
            .child(
                canvas(move |b, _, _| painted.set(b), |_, _, _, _| ())
                    .absolute()
                    .size_full(),
            );
        for (index, slot) in view.palette.order.iter().enumerate() {
            let Some(swatch) = view.palette.get(*slot) else {
                continue;
            };
            let rgb = [swatch.rgba[0], swatch.rgba[1], swatch.rgba[2]];
            let label = match &swatch.name {
                Some(name) => format!("{} · {name}", rgb_hex(rgb)),
                None => rgb_hex(rgb),
            };
            grid = grid.child(
                Swatch::design(t, rgb_fill(rgb))
                    .selected(index == view.state.slot)
                    .build()
                    .id(("manager-swatch", index))
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .aria_selected(index == view.state.slot)
                    .cursor_pointer()
                    .anchored_hint(label, t, TooltipSide::Above, &self.overlays)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.manager_action(PaletteAction::SelectSlot(index), cx);
                        window.focus(&this.palettes.grid_focus, cx);
                    }))
                    .on_drag(
                        SwatchDrag {
                            palette: id,
                            from: index,
                            rgb,
                            theme: t,
                        },
                        |drag, _, _, cx| cx.new(|_| drag.clone()),
                    ),
            );
        }
        grid = grid.child(
            Button::design("add-swatch", "+", t)
                .height(quantum_ui::ControlHeight::Default)
                .disabled(count == 256)
                .build()
                .w(px(CELL))
                .px(px(Space::S0.px()))
                .anchored_hint(
                    if count == 256 {
                        "256 swatches maximum per palette"
                    } else {
                        "Add color"
                    },
                    t,
                    TooltipSide::Above,
                    &self.overlays,
                )
                .when(count < 256, |d| {
                    d.on_click(cx.listener(|this, _, window, cx| {
                        this.manager_action(
                            PaletteAction::AddSwatch {
                                rgb: this.palettes.hsb.to_rgb(),
                                at: None,
                            },
                            cx,
                        );
                        this.reveal_palette_swatch();
                        window.focus(&this.palettes.grid_focus, cx);
                    }))
                }),
        );
        if let Some(boundary) = self.palettes.drop_boundary {
            let (col, row) = if boundary > 0 && boundary.is_multiple_of(COLUMNS) {
                (COLUMNS, boundary / COLUMNS - 1)
            } else {
                (boundary % COLUMNS, boundary / COLUMNS)
            };
            grid = grid.child(
                div()
                    .absolute()
                    .left(px(col as f32 * (CELL + GAP) - GAP / 2. - 1.))
                    .top(px(row as f32 * (CELL + GAP) - 1.))
                    .w(px(Space::S1.px()))
                    .h(px(CELL + geometry::EDGE * 2.))
                    .bg(t.color(ColorToken::AccentText)),
            );
        }
        grid = grid
            .on_drag_move(
                cx.listener(move |this, event: &DragMoveEvent<SwatchDrag>, _, cx| {
                    let next = (event.drag(cx).palette == id
                        && event.bounds.contains(&event.event.position))
                    .then(|| {
                        layout::insertion(
                            f32::from(event.event.position.x - event.bounds.left()),
                            f32::from(event.event.position.y - event.bounds.top()),
                            count,
                            COLUMNS,
                        )
                    });
                    if this.palettes.drop_boundary != next {
                        this.palettes.drop_boundary = next;
                        cx.notify();
                    }
                }),
            )
            .on_drop(cx.listener(move |this, drag: &SwatchDrag, window, cx| {
                if drag.palette == id {
                    let b = bounds.get();
                    let p = window.mouse_position();
                    let boundary = layout::insertion(
                        f32::from(p.x - b.left()),
                        f32::from(p.y - b.top()),
                        count,
                        COLUMNS,
                    );
                    this.manager_action(
                        PaletteAction::MoveSwatch {
                            from: drag.from,
                            to: layout::drop_position(drag.from, boundary, count),
                        },
                        cx,
                    );
                }
                this.palettes.drop_boundary = None;
                window.focus(&this.palettes.grid_focus, cx);
                cx.notify();
            }));
        let slot = view.state.slot;
        div()
            .flex()
            .flex_col()
            .gap(px(Space::S3.px()))
            .child(
                quantum_ui::FocusGroup::frame(
                    "palette-grid",
                    "Palette colors; arrow keys select, Delete removes",
                    t,
                    &self.palettes.grid_focus,
                )
                .h(px(metrics::PALETTE_GRID_HEIGHT))
                .flex_shrink_0()
                .overflow_y_scroll()
                .track_scroll(&self.palettes.grid_scroll)
                .p(px(Space::S2.px()))
                .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| {
                    let Some(view) = this.editor.palette_view() else {
                        return;
                    };
                    let key = e.keystroke.key.as_str();
                    let action = match key {
                        "left" | "right" | "up" | "down" | "home" | "end" => {
                            PaletteAction::SelectSlot(layout::navigate(
                                view.state.slot,
                                view.palette.order.len(),
                                COLUMNS,
                                key,
                            ))
                        }
                        "delete" | "backspace" => PaletteAction::RemoveSwatch(view.state.slot),
                        "d" if ShortcutPlatform::current().is_primary(e.keystroke.modifiers) => {
                            PaletteAction::DuplicateSwatch(view.state.slot)
                        }
                        _ => return,
                    };
                    this.manager_action(action, cx);
                    this.reveal_palette_swatch();
                    cx.stop_propagation();
                }))
                .child(grid),
            )
            .child(self.palette_swatch_actions(slot, count, cx))
    }
    fn reveal_palette_swatch(&self) {
        let Some(view) = self.editor.palette_view() else {
            return;
        };
        let top = (view.state.slot / COLUMNS) as f32 * (CELL + GAP);
        let current = -f32::from(self.palettes.grid_scroll.offset().y);
        let offset = if top < current {
            top
        } else if top + CELL > current + 112. {
            top + CELL - 112.
        } else {
            current
        };
        self.palettes
            .grid_scroll
            .set_offset(point(px(Space::S0.px()), -px(offset)));
    }
    fn palette_color_editor(&self, view: &PaletteView, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let current = selected_rgb(view);
        let hsb = self.palettes.hsb;
        let fields = div()
            .flex_1()
            .min_w(px(Space::S0.px()))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(Space::S4.px()))
            .child(
                row()
                    .gap(px(Space::S4.px()))
                    .child(
                        div()
                            .child(caption("Hex", t).mb(px(Space::S2.px())))
                            .child(self.palettes.hex.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(caption("Previous / current", t).mb(px(Space::S2.px())))
                            .child(self.previous_color(current, cx)),
                    ),
            )
            .children((0..2).map(|group| {
                row().gap(px(Space::S3.px())).children((0..3).map(|column| {
                    let index = group * 3 + column;
                    div()
                        .flex_1()
                        .child(
                            caption(["R", "G", "B", "H °", "S %", "B %"][index], t)
                                .mb(px(Space::S2.px())),
                        )
                        .child(self.palettes.channels[index].clone())
                }))
            }));
        Card::new(t).rows([div()
            .p(px(Space::S5.px()))
            .flex()
            .flex_col()
            .gap(px(Space::S4.px()))
            .child(
                row()
                    .child(
                        div()
                            .flex_1()
                            .font_weight(type_size::SEMIBOLD)
                            .child("Color"),
                    )
                    .when(view.state.preview.is_some(), |d| {
                        d.child(caption("Preview · Esc to cancel", t))
                    })
                    .when(cfg!(target_os = "macos"), |d| {
                        #[cfg(target_os = "macos")]
                        let d = d.child(self.palette_eyedropper(cx));
                        d
                    }),
            )
            .child(
                row()
                    .items_start()
                    .gap(px(Space::S5.px()))
                    .child(
                        div()
                            .w(px(metrics::COLOR_EDITOR_WIDTH))
                            .flex_shrink_0()
                            .min_w(px(Space::S0.px()))
                            .flex()
                            .flex_col()
                            .gap(px(Space::S3.px()))
                            .child(
                                ColorSquare::new("palette-sb", t, hsb)
                                    .gesture(self.palettes.square_gesture.clone())
                                    .on_change(cx.listener(|this, change, _, cx| {
                                        this.picker_change(change, cx)
                                    })),
                            )
                            .child(
                                HueSlider::new("palette-hue", t, hsb)
                                    .gesture(self.palettes.hue_gesture.clone())
                                    .on_change(cx.listener(|this, change, _, cx| {
                                        this.picker_change(change, cx)
                                    })),
                            ),
                    )
                    .child(fields),
            )])
    }
    fn palette_recents(&self, view: &PaletteView, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let add = self.palettes.recent_add;
        div()
            .flex()
            .flex_col()
            .gap(px(Space::S3.px()))
            .child(row().child(caption("Recent colors", t).flex_1()).child(
                SegmentedControl::design(t).children([false, true].map(|target_add| {
                    Segment::design(
                        if target_add {
                            "recent-add"
                        } else {
                            "recent-apply"
                        },
                        if target_add {
                            "+ Add swatch"
                        } else {
                            "Apply to selected"
                        },
                        t,
                    )
                    .height(quantum_ui::ControlHeight::Dense)
                    .selected(add == target_add)
                    .build()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.palettes.recent_add = target_add;
                        cx.notify();
                    }))
                })),
            ))
            .child(
                row()
                    .gap(px(Space::S3.px()))
                    .h(px(Space::S6.px()))
                    .when(view.state.recent_colors.is_empty(), |d| {
                        d.child(caption("Colors you use will appear here.", t))
                    })
                    .children(view.state.recent_colors.iter().take(16).enumerate().map(
                        |(index, rgb)| {
                            Swatch::design(t, rgb_fill(*rgb))
                                .height(quantum_ui::ControlHeight::Dense)
                                .interactive(("recent-color", index), rgb_hex(*rgb))
                                .cursor_pointer()
                                .anchored_hint(rgb_hex(*rgb), t, TooltipSide::Above, &self.overlays)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.manager_action(
                                        PaletteAction::UseRecentColor {
                                            index,
                                            target: if this.palettes.recent_add {
                                                RecentColorTarget::NewSwatch { at: None }
                                            } else {
                                                RecentColorTarget::SelectedSwatch
                                            },
                                        },
                                        cx,
                                    );
                                    this.reveal_palette_swatch();
                                }))
                        },
                    )),
            )
    }
    pub(in crate::ui) fn palette_manager(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        let Some(view) = self.editor.palette_view() else {
            return div().into_any_element();
        };
        let kind = match view.kind {
            PaletteKind::Project => "Project · Read-only",
            PaletteKind::BuiltIn => "Built-in · Read-only",
            PaletteKind::Custom => "Custom",
        };
        let notice = if let Some((id, source)) = &self.palettes.copied_from
            && *id == view.selected
        {
            format!("Editing a copy of {source}")
        } else if view.read_only {
            "Read-only palette. Your first edit makes a custom copy.".into()
        } else {
            "Drag colors to reorder. Changes are saved automatically.".into()
        };
        let main = Panel::section(t)
            .id("palette-main-scroll")
            .flex_1()
            .min_w(px(Space::S0.px()))
            .min_h(px(Space::S0.px()))
            .overflow_y_scroll()
            .p(px(Space::S5.px()))
            .flex()
            .flex_col()
            .gap(px(Space::S4.px()))
            .child(
                row()
                    .gap(px(Space::S4.px()))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(Space::S0.px()))
                            .when(view.read_only, |d| {
                                d.child(
                                    quantum_ui::FieldFrame::design(t)
                                        .build()
                                        .flex()
                                        .items_center()
                                        .py(px(Space::S0.px()))
                                        .cursor_default()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(view.name.clone()),
                                )
                            })
                            .when(!view.read_only, |d| d.child(self.palettes.name.clone())),
                    )
                    .child(Badge::label(kind, t))
                    .child(caption(format!("{} / 256", view.palette.order.len()), t)),
            )
            .child(caption(notice, t).h(px(Space::S5.px())))
            .child(self.palette_grid(&view, cx))
            .child(self.palette_color_editor(&view, cx))
            .child(self.palette_recents(&view, cx))
            .when(self.palettes.delete_confirmation, |d| {
                d.child(
                    row()
                        .gap(px(Space::S3.px()))
                        .child(caption(format!("Delete “{}”?", view.name), t).flex_1())
                        .child(
                            Button::design("cancel-delete-palette", "Cancel", t)
                                .height(quantum_ui::ControlHeight::Dense)
                                .build()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.palettes.delete_confirmation = false;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::design("confirm-delete-palette", "Delete", t)
                                .style(quantum_ui::ButtonStyle::Danger)
                                .height(quantum_ui::ControlHeight::Dense)
                                .build()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.palette_edit(PaletteOperation::Delete, cx)
                                })),
                        ),
                )
            })
            .child(
                div()
                    .min_h(px(Space::S5.px()))
                    .text_style(TextSize::Small)
                    .when_some(self.palettes.error.clone(), |d, error| {
                        d.text_color(t.color(ColorToken::Danger)).child(error)
                    }),
            );
        let menu = self.palettes.menu;
        let trigger = div()
            .relative()
            .child(
                Menu::button("palette-interchange", "Import / Export", menu, t).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.palettes.menu = !this.palettes.menu;
                        this.palettes.menu_index = 0;
                        cx.notify();
                    }),
                ),
            )
            .when(menu, |d| {
                d.child(
                    deferred(
                        Menu::panel("palette-file-menu", t)
                            .absolute()
                            .right(px(Space::S0.px()))
                            .top(px(metrics::SWATCH_PITCH))
                            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                this.palettes.menu = false;
                                cx.notify();
                            }))
                            .child(
                                Menu::item(
                                    "palette-import",
                                    "Import palette…",
                                    self.palettes.menu_index == 0,
                                    t,
                                )
                                .tab_stop(false)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.palette_menu_action(0, cx)),
                                ),
                            )
                            .child(
                                caption("HEX, GPL, JASC-PAL", t)
                                    .px(px(Space::S4.px()))
                                    .pb(px(Space::S3.px())),
                            )
                            .child(
                                div()
                                    .h(px(metrics::EDGE))
                                    .bg(t.color(ColorToken::Border))
                                    .my(px(Space::S2.px())),
                            )
                            .children(view.formats.iter().enumerate().map(|(index, format)| {
                                Menu::item(
                                    ("palette-export", index),
                                    format!("Export {}…", format.label),
                                    self.palettes.menu_index == index + 1,
                                    t,
                                )
                                .tab_stop(false)
                                .on_click(cx.listener(
                                    move |this, _, _, cx| this.palette_menu_action(index + 1, cx),
                                ))
                            })),
                    )
                    .with_priority(2),
                )
            });
        let panel = Dialog::panel("palette-manager", "Palettes", t, &self.modal_focus)
            .w(px(metrics::PALETTE_DIALOG_WIDTH))
            .h(px(metrics::PALETTE_DIALOG_HEIGHT))
            .p(px(Space::S0.px()))
            .capture_key_down(cx.listener(Self::palette_dialog_key))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.palettes.drop_boundary.take().is_some() {
                        cx.notify();
                    }
                }),
            )
            .child(
                Dialog::title("Palettes", t).child(trigger).child(
                    Button::design("close-palettes", "Done", t)
                        .style(quantum_ui::ButtonStyle::Primary)
                        .build()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.close_palettes(window, cx)),
                        ),
                ),
            )
            .child(
                row()
                    .flex_1()
                    .min_h(px(Space::S0.px()))
                    .items_stretch()
                    .child(self.palette_sidebar(&view, cx))
                    .child(main),
            );
        let panel = panel.on_drop(
            cx.listener(|this, paths: &ExternalPaths, _, cx| this.palette_drop_file(paths, cx)),
        );
        Dialog::scrim(t, false).child(panel).into_any_element()
    }
}

impl EditorView {
    fn previous_color(&self, current: [u8; 3], cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .h(px(quantum_ui::tokens::ControlSize::Lg.height()))
            .rounded(px(Radius::Sm.px()))
            .border_1()
            .border_color(t.swatch_edge())
            .child(
                Swatch::design(t, rgb_fill(self.palettes.before))
                    .interactive("previous-color", "Restore previous color")
                    .flex_1()
                    .h_full()
                    .border_0()
                    .bg(rgb_fill(self.palettes.before))
                    .rounded_l(px(Radius::Sm.px()))
                    .rounded_r(px(Space::S0.px()))
                    .cursor_pointer()
                    .anchored_hint(
                        format!(
                            "Previous {} · Click to restore",
                            rgb_hex(self.palettes.before)
                        ),
                        t,
                        TooltipSide::Above,
                        &self.overlays,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.manager_action(
                            PaletteAction::SetColor {
                                rgb: this.palettes.before,
                                commit: true,
                            },
                            cx,
                        )
                    })),
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .border_l_1()
                    .border_color(t.swatch_edge())
                    .bg(rgb_fill(current))
                    .rounded_r(px(Radius::Sm.px())),
            )
    }
}

impl EditorView {
    fn palette_swatch_actions(&self, slot: usize, count: usize, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .gap(px(Space::S3.px()))
            .child(caption(format!("Color {}", slot + 1), t).flex_1())
            .child(
                Button::design("duplicate-swatch", "Duplicate swatch", t)
                    .style(quantum_ui::ButtonStyle::Ghost)
                    .height(quantum_ui::ControlHeight::Dense)
                    .disabled(count == 256)
                    .build()
                    .when(count < 256, |d| {
                        d.on_click(cx.listener(move |this, _, _, cx| {
                            this.manager_action(PaletteAction::DuplicateSwatch(slot), cx);
                            this.reveal_palette_swatch();
                        }))
                    }),
            )
            .child(
                Button::design("remove-swatch", "Remove", t)
                    .style(quantum_ui::ButtonStyle::Ghost)
                    .height(quantum_ui::ControlHeight::Dense)
                    .disabled(count <= 1)
                    .build()
                    .anchored_hint(
                        if count <= 1 {
                            "Cannot remove the last swatch from a palette"
                        } else {
                            "Remove selected swatch"
                        },
                        t,
                        TooltipSide::Above,
                        &self.overlays,
                    )
                    .when(count > 1, |d| {
                        d.on_click(cx.listener(move |this, _, _, cx| {
                            this.manager_action(PaletteAction::RemoveSwatch(slot), cx)
                        }))
                    }),
            )
    }
}
