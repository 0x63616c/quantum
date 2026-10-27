//! Grouped palette library, swatch grid and color editor presentation.
use super::*;
use crate::ui::palette_layout::{self as layout, CELL, COLUMNS, GAP};
use voxelite_plugin_api::{PaletteKind, RecentColorTarget};
use voxelite_ui::tokens::ControlSize;
use voxelite_ui::{
    Badge, Card, ColorSquare, HueSlider, Menu, Segmented, SidebarItem, SidebarSection, Swatch,
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
        Swatch::new(self.theme, rgb_fill(self.rgb))
            .selected(true)
            .build()
    }
}
fn rgb_fill(rgb: [u8; 3]) -> Hsla {
    color_fill([rgb[0], rgb[1], rgb[2], 255])
}
fn caption(text: impl Into<SharedString>, t: Theme) -> Div {
    div()
        .text_size(px(type_size::BODY))
        .text_color(t.muted())
        .child(text.into())
}
fn thumbnail(palette: &Palette, t: Theme) -> Div {
    let colors: Vec<_> = layout::thumbnail_indices(palette.order.len())
        .iter()
        .filter_map(|i| palette.get(palette.order[*i]).map(|s| color_fill(s.rgba)))
        .collect();
    div()
        .w(px(44.))
        .h(px(28.))
        .flex_shrink_0()
        .rounded(px(4.))
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
                                size(width + px(0.5), height + px(0.5)),
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
            .min_h(px(0.))
            .overflow_y_scroll()
            .track_scroll(&self.palettes.sidebar_scroll)
            .px(px(10.));
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
                    SidebarItem::custom(
                        ("palette-choice", index),
                        format!(
                            "{}, {}{}",
                            choice.name,
                            color_count,
                            if choice.read_only { ", read-only" } else { "" }
                        ),
                        selected,
                        t,
                    )
                    .tab_stop(false)
                    .h(px(52.))
                    .px(px(8.))
                    .gap(px(10.))
                    .mb(px(2.))
                    .child(thumbnail(&choice.palette, t))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(choice.name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(if selected {
                                        t.accent_fg().opacity(0.8)
                                    } else {
                                        t.muted()
                                    })
                                    .child(color_count),
                            ),
                    )
                    .when(choice.read_only, |d| {
                        d.child(icon("lock").size(px(12.)).text_color(if selected {
                            t.accent_fg().opacity(0.8)
                        } else {
                            t.muted()
                        }))
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.manager_action(PaletteAction::ChoosePalette(id), cx);
                        this.palettes.grid_scroll.set_offset(point(px(0.), px(0.)));
                        window.focus(&this.palettes.sidebar_focus, cx);
                    })),
                );
            }
            if count == 0 {
                choices = choices.child(
                    caption("Your palettes appear here.", t)
                        .px(px(12.))
                        .py(px(8.)),
                );
            }
        }
        div()
            .w(px(236.))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(t.sidebar())
            .border_r_1()
            .border_color(t.separator())
            .child(
                div()
                    .id("palette-library")
                    .role(Role::ListBox)
                    .aria_label("Palettes")
                    .tab_index(0)
                    .track_focus(&self.palettes.sidebar_focus)
                    .flex_1()
                    .min_h(px(0.))
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(transparent_black())
                    .focus_visible(move |d| d.border_color(t.accent()))
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
                    .p(px(12.))
                    .border_t_1()
                    .border_color(t.separator())
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        Button::new("new-palette", "New palette", t)
                            .icon("plus")
                            .build()
                            .w_full()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.palette_edit(PaletteOperation::Create, cx)
                            })),
                    )
                    .child(
                        Button::new("duplicate-palette", "Duplicate palette", t)
                            .variant(ButtonVariant::Ghost)
                            .icon("copy")
                            .build()
                            .w_full()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.palette_edit(PaletteOperation::Duplicate, cx)
                            })),
                    )
                    .when(view.kind == PaletteKind::Custom, |d| {
                        d.child(
                            Button::new("delete-palette", "Delete palette…", t)
                                .variant(ButtonVariant::Ghost)
                                .icon("trash-2")
                                .build()
                                .w_full()
                                .text_color(t.danger())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.palettes.delete_confirmation = true;
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        caption("Editing a read-only palette makes an editable copy.", t)
                            .mt(px(6.))
                            .line_height(px(16.)),
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
            .p(px(0.))
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
                Swatch::new(t, rgb_fill(rgb))
                    .selected(index == view.state.slot)
                    .build()
                    .id(("manager-swatch", index))
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .aria_selected(index == view.state.slot)
                    .cursor_pointer()
                    .anchored_hint(label, t, TooltipSide::Above)
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
            Button::new("add-swatch", "+", t)
                .size(ControlSize::Md)
                .disabled(count == 256)
                .build()
                .w(px(CELL))
                .px(px(0.))
                .anchored_hint(
                    if count == 256 {
                        "256 swatches maximum per palette"
                    } else {
                        "Add color"
                    },
                    t,
                    TooltipSide::Above,
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
                    .w(px(2.))
                    .h(px(CELL + 2.))
                    .bg(t.accent()),
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
            .gap(px(8.))
            .child(
                div()
                    .id("palette-grid")
                    .tab_index(0)
                    .track_focus(&self.palettes.grid_focus)
                    .role(Role::ListBox)
                    .aria_label("Palette colors; arrow keys select, Delete removes")
                    .h(px(118.))
                    .flex_shrink_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.palettes.grid_scroll)
                    .p(px(3.))
                    .focus_visible(move |d| d.bg(t.sidebar()).rounded(px(4.)))
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
                            "d" if ShortcutPlatform::current()
                                .is_primary(e.keystroke.modifiers) =>
                            {
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
            .child(
                row()
                    .gap(px(6.))
                    .child(caption(format!("Color {}", slot + 1), t).flex_1())
                    .child(
                        Button::new("duplicate-swatch", "Duplicate swatch", t)
                            .variant(ButtonVariant::Ghost)
                            .size(ControlSize::Sm)
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
                        Button::new("remove-swatch", "Remove", t)
                            .variant(ButtonVariant::Ghost)
                            .size(ControlSize::Sm)
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
                            )
                            .when(count > 1, |d| {
                                d.on_click(cx.listener(move |this, _, _, cx| {
                                    this.manager_action(PaletteAction::RemoveSwatch(slot), cx)
                                }))
                            }),
                    ),
            )
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
            .set_offset(point(px(0.), -px(offset)));
    }
    fn palette_color_editor(&self, view: &PaletteView, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let current = selected_rgb(view);
        let hsb = self.palettes.hsb;
        let fields = div()
            .flex_1()
            .min_w(px(0.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .gap(px(10.))
                    .child(
                        div()
                            .child(caption("Hex", t).mb(px(4.)))
                            .child(self.palettes.hex.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(caption("Previous / current", t).mb(px(4.)))
                            .child(
                                row()
                                    .h(px(40.))
                                    .rounded(px(6.))
                                    .overflow_hidden()
                                    .border_1()
                                    .border_color(t.swatch_edge())
                                    .child(
                                        div()
                                            .id("previous-color")
                                            .tab_index(0)
                                            .role(Role::Button)
                                            .aria_label("Restore previous color")
                                            .flex_1()
                                            .h_full()
                                            .bg(rgb_fill(self.palettes.before))
                                            .cursor_pointer()
                                            .focus_visible(move |d| {
                                                d.border_2().border_color(t.accent())
                                            })
                                            .anchored_hint(
                                                format!(
                                                    "Previous {} · Click to restore",
                                                    rgb_hex(self.palettes.before)
                                                ),
                                                t,
                                                TooltipSide::Above,
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
                                            .bg(rgb_fill(current)),
                                    ),
                            ),
                    ),
            )
            .children((0..2).map(|group| {
                row().gap(px(8.)).children((0..3).map(|column| {
                    let index = group * 3 + column;
                    div()
                        .flex_1()
                        .child(caption(["R", "G", "B", "H °", "S %", "B %"][index], t).mb(px(4.)))
                        .child(self.palettes.channels[index].clone())
                }))
            }));
        Card::new(t).rows([div()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
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
                    .gap(px(16.))
                    .child(
                        div()
                            .w(px(216.))
                            .flex_shrink_0()
                            .min_w(px(0.))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
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
            .gap(px(8.))
            .child(row().child(caption("Recent colors", t).flex_1()).child(
                Segmented::row(t).children([false, true].map(|target_add| {
                    Button::new(
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
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Sm)
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
                    .gap(px(8.))
                    .h(px(24.))
                    .when(view.state.recent_colors.is_empty(), |d| {
                        d.child(caption("Colors you use will appear here.", t))
                    })
                    .children(view.state.recent_colors.iter().take(16).enumerate().map(
                        |(index, rgb)| {
                            Swatch::new(t, rgb_fill(*rgb))
                                .size(ControlSize::Sm)
                                .build()
                                .id(("recent-color", index))
                                .tab_index(0)
                                .role(Role::Button)
                                .aria_label(rgb_hex(*rgb))
                                .cursor_pointer()
                                .anchored_hint(rgb_hex(*rgb), t, TooltipSide::Above)
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
        let main = div()
            .id("palette-main-scroll")
            .flex_1()
            .min_w(px(0.))
            .min_h(px(0.))
            .overflow_y_scroll()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .when(view.read_only, |d| {
                                d.child(
                                    voxelite_ui::TextField::new(t)
                                        .build()
                                        .flex()
                                        .items_center()
                                        .py(px(0.))
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
            .child(caption(notice, t).h(px(18.)))
            .child(self.palette_grid(&view, cx))
            .child(self.palette_color_editor(&view, cx))
            .child(self.palette_recents(&view, cx))
            .when(self.palettes.delete_confirmation, |d| {
                d.child(
                    row()
                        .gap(px(8.))
                        .child(caption(format!("Delete “{}”?", view.name), t).flex_1())
                        .child(
                            Button::new("cancel-delete-palette", "Cancel", t)
                                .size(ControlSize::Sm)
                                .build()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.palettes.delete_confirmation = false;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("confirm-delete-palette", "Delete", t)
                                .variant(ButtonVariant::Danger)
                                .size(ControlSize::Sm)
                                .build()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.palette_edit(PaletteOperation::Delete, cx)
                                })),
                        ),
                )
            })
            .child(
                div()
                    .min_h(px(18.))
                    .text_size(px(12.))
                    .when_some(self.palettes.error.clone(), |d, error| {
                        d.text_color(t.danger()).child(error)
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
                            .right(px(0.))
                            .top(px(38.))
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
                            .child(caption("HEX, GPL, JASC-PAL", t).px(px(12.)).pb(px(8.)))
                            .child(div().h(px(1.)).bg(t.separator()).my(px(4.)))
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
            .w(px(860.))
            .h(px(780.))
            .p(px(0.))
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
                    Button::new("close-palettes", "Done", t)
                        .variant(ButtonVariant::Primary)
                        .build()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.close_palettes(window, cx)),
                        ),
                ),
            )
            .child(
                row()
                    .flex_1()
                    .min_h(px(0.))
                    .items_stretch()
                    .child(self.palette_sidebar(&view, cx))
                    .child(main),
            );
        #[cfg(not(target_family = "wasm"))]
        let panel = panel.on_drop(
            cx.listener(|this, paths: &ExternalPaths, _, cx| this.palette_drop_file(paths, cx)),
        );
        Dialog::scrim(t, false).child(panel).into_any_element()
    }
}
