//! Interactive design-system gallery.
//! Optional: `--section buttons|rows|navigation|menus|fields|feedback|tokens`.
#[path = "ui_kit/assets.rs"]
mod assets;
#[path = "ui_kit/sections.rs"]
mod sections;
use gpui::{prelude::*, *};
use quantum_ui::*;
use quantum_ui::{Menu, MenuItem};
use std::{cell::Cell, rc::Rc};

const DIALOG_DESCRIPTION: &str = "The dialog always provides Cancel and Escape. \
    Destructive actions sit at the left; the primary action sits at the right.";
const FIELDS_HEADING: &str = "Native fields: edit, select, undo, paste, validate";
const FIELDS_FOCUS_HINT: &str = "Tab into the field to inspect keyboard-only focus";

struct Gallery {
    overlays: Entity<OverlayState>,
    fields: Vec<Entity<TextField>>,
    focus: FocusHandle,
    menu: Option<Theme>,
    dialog: Option<Theme>,
    section: Option<String>,
    bounds: [Rc<Cell<Bounds<Pixels>>>; 2],
}

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut fields = Vec::new();
        for theme in [Theme::Dark, Theme::Light] {
            for error in [None, Some("Enter a project name".into())] {
                let field = TextField::design("Project name", cx);
                field.update(cx, |field, cx| {
                    field.appearance(theme, cx);
                    field.error(error, cx);
                });
                fields.push(field);
            }
        }
        let args: Vec<_> = std::env::args().collect();
        let section = args
            .windows(2)
            .find(|a| a[0] == "--section")
            .map(|a| a[1].clone());
        Self {
            overlays: OverlayState::new(cx),
            fields,
            focus: cx.focus_handle(),
            menu: None,
            dialog: None,
            section,
            bounds: Default::default(),
        }
    }

    fn set_overlay(
        &mut self,
        id: &'static str,
        layer: OverlayLayer,
        open: bool,
        cx: &mut Context<Self>,
    ) {
        self.overlays
            .update(cx, |state, cx| state.set_open(id, layer, open, cx));
        cx.notify();
    }

    fn live(&self, t: Theme, index: usize, cx: &Context<Self>) -> Div {
        let measure = self.bounds[index].clone();
        row()
            .gap(px(Space::S3.px()))
            .child(
                Button::design("live-menu", "Open popover", t)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.menu = if this.menu == Some(t) { None } else { Some(t) };
                        this.set_overlay(
                            "gallery-menu",
                            OverlayLayer::Popover,
                            this.menu.is_some(),
                            cx,
                        );
                    }))
                    .build()
                    .relative()
                    .child(
                        canvas(move |b, _, _| measure.set(b), |_, _, _, _| {})
                            .absolute()
                            .size_full(),
                    ),
            )
            .child(
                Button::design("live-dialog", "Open dialog", t)
                    .style(ButtonStyle::Primary)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.dialog = Some(t);
                        this.menu = None;
                        this.set_overlay("gallery-menu", OverlayLayer::Popover, false, cx);
                        this.set_overlay("gallery-dialog", OverlayLayer::Dialog, true, cx);
                        window.focus(&this.focus, cx);
                    })),
            )
            .child(IconButton::design(
                "live-tooltip",
                "copy",
                "Tooltip: hover, then press to dismiss",
                t,
                &self.overlays,
            ))
    }

    fn live_menu(&self, t: Theme, cx: &Context<Self>) -> Deferred {
        let index = usize::from(t == Theme::Light);
        let dismiss = MenuItem::new("dismiss-menu", "Close popover", t).on_click(cx.listener(
            |this, _, _, cx| {
                this.menu = None;
                this.set_overlay("gallery-menu", OverlayLayer::Popover, false, cx);
            },
        ));
        Popover::anchored(
            self.bounds[index].get(),
            PopoverSide::Below,
            Menu::design("live-menu-panel", t).child(dismiss),
        )
    }

    fn live_dialog(&self, t: Theme, cx: &Context<Self>) -> impl IntoElement {
        let cancel = cx.entity().downgrade();
        Dialog::design(
            "live-dialog-panel",
            "Save project?",
            t,
            &self.focus,
            move |_, cx| {
                let _ = cancel.update(cx, |this, cx| {
                    this.dialog = None;
                    this.set_overlay("gallery-dialog", OverlayLayer::Dialog, false, cx);
                });
            },
        )
        .body(DIALOG_DESCRIPTION)
        .danger(Button::design("discard", "Discard", t))
        .primary(Button::design("save", "Save", t))
    }

    fn fields_panel(&self, t: Theme, index: usize) -> Div {
        Panel::design(t)
            .header(FIELDS_HEADING)
            .body(
                "fields",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(Space::S4.px()))
                    .child(self.fields[index * 2].clone())
                    .child(self.fields[index * 2 + 1].clone()),
            )
            .footer(InlineError::message(
                FIELDS_FOCUS_HINT,
                FeedbackKind::Info,
                t,
            ))
            .build()
    }

    fn column(&self, t: Theme, index: usize, cx: &Context<Self>) -> Stateful<Div> {
        let visible = |section: &str| {
            self.section
                .as_deref()
                .is_none_or(|filter| filter == section)
        };
        TextSize::Body
            .apply(div())
            .id(("theme", index))
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(Space::S0.px()))
            .gap(px(Space::S6.px()))
            .p(px(Space::S6.px()))
            .bg(t.color(ColorToken::Surface))
            .text_color(t.color(ColorToken::TextPrimary))
            .child(
                TextSize::Heading
                    .apply(div())
                    .child(format!("Quantum UI kit · {t:?}")),
            )
            .child(self.live(t, index, cx))
            .when(visible("tokens"), |d| d.child(sections::tokens(t)))
            .when(visible("buttons"), |d| {
                d.child(sections::buttons(t, &self.overlays))
            })
            .when(visible("rows"), |d| {
                d.child(sections::rows(t, &self.overlays))
            })
            .when(visible("navigation"), |d| d.child(sections::navigation(t)))
            .when(visible("menus"), |d| {
                d.child(sections::menus(t, &self.overlays))
            })
            .when(visible("fields"), |d| d.child(self.fields_panel(t, index)))
            .when(visible("feedback"), |d| d.child(sections::feedback(t)))
    }
}

impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("gallery")
            .size_full()
            .overflow_y_scroll()
            .child(
                row()
                    .items_stretch()
                    .min_h_full()
                    .flex_shrink_0()
                    .child(self.column(Theme::Dark, 0, cx))
                    .child(self.column(Theme::Light, 1, cx)),
            )
            .when_some(self.menu, |d, t| d.child(self.live_menu(t, cx)))
            .when_some(self.dialog, |d, t| d.child(self.live_dialog(t, cx)))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(assets::Assets)
        .run(|cx| {
            if let Err(error) = cx.text_system().add_fonts(assets::fonts()) {
                eprintln!("Cannot load gallery fonts: {error}");
                cx.quit();
                return;
            }
            cx.set_text_rendering_mode(TextRenderingMode::Grayscale);
            let bounds = Bounds::centered(None, size(px(1440.), px(1000.)), cx);
            if let Err(error) = cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Quantum UI kit".into()),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(Gallery::new),
            ) {
                eprintln!("Cannot open gallery: {error}");
                cx.quit();
            }
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            cx.activate(true);
        });
}
