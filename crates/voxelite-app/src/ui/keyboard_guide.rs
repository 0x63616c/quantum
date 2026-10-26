//! Compact, structured reference for the editor's keyboard and mouse bindings.
use super::{action_shortcut, row, shortcut};
use crate::{actions::Op, theme::Theme};
use gpui::{prelude::*, *};

enum GuideShortcut {
    Action(Op),
    Keys(&'static [&'static str]),
    Gesture(&'static str),
}

struct GuideSection {
    title: &'static str,
    rows: &'static [(&'static str, GuideShortcut)],
}

// Actions resolve their key through Op::key, just like menus and tooltips.
// Pointer gestures and cursor keys follow EditorView's event handlers.
const COLUMNS: [&[GuideSection]; 2] = [
    &[
        GuideSection {
            title: "Tools",
            rows: &[
                ("Place", GuideShortcut::Action(Op::Place)),
                ("Erase", GuideShortcut::Action(Op::Erase)),
                ("Paint", GuideShortcut::Action(Op::Paint)),
                ("Fill", GuideShortcut::Action(Op::Fill)),
                ("Box", GuideShortcut::Action(Op::Box)),
                ("Color picker", GuideShortcut::Action(Op::Picker)),
                ("Apply at pointer", GuideShortcut::Gesture("Click")),
            ],
        },
        GuideSection {
            title: "Navigate",
            rows: &[
                ("Orbit", GuideShortcut::Gesture("Right-drag")),
                ("Pan", GuideShortcut::Gesture("Middle-drag")),
                ("Pan (alternate)", GuideShortcut::Keys(&["shift-rightdrag"])),
                ("Zoom", GuideShortcut::Gesture("Scroll")),
                ("Frame model · isometric", GuideShortcut::Action(Op::Frame)),
            ],
        },
    ],
    &[
        GuideSection {
            title: "Box & cursor",
            rows: &[
                (
                    "Set opposite corners",
                    GuideShortcut::Gesture("Click, then click"),
                ),
                ("Cancel box", GuideShortcut::Keys(&["escape"])),
                (
                    "Move cursor · X/Z",
                    GuideShortcut::Keys(&["left", "right", "up", "down"]),
                ),
                (
                    "Move cursor · Y",
                    GuideShortcut::Keys(&["pageup", "pagedown"]),
                ),
                ("Apply at cursor", GuideShortcut::Keys(&["enter"])),
            ],
        },
        GuideSection {
            title: "Colors",
            rows: &[
                ("Sample at pointer", GuideShortcut::Keys(&["alt-click"])),
                ("Previous / next color", GuideShortcut::Keys(&["[", "]"])),
            ],
        },
        GuideSection {
            title: "File & history",
            rows: &[
                ("Search commands", GuideShortcut::Action(Op::Commands)),
                ("Undo", GuideShortcut::Action(Op::Undo)),
                ("Redo", GuideShortcut::Action(Op::Redo)),
                ("Open project", GuideShortcut::Action(Op::Open)),
                ("Save", GuideShortcut::Action(Op::Save)),
                ("Save as…", GuideShortcut::Action(Op::SaveAs)),
            ],
        },
    ],
];

pub(super) fn keyboard_guide(theme: Theme) -> Div {
    row()
        .items_start()
        .gap(px(32.0))
        .children(COLUMNS.iter().map(|sections| {
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w(px(0.0))
                .gap(px(16.0))
                .children(sections.iter().map(|section| {
                    div()
                        .child(
                            div()
                                .mb(px(6.0))
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.muted())
                                .child(section.title),
                        )
                        .children(section.rows.iter().map(|(label, binding)| {
                            let keys = match binding {
                                GuideShortcut::Action(op) => action_shortcut(*op),
                                GuideShortcut::Keys(keys) => row()
                                    .gap(px(4.0))
                                    .children(keys.iter().map(|key| shortcut(key))),
                                GuideShortcut::Gesture(text) => {
                                    div().text_size(px(12.0)).child(*text)
                                }
                            };
                            row()
                                .h(px(28.0))
                                .justify_between()
                                .gap(px(12.0))
                                .whitespace_nowrap()
                                .child(div().child(*label))
                                .child(keys.flex_shrink_0().text_color(theme.muted()))
                        }))
                }))
        }))
}
