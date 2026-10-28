//! Native close-up of the four ASCII presets. Space replays the loading cycle;
//! `--light` and `--reduced-motion` exercise the other presentation modes.
use gpui::{prelude::*, *};
use std::time::{Duration, Instant};
use voxelite_ui::{
    AsciiReveal, AsciiSkeleton, Panel, Theme,
    ascii_reveal::{presets::Preset, skeleton::SkeletonVariant},
    tokens::{space, type_size},
};

struct Demo {
    theme: Theme,
    reduced: bool,
    loading: bool,
    start: Instant,
    task: Option<Task<()>>,
    focus: FocusHandle,
}

impl Demo {
    fn replay(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        self.start = Instant::now();
        self.task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(2400))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.loading = false;
                this.start = Instant::now();
                cx.notify();
            });
        }));
        cx.notify();
    }
}

impl Render for Demo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        div()
            .size_full()
            .bg(t.background())
            .text_color(t.text())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY))
            .p(px(space::PANEL))
            .flex()
            .flex_col()
            .gap(px(space::PANEL))
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "space" {
                    this.replay(cx);
                }
            }))
            .child(
                div()
                    .text_size(px(type_size::DISPLAY))
                    .child("Quantum / ASCII loading"),
            )
            .child(
                div()
                    .text_color(t.muted())
                    .child("Native GPUI · ascii-reveal by Quan Hoang · Space to replay"),
            )
            .children(
                [
                    [
                        ("Decode", Preset::Decode, SkeletonVariant::Scan),
                        ("Dither", Preset::Dither, SkeletonVariant::Dither),
                    ],
                    [
                        ("Terminal", Preset::Terminal, SkeletonVariant::Terminal),
                        ("Blueprint", Preset::Blueprint, SkeletonVariant::Braille),
                    ],
                ]
                .into_iter()
                .enumerate()
                .map(|(row, presets)| {
                    div().flex().flex_1().gap(px(space::PANEL)).children(
                        presets
                            .into_iter()
                            .enumerate()
                            .map(|(col, (name, preset, variant))| {
                                let id = row * 2 + col;
                                Panel::section(t)
                                    .flex_1()
                                    .p(px(space::PANEL))
                                    .flex()
                                    .flex_col()
                                    .gap(px(space::XL))
                                    .child(div().text_size(px(type_size::TITLE)).child(name))
                                    .child(
                                        AsciiSkeleton::new(("block", id), t)
                                            .rows(5)
                                            .variant(variant)
                                            .animate(self.loading)
                                            .reduced_motion(self.reduced)
                                            .started_at(self.start),
                                    )
                                    .child(
                                        AsciiSkeleton::new(("lines", id), t)
                                            .lines(2)
                                            .variant(variant)
                                            .animate(self.loading)
                                            .reduced_motion(self.reduced)
                                            .started_at(self.start),
                                    )
                                    .child(
                                        AsciiReveal::new(
                                            ("label", id),
                                            "Highland Study · 1,040,000 voxels",
                                            t,
                                        )
                                        .preset(preset)
                                        .skeleton_variant(variant)
                                        .loading(self.loading)
                                        .reduced_motion(self.reduced),
                                    )
                            }),
                    )
                }),
            )
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let theme = if args.iter().any(|s| s == "--light") {
        Theme::Light
    } else {
        Theme::Dark
    };
    let reduced = args.iter().any(|s| s == "--reduced-motion")
        || voxelite_app::reduce_motion::prefers_reduced_motion();
    gpui_platform::application().run(move |cx| {
        cx.text_system()
            .add_fonts(vec![
                std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/Geist.ttf").as_slice()),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../assets/fonts/GeistMono.ttf").as_slice(),
                ),
            ])
            .expect("bundled fonts");
        cx.set_text_rendering_mode(TextRenderingMode::Grayscale);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1120.), px(760.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Quantum ASCII presets".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    let mut demo = Demo {
                        theme,
                        reduced,
                        loading: true,
                        start: Instant::now(),
                        task: None,
                        focus,
                    };
                    demo.replay(cx);
                    demo
                })
            },
        )
        .expect("open gallery");
        cx.activate(true);
    });
}
