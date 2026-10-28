use gpui::{prelude::*, *};
use std::time::Duration;
use voxelite_ui::*;
use voxelite_ui::{Menu, MenuItem};

const ROW_TITLE: &str = "A long row title that truncates without moving trailing actions or \
    clipping the final word, even when the document name is much longer than the available space";
const APPEARANCE_DESCRIPTION: &str =
    "The description wraps while the trailing control retains its size.";
const BUTTON_STATES: &str = "Rest · Hover · Press · Keyboard focus · Disabled · Loading";

pub const STATES: [ControlState; 6] = [
    ControlState::Rest,
    ControlState::Hovered,
    ControlState::Pressed,
    ControlState::Focused,
    ControlState::Disabled,
    ControlState::Loading,
];

pub const HEIGHTS: [ControlHeight; 3] = [
    ControlHeight::Dense,
    ControlHeight::Default,
    ControlHeight::Large,
];

fn stack() -> Div {
    div().flex().flex_col().gap(px(Space::S4.px()))
}

fn line() -> Div {
    row().gap(px(Space::S3.px())).flex_wrap()
}

fn caption(label: impl Into<SharedString>, t: Theme) -> Div {
    SectionLabel::label(label, t)
}

pub fn buttons(t: Theme, overlays: &Entity<OverlayState>) -> Div {
    stack()
        .child(caption(BUTTON_STATES, t))
        .children(
            [
                ButtonStyle::Primary,
                ButtonStyle::Secondary,
                ButtonStyle::Ghost,
                ButtonStyle::Danger,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, style)| button_style(t, i, style)),
        )
        .child(caption("IconButton: dense / default, ghost / selected", t))
        .children(
            [ControlHeight::Dense, ControlHeight::Default]
                .into_iter()
                .enumerate()
                .flat_map(|(i, height)| {
                    [false, true]
                        .into_iter()
                        .enumerate()
                        .map(move |(j, selected)| {
                            icon_button_states(t, overlays, i * 100 + j * 10, height, selected)
                        })
                }),
        )
}

fn button_style(t: Theme, index: usize, style: ButtonStyle) -> Div {
    stack().child(caption(format!("{style:?}"), t)).children(
        HEIGHTS
            .into_iter()
            .enumerate()
            .map(|(i, height)| button_states(t, index * 100 + i * 10, style, height)),
    )
}

fn button_states(t: Theme, id_offset: usize, style: ButtonStyle, height: ControlHeight) -> Div {
    line().children(STATES.into_iter().enumerate().map(|(i, state)| {
        Button::design(("action", id_offset + i), "Action", t)
            .style(style)
            .height(height)
            .state(state)
    }))
}

fn icon_button_states(
    t: Theme,
    overlays: &Entity<OverlayState>,
    id_offset: usize,
    height: ControlHeight,
    selected: bool,
) -> Div {
    line().children(STATES.into_iter().enumerate().map(|(i, state)| {
        IconButton::design(
            ("icon", id_offset + i),
            "copy",
            "Copy configuration",
            t,
            overlays,
        )
        .height(height)
        .state(state)
        .selected(selected)
    }))
}

pub fn rows(t: Theme, overlays: &Entity<OverlayState>) -> Div {
    Panel::design(t)
        .header("Panel title")
        .actions(IconButton::design(
            "panel-more",
            "ellipsis",
            "More actions",
            t,
            overlays,
        ))
        .body("panel-scroll", row_examples(t))
        .footer(
            StatusBarItem::new("Voxels", Some(1234), 6, t)
                .separator(false)
                .build(),
        )
        .build()
}

fn row_examples(t: Theme) -> Div {
    stack()
        .children(
            STATES
                .into_iter()
                .enumerate()
                .map(|(i, state)| list_row(t, i, state)),
        )
        .child(caption("Settings group", t))
        .child(SettingsRow::row(
            "Appearance",
            APPEARANCE_DESCRIPTION,
            Button::design("appearance", "System", t),
            t,
        ))
        .child(
            EmptyState::new("folder-open", "No projects yet", t)
                .primary(Button::design("new", "New project", t).style(ButtonStyle::Primary))
                .secondary(Button::design("open", "Open…", t)),
        )
}

fn list_row(t: Theme, index: usize, state: ControlState) -> ListRow {
    let selected = index.is_multiple_of(2);
    let selection = if selected { "selected" } else { "unselected" };
    ListRow::new(("row", index), ROW_TITLE, t)
        .icon("layer")
        .meta(format!("{state:?} · {selection}"))
        .selected(selected)
        .navigation(true)
        .state(state)
        .trailing(Button::design(("row-action", index), "Edit", t).style(ButtonStyle::Ghost))
}

pub fn navigation(t: Theme) -> Div {
    stack()
        .child(caption(
            "Tabs: underline + selected fill; Segments: fill only",
            t,
        ))
        .children([false, true].into_iter().enumerate().map(|(j, selected)| {
            stack()
                .child(caption(if selected { "Selected" } else { "Unselected" }, t))
                .child(
                    Tabs::design(("tabs", j), t).children(STATES.into_iter().enumerate().map(
                        move |(i, state)| {
                            Tab::design(("tab", j * 10 + i), format!("{state:?}"), t)
                                .selected(selected)
                                .state(state)
                        },
                    )),
                )
                .child(
                    SegmentedControl::design(t).children(STATES.into_iter().enumerate().map(
                        move |(i, state)| {
                            Segment::design(("segment", j * 10 + i), format!("{state:?}"), t)
                                .selected(selected)
                                .state(state)
                        },
                    )),
                )
        }))
}

pub fn menus(t: Theme, overlays: &Entity<OverlayState>) -> Div {
    stack()
        .child(
            line().child(menu_items(t)).child(
                Popover::design(t).child(
                    TextSize::Body
                        .apply(div())
                        .p(px(Space::S5.px()))
                        .child("Popover surface"),
                ),
            ),
        )
        .child(
            Tooltip::new("Copy configuration", t, overlays)
                .shortcut("⌘C")
                .build(),
        )
        .child(line().children(["Esc", "⌘K", "⇧⌘S"].map(|key| Kbd::design(key, t))))
}

fn menu_items(t: Theme) -> Stateful<Div> {
    Menu::design("menu", t)
        .children(STATES.into_iter().enumerate().map(|(i, state)| {
            MenuItem::new(("menu-item", i), format!("{state:?}"), t)
                .shortcut("⌘K")
                .state(state)
        }))
        .child(Submenu::new(
            MenuItem::new("recent", "Open recent", t),
            false,
            Menu::design("nested", t).child(MenuItem::new("project", "Workshop", t)),
        ))
        .child(Menu::destructive_group(
            [MenuItem::new("delete", "Delete project", t)],
            t,
        ))
}

pub fn feedback(t: Theme) -> Div {
    stack()
        .children(
            [
                FeedbackKind::Info,
                FeedbackKind::Success,
                FeedbackKind::Danger,
            ]
            .into_iter()
            .map(|kind| {
                stack()
                    .child(Toast::message(format!("{kind:?} notification"), kind, t))
                    .child(InlineError::message(
                        format!("{kind:?} inline feedback"),
                        kind,
                        t,
                    ))
            }),
        )
        .child(
            line()
                .child(Spinner::new("spinner", t))
                .child(Spinner::new("reduced-spinner", t).reduced_motion(true))
                .child(StatusBarItem::new("X", Some(-42), 5, t).build())
                .child(StatusBarItem::new("Count", None::<u32>, 6, t).build()),
        )
        .child(
            Shimmer::new("block-loading", t)
                .rows(3)
                .elapsed(Duration::from_millis(600)),
        )
        .child(caption("Swatches: unselected / selected / locked", t))
        .child(
            line()
                .children([false, true].map(|selected| {
                    Swatch::design(t, t.color(ColorToken::Accent))
                        .selected(selected)
                        .build()
                }))
                .child(
                    Swatch::design(t, t.color(ColorToken::Success))
                        .locked(true)
                        .build(),
                ),
        )
}

pub fn tokens(t: Theme) -> Div {
    stack()
        .child(line().children(ColorToken::ALL.into_iter().map(|token| {
            stack()
                .child(
                    div()
                        .size(px(Space::S7.px()))
                        .rounded(px(Radius::Xs.px()))
                        .bg(t.color(token)),
                )
                .child(TextSize::Caption.apply(div()).child(format!("{token:?}")))
        })))
        .children(
            [
                TextSize::Caption,
                TextSize::Small,
                TextSize::Body,
                TextSize::Title,
                TextSize::Heading,
                TextSize::Display,
                TextSize::Mono,
            ]
            .map(|s| s.apply(div()).child(format!("{s:?} · Quantum 0123456789"))),
        )
}
