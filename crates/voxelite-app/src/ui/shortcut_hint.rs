//! Typography shared by every visual shortcut hint.
use crate::{
    actions::Op,
    shortcuts::{ShortcutPlatform, ShortcutSymbol, ShortcutToken, shortcut_tokens},
};
use gpui::{prelude::*, *};

#[derive(Clone, IntoElement)]
struct Shortcut {
    tokens: Vec<ShortcutToken>,
}

impl Shortcut {
    fn new(binding: &str) -> Self {
        Self {
            tokens: shortcut_tokens(binding, ShortcutPlatform::current()),
        }
    }
}

impl RenderOnce for Shortcut {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.pixel_snap(
            style
                .line_height
                .to_pixels(font_size.into(), window.rem_size()),
        );
        let font = gpui::font("Geist Mono");
        let font_id = cx.text_system().resolve_font(&font);
        let cap_height = cx.text_system().cap_height(font_id, font_size);
        // Use the same shaped-line metrics as GPUI's text painter. Some
        // backends report a signed font descent but a positive line descent.
        let mut run = style.to_run(1);
        run.font = font.clone();
        let metrics = window
            .text_system()
            .shape_line("H".into(), font_size, &[run], None);
        let baseline = (line_height - metrics.ascent - metrics.descent) / 2.0 + metrics.ascent;
        let label: String = self.tokens.iter().map(ShortcutToken::text).collect();
        let mut result = div()
            .id(SharedString::from(format!("shortcut-{label}")))
            .flex()
            .flex_shrink_0()
            .h(line_height)
            .font(font)
            .font_weight(FontWeight::NORMAL)
            .line_height(line_height)
            .role(Role::Label)
            .aria_label(label);
        let mut text = String::new();
        for token in self.tokens {
            match token {
                ShortcutToken::Text(part) => text.push_str(&part),
                ShortcutToken::Symbol(symbol) => {
                    if !text.is_empty() {
                        result = result.child(std::mem::take(&mut text));
                    }
                    let aspect = match symbol {
                        ShortcutSymbol::Backspace | ShortcutSymbol::Delete => 1.4,
                        ShortcutSymbol::Return | ShortcutSymbol::Tab | ShortcutSymbol::Space => 1.2,
                        _ => 1.0,
                    };
                    // Paths include their stroke within a 20-unit cap box. A
                    // 2-unit stroke is 0.852px at 12px, like Geist Mono Regular.
                    // The small trailing bearing keeps Mac chords compact.
                    result = result.child(
                        div()
                            .relative()
                            .flex_shrink_0()
                            .w(cap_height * aspect + font_size * 0.08)
                            .h(line_height)
                            .child(
                                svg()
                                    .path(symbol.path())
                                    .absolute()
                                    .top(baseline - cap_height)
                                    .w(cap_height * aspect)
                                    .h(cap_height)
                                    .text_color(style.color),
                            ),
                    );
                }
            }
        }
        if !text.is_empty() {
            result = result.child(text);
        }
        result
    }
}

pub(super) fn shortcut(binding: &str) -> Div {
    div().text_size(px(12.0)).child(Shortcut::new(binding))
}

pub(super) fn action_shortcut(op: Op) -> Div {
    div().when_some(op.key(), |d, key| d.child(shortcut(key)))
}

#[derive(Clone)]
enum HintPart {
    Text(SharedString),
    Key(Shortcut),
}

/// Rich prose keeps bindings structured even inside tooltips and help text.
#[derive(Clone, Default, IntoElement)]
pub(super) struct HintText(Vec<HintPart>);

impl From<&str> for HintText {
    fn from(text: &str) -> Self {
        Self::default().text(text.to_owned())
    }
}

impl From<String> for HintText {
    fn from(text: String) -> Self {
        Self::default().text(text)
    }
}

impl HintText {
    pub(super) fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.0.push(HintPart::Text(text.into()));
        self
    }

    pub(super) fn key(mut self, binding: &str) -> Self {
        self.0.push(HintPart::Key(Shortcut::new(binding)));
        self
    }

    pub(super) fn action(self, op: Op) -> Self {
        if let Some(key) = op.key() {
            self.key(key)
        } else {
            self
        }
    }
}

impl RenderOnce for HintText {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // Preserve normal text wrapping/ellipsis for messages without keys.
        if let [HintPart::Text(text)] = self.0.as_slice() {
            return text.clone().into_any_element();
        }
        let mut result = div().flex().flex_col();
        let mut line = div().flex().flex_wrap();
        for part in self.0 {
            match part {
                HintPart::Key(key) => line = line.child(key),
                HintPart::Text(text) => {
                    for (index, paragraph) in text.split('\n').enumerate() {
                        if index > 0 {
                            result = result.child(line);
                            line = div().flex().flex_wrap();
                        }
                        // Keep prose wrapping at word boundaries while a key
                        // chord remains an indivisible inline item.
                        for word in paragraph.split_inclusive(' ') {
                            line = line.child(SharedString::from(word.to_owned()));
                        }
                    }
                }
            }
        }
        result.child(line).into_any_element()
    }
}
