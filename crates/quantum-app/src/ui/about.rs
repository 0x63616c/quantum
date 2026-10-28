//! Small About and acknowledgements panels built from the shared dialog shell.
use super::*;
use quantum_ui::Card;
use quantum_ui::ColorToken;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;

const TAGLINE: &str = "A voxel workspace by Calum";

fn version_label(version: &str, build: Option<&str>) -> String {
    match build.filter(|value| !value.trim().is_empty()) {
        Some(build) => format!("Version {version} (build {build})"),
        None => format!("Version {version}"),
    }
}

// Summaries are extracted from the bundled notices rather than retyping upstream
// attribution. The complete original license text remains embedded in the binary.
fn attribution(license: &str) -> String {
    license
        .lines()
        .filter(|line| {
            line.trim_start()
                .strip_prefix("Copyright ")
                .is_some_and(|rest| {
                    rest.starts_with("(c)")
                        || rest.chars().next().is_some_and(|ch| ch.is_ascii_digit())
                })
        })
        .take(2)
        .collect::<Vec<_>>()
        .join("\n")
}

fn licenses() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "ascii-reveal · MIT",
            include_str!("../../../quantum-ui/NOTICE"),
        ),
        (
            "Geist · SIL Open Font License 1.1",
            include_str!("../../assets/fonts/OFL-Geist.txt"),
        ),
        (
            "Geist Mono · SIL Open Font License 1.1",
            include_str!("../../assets/fonts/OFL-GeistMono.txt"),
        ),
        (
            "Lucide icons · ISC",
            include_str!("../../assets/icons/LICENSE"),
        ),
        (
            "Feather icons · MIT",
            include_str!("../../assets/icons/LICENSE-Feather"),
        ),
        (
            "GPUI / Zed · Apache-2.0",
            include_str!("../../../../vendor/gpui-apple/LICENSE-APACHE"),
        ),
        #[cfg(target_os = "macos")]
        (
            "Sparkle · MIT (macOS)",
            include_str!("../../assets/licenses/Sparkle-LICENSE.txt"),
        ),
    ]
}

impl EditorView {
    fn close_about(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.info = None;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub(super) fn about_dialog(&self, acknowledgements: bool, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        // Close uses the same square geometry as other compact icon actions.
        let close = IconButton::new("close-about", "close", "Close About Quantum", t)
            .height(quantum_ui::ControlHeight::Dense)
            .build()
            .on_click(cx.listener(|this, _, window, cx| this.close_about(window, cx)));
        let panel = if acknowledgements {
            Dialog::panel("acknowledgements", "Acknowledgements", t, &self.modal_focus)
                .w(px(metrics::CONFIRM_WIDTH))
                .h(px(metrics::COMMAND_HEIGHT))
                .child(Dialog::title("Acknowledgements", t).child(close))
                .child(
                    div()
                        .id("license-list")
                        .flex_1()
                        .min_h(px(Space::S0.px()))
                        .overflow_y_scroll()
                        .p(px(Space::S5.px()))
                        .child(
                            Card::new(t).rows(licenses().into_iter().map(|(name, license)| {
                                div()
                                    .p(px(Space::S4.px()))
                                    .child(div().font_weight(type_size::SEMIBOLD).child(name))
                                    .child(
                                        div()
                                            .mt(px(Space::S2.px()))
                                            .text_style(TextSize::Caption)
                                            .text_color(t.color(ColorToken::TextSecondary))
                                            .child(attribution(license)),
                                    )
                            })),
                        ),
                )
                .child(
                    Dialog::footer(t).child(
                        Button::design("about-back", "Back", t)
                            .style(quantum_ui::ButtonStyle::Ghost)
                            .build()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.info = Some(("About Quantum", InfoContent::About));
                                window.focus(&this.modal_focus, cx);
                                cx.notify();
                            })),
                    ),
                )
        } else {
            Dialog::panel("about-quantum", "About Quantum", t, &self.modal_focus)
                .w(px(metrics::ABOUT_WIDTH))
                .h(px(metrics::ABOUT_HEIGHT))
                .child(row().justify_end().p(px(Space::S3.px())).child(close))
                .child(
                    div()
                        .flex_1()
                        .min_h(px(Space::S0.px()))
                        .flex()
                        .flex_col()
                        .items_center()
                        .px(px(Space::S5.px()))
                        .gap(px(Space::S4.px()))
                        .child(
                            img("app-icon-256.png")
                                .size(px(metrics::APP_ICON))
                                .flex_shrink_0(),
                        )
                        .child(
                            div()
                                .text_style(TextSize::Display)
                                .font_weight(type_size::BOLD)
                                .child("Quantum"),
                        )
                        .child(
                            div()
                                .text_style(TextSize::Small)
                                .text_color(t.color(ColorToken::TextSecondary))
                                .text_center()
                                .child(version_label(crate::ui::APP_VERSION, crate::ui::APP_BUILD)),
                        )
                        .child(
                            div()
                                .text_style(TextSize::Small)
                                .text_center()
                                .child(TAGLINE),
                        )
                        .child(
                            div()
                                .text_style(TextSize::Caption)
                                .text_color(t.color(ColorToken::TextSecondary))
                                .child("© 2026 Calum Webb"),
                        ),
                )
                .child(
                    row()
                        .justify_center()
                        .gap(px(Space::S2.px()))
                        .px(px(Space::S3.px()))
                        .pb(px(Space::S6.px()))
                        .children(
                            [
                                (
                                    "about-website",
                                    "Website",
                                    "https://quantum.worldwidewebb.co",
                                ),
                                (
                                    "about-github",
                                    "GitHub",
                                    "https://github.com/0x63616c/quantum",
                                ),
                            ]
                            .map(|(id, label, url)| {
                                Button::design(id, label, t)
                                    .style(quantum_ui::ButtonStyle::Ghost)
                                    .height(quantum_ui::ControlHeight::Dense)
                                    .build()
                                    .px(px(Space::S3.px()))
                                    .on_click(move |_, _, cx| cx.open_url(url))
                            }),
                        )
                        .child(
                            Button::design("about-licenses", "Acknowledgements", t)
                                .style(quantum_ui::ButtonStyle::Ghost)
                                .height(quantum_ui::ControlHeight::Dense)
                                .build()
                                .px(px(Space::S3.px()))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.info =
                                        Some(("Acknowledgements", InfoContent::Acknowledgements));
                                    window.focus(&this.modal_focus, cx);
                                    cx.notify();
                                })),
                        ),
                )
        };
        Dialog::scrim(t, true).child(panel).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{attribution, licenses, version_label};
    #[test]
    fn build_label_never_invents_a_number() {
        assert_eq!(version_label("0.2.0", None), "Version 0.2.0");
        assert_eq!(version_label("0.2.0", Some(" ")), "Version 0.2.0");
        assert_eq!(
            version_label("0.2.0", Some("abc1234")),
            "Version 0.2.0 (build abc1234)"
        );
    }
    #[test]
    fn attribution_excludes_license_clauses_and_templates() {
        assert_eq!(
            attribution(
                "Copyright 2024 The Authors\nCopyright Holder. This restriction...\nCopyright [yyyy] [name]"
            ),
            "Copyright 2024 The Authors"
        );
    }
    #[test]
    fn every_bundled_notice_has_attribution() {
        for (name, notice) in licenses() {
            assert!(!attribution(notice).is_empty(), "{name}");
        }
    }
}
