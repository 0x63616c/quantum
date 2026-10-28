//! Typography shared by every visual shortcut hint.
use crate::{
    actions::Op,
    shortcuts::{ShortcutPlatform, ShortcutSymbol, ShortcutToken, shortcut_tokens},
};
use gpui::{prelude::*, *};
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;
use quantum_ui::tokens::metrics;
use quantum_ui::tokens::type_size;

#[derive(Clone, IntoElement)]
struct Keycap {
    tokens: Vec<ShortcutToken>,
}

impl Keycap {
    fn new(binding: &str) -> Self {
        let platform = ShortcutPlatform::current();
        // Headless Linux captures can exercise the Mac typography without
        // changing real key bindings. This hook is absent from release builds.
        #[cfg(debug_assertions)]
        let platform = if std::env::var_os("QUANTUM_CAPTURE_MAC_UI").is_some() {
            ShortcutPlatform::Apple
        } else {
            platform
        };
        Self {
            tokens: shortcut_tokens(binding, platform),
        }
    }
}

impl RenderOnce for Keycap {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = window.text_style();
        let font_size = px(Space::S4.px());
        let line_height = px(Space::S5.px());
        let font = gpui::font("Geist");
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
            .text_size(font_size)
            .font_weight(type_size::NORMAL)
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
                    // The SVG ink fills its viewBox. Match the UI letter cap
                    // height and leave a single logical pixel between symbols.
                    let symbol_height = window.pixel_snap(cap_height);
                    result = result.child(
                        div()
                            .relative()
                            .flex_shrink_0()
                            .w(symbol_height * aspect + px(metrics::EDGE))
                            .h(line_height)
                            .child(
                                svg()
                                    .path(symbol.path())
                                    .absolute()
                                    .top(window.pixel_snap(baseline - symbol_height))
                                    .w(symbol_height * aspect)
                                    .h(symbol_height)
                                    .text_color(style.color),
                            ),
                    );
                }
            }
        }
        if !text.is_empty() {
            result = result.child(text);
        }
        quantum_ui::Kbd::frame(style.color).child(result)
    }
}

pub(super) fn shortcut(binding: &str) -> Div {
    div()
        .text_style(TextSize::Small)
        .child(Keycap::new(binding))
}

pub(super) fn action_shortcut(op: Op) -> Div {
    div().when_some(op.key(), |d, key| d.child(shortcut(key)))
}

pub(super) fn sheet_shortcut(op: Op) -> Div {
    if op.key().is_some() {
        action_shortcut(op)
    } else {
        div().child("—")
    }
}

#[derive(Clone)]
enum HintPart {
    Text(SharedString),
    Key(Keycap),
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
    pub(super) fn tooltip_parts(self) -> (String, Option<String>) {
        let mut label = String::new();
        let mut keys = Vec::new();
        for part in self.0 {
            match part {
                HintPart::Text(text) => label.push_str(&text),
                HintPart::Key(key) => keys.push(
                    key.tokens
                        .iter()
                        .map(ShortcutToken::text)
                        .collect::<String>(),
                ),
            }
        }
        (label, (!keys.is_empty()).then(|| keys.join(" ")))
    }

    pub(super) fn plain_text(&self) -> Option<&str> {
        match self.0.as_slice() {
            [HintPart::Text(text)] => Some(text.as_ref()),
            _ => None,
        }
    }

    pub(super) fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.0.push(HintPart::Text(text.into()));
        self
    }

    pub(super) fn key(mut self, binding: &str) -> Self {
        self.0.push(HintPart::Key(Keycap::new(binding)));
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
        let new_line = || {
            div()
                .flex()
                .flex_nowrap()
                .whitespace_nowrap()
                .flex_none()
                .items_center()
                .gap_x(px(Space::S3.px()))
        };
        let mut result = div().flex().flex_col().flex_none().items_start();
        let mut line = new_line();
        for part in self.0 {
            match part {
                HintPart::Key(key) => line = line.child(key),
                HintPart::Text(text) => {
                    for (index, paragraph) in text.split('\n').enumerate() {
                        if index > 0 {
                            result = result.child(line);
                            line = new_line();
                        }
                        // Shape prose as a whole: spacing around keycaps comes
                        // only from the row gap, never spaces or per-word gaps.
                        let paragraph = paragraph.trim();
                        if !paragraph.is_empty() {
                            line = line.child(
                                div()
                                    .flex_none()
                                    .child(SharedString::from(paragraph.to_owned())),
                            );
                        }
                    }
                }
            }
        }
        result.child(line).into_any_element()
    }
}
