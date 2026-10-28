//! Small About and acknowledgements panels built from the shared dialog shell.
use super::*;
use voxelite_ui::Card;
use voxelite_ui::tokens::ControlSize;

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
            include_str!("../../../voxelite-ui/NOTICE"),
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
        // Preserve the original content-width compact action (29 × 24).
        let close = Button::new("close-about", "Close About Quantum", t)
            .variant(ButtonVariant::Icon)
            .icon("close")
            .size(ControlSize::Sm)
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
                        .min_h(px(space::NONE))
                        .overflow_y_scroll()
                        .p(px(space::XL))
                        .child(
                            Card::new(t).rows(licenses().into_iter().map(|(name, license)| {
                                div()
                                    .p(px(space::LG))
                                    .child(div().font_weight(type_size::SEMIBOLD).child(name))
                                    .child(
                                        div()
                                            .mt(px(space::XS))
                                            .text_size(px(type_size::CAPTION))
                                            .line_height(px(type_size::LINE_SM))
                                            .text_color(t.muted())
                                            .child(attribution(license)),
                                    )
                            })),
                        ),
                )
                .child(
                    Dialog::footer(t).child(
                        Button::new("about-back", "Back", t)
                            .variant(ButtonVariant::Ghost)
                            .build()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.info = Some(("About Quantum", InfoContent::About));
                                window.focus(&this.modal_focus, cx);
                                cx.notify();
                            })),
                    ),
                )
        } else {
            Dialog::panel("about-voxelite", "About Quantum", t, &self.modal_focus)
                .w(px(metrics::ABOUT_WIDTH))
                .h(px(metrics::ABOUT_HEIGHT))
                .child(row().justify_end().p(px(space::MD)).child(close))
                .child(
                    div()
                        .flex_1()
                        .min_h(px(space::NONE))
                        .flex()
                        .flex_col()
                        .items_center()
                        .px(px(space::XL))
                        .gap(px(space::LG))
                        .child(
                            img("app-icon-256.png")
                                .size(px(metrics::APP_ICON))
                                .flex_shrink_0(),
                        )
                        .child(
                            div()
                                .text_size(px(type_size::DISPLAY))
                                .font_weight(type_size::BOLD)
                                .child("Quantum"),
                        )
                        .child(
                            div()
                                .text_size(px(type_size::BODY))
                                .text_color(t.muted())
                                .text_center()
                                .child(version_label(
                                    env!("CARGO_PKG_VERSION"),
                                    option_env!("VOXELITE_BUILD"),
                                )),
                        )
                        .child(
                            div()
                                .text_size(px(type_size::BODY))
                                .text_center()
                                .child(TAGLINE),
                        )
                        .child(
                            div()
                                .text_size(px(type_size::CAPTION))
                                .text_color(t.muted())
                                .child("© 2026 Calum Webb"),
                        ),
                )
                .child(
                    row()
                        .justify_center()
                        .gap(px(space::XS))
                        .px(px(space::MD))
                        .pb(px(space::XXL))
                        .children(
                            [
                                (
                                    "about-website",
                                    "Website",
                                    "https://voxelite.worldwidewebb.co",
                                ),
                                (
                                    "about-github",
                                    "GitHub",
                                    "https://github.com/0x63616c/voxelite",
                                ),
                            ]
                            .map(|(id, label, url)| {
                                Button::new(id, label, t)
                                    .variant(ButtonVariant::Ghost)
                                    .size(ControlSize::Sm)
                                    .build()
                                    .px(px(space::SM))
                                    .on_click(move |_, _, cx| cx.open_url(url))
                            }),
                        )
                        .child(
                            Button::new("about-licenses", "Acknowledgements", t)
                                .variant(ButtonVariant::Ghost)
                                .size(ControlSize::Sm)
                                .build()
                                .px(px(space::SM))
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
