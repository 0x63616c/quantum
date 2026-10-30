//! Compact, structured reference for the editor's keyboard and mouse bindings.
use super::{action_shortcut, row, shortcut};
use crate::{
    actions::{GuideShortcut, keyboard_guide_sections},
    shortcuts::ShortcutPlatform,
    theme::Theme,
};
use gpui::{prelude::*, *};
use quantum_ui::tokens::{space, type_size};

pub(super) fn keyboard_guide(theme: Theme) -> Div {
    let columns = keyboard_guide_sections(ShortcutPlatform::current());
    row()
        .items_start()
        .gap(px(space::XXXL))
        .children(columns.iter().map(|sections| {
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w(px(space::NONE))
                .gap(px(space::XL))
                .children(sections.iter().map(|section| {
                    div()
                        .child(
                            div()
                                .mb(px(space::SM))
                                .text_size(px(type_size::BODY))
                                .line_height(px(type_size::LINE_SM))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.muted())
                                .child(section.title),
                        )
                        .children(section.rows.iter().map(|(label, binding)| {
                            let keys = match binding {
                                GuideShortcut::Action(op) => action_shortcut(*op),
                                GuideShortcut::Keys(keys) => row()
                                    .gap(px(space::XS))
                                    .children(keys.iter().map(|key| shortcut(key))),
                                GuideShortcut::Gesture(text) => {
                                    div().text_size(px(type_size::BODY)).child(*text)
                                }
                            };
                            row()
                                .h(px(space::PANE))
                                .justify_between()
                                .gap(px(space::LG))
                                .whitespace_nowrap()
                                .child(div().child(*label))
                                .child(keys.flex_shrink_0().text_color(theme.muted()))
                        }))
                }))
        }))
}
