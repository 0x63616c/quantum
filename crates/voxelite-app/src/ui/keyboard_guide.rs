//! Compact, structured reference for the editor's keyboard and mouse bindings.
use super::{action_shortcut, row, shortcut};
use crate::{
    actions::{GuideShortcut, keyboard_guide_sections},
    shortcuts::ShortcutPlatform,
    theme::Theme,
};
use gpui::{prelude::*, *};

pub(super) fn keyboard_guide(theme: Theme) -> Div {
    let columns = keyboard_guide_sections(ShortcutPlatform::current());
    row()
        .items_start()
        .gap(px(32.0))
        .children(columns.iter().map(|sections| {
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
