//! Compact, structured reference for the editor's keyboard and mouse bindings.
use super::{action_shortcut, row, shortcut};
use crate::{
    actions::{GuideShortcut, keyboard_guide_sections},
    shortcuts::ShortcutPlatform,
    theme::Theme,
};
use gpui::{prelude::*, *};
use quantum_ui::ColorToken;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;
use quantum_ui::tokens::type_size;

pub(super) fn keyboard_guide(theme: Theme) -> Div {
    let columns = keyboard_guide_sections(ShortcutPlatform::current());
    row()
        .items_start()
        .gap(px(Space::S7.px()))
        .children(columns.iter().map(|sections| {
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w(px(Space::S0.px()))
                .gap(px(Space::S5.px()))
                .children(sections.iter().map(|section| {
                    div()
                        .child(
                            div()
                                .mb(px(Space::S3.px()))
                                .text_style(TextSize::Small)
                                .font_weight(type_size::SEMIBOLD)
                                .text_color(theme.color(ColorToken::TextSecondary))
                                .child(section.title),
                        )
                        .children(section.rows.iter().map(|(label, binding)| {
                            let keys = match binding {
                                GuideShortcut::Action(op) => action_shortcut(*op),
                                GuideShortcut::Keys(keys) => row()
                                    .gap(px(Space::S2.px()))
                                    .children(keys.iter().map(|key| shortcut(key))),
                                GuideShortcut::Gesture(text) => {
                                    div().text_style(TextSize::Small).child(*text)
                                }
                            };
                            row()
                                .h(px(Space::S7.px()))
                                .justify_between()
                                .gap(px(Space::S4.px()))
                                .whitespace_nowrap()
                                .child(div().child(*label))
                                .child(
                                    keys.flex_shrink_0()
                                        .text_color(theme.color(ColorToken::TextSecondary)),
                                )
                        }))
                }))
        }))
}
