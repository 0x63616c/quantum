//! Shared action registry for native menus, the Linux menubar and command search.
use gpui::{Action, App, KeyBinding, Menu, MenuItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Action)]
#[action(no_json)]
pub enum Op {
    New,
    Open,
    Save,
    SaveAs,
    Quit,
    Undo,
    Redo,
    Frame,
    Front,
    Right,
    Top,
    Grid,
    Theme,
    Place,
    Erase,
    Paint,
    Fill,
    Box,
    Picker,
    MirrorX,
    MirrorY,
    MirrorZ,
    AddLayer,
    Palette,
    Commands,
    Agent,
    Extensions,
    Help,
}
impl Op {
    pub fn label(self) -> &'static str {
        match self {
            Self::New => "New project",
            Self::Open => "Open project…",
            Self::Save => "Save",
            Self::SaveAs => "Save as…",
            Self::Quit => "Quit Voxelite",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Frame => "Frame model",
            Self::Front => "Front view",
            Self::Right => "Right view",
            Self::Top => "Top view",
            Self::Grid => "Toggle grid",
            Self::Theme => "Switch appearance",
            Self::Place => "Place",
            Self::Erase => "Erase",
            Self::Paint => "Paint",
            Self::Fill => "Fill",
            Self::Box => "Box",
            Self::Picker => "Color picker",
            Self::MirrorX => "Mirror X",
            Self::MirrorY => "Mirror Y",
            Self::MirrorZ => "Mirror Z",
            Self::AddLayer => "Add layer",
            Self::Palette => "Choose palette…",
            Self::Commands => "Search commands…",
            Self::Agent => "Agent connection",
            Self::Extensions => "Extensions",
            Self::Help => "Keyboard & mouse guide",
        }
    }
    pub fn key(self) -> Option<&'static str> {
        match self {
            Self::New => Some("secondary-n"),
            Self::Open => Some("secondary-o"),
            Self::Save => Some("secondary-s"),
            Self::SaveAs => Some("secondary-shift-s"),
            Self::Quit => Some("secondary-q"),
            Self::Undo => Some("secondary-z"),
            Self::Redo => Some("secondary-shift-z"),
            Self::Commands => Some("secondary-k"),
            Self::Frame => Some("f"),
            Self::Place => Some("b"),
            Self::Erase => Some("e"),
            Self::Paint => Some("p"),
            Self::Fill => Some("g"),
            Self::Box => Some("u"),
            Self::Picker => Some("i"),
            _ => None,
        }
    }
    pub fn shortcut(self) -> &'static str {
        match self {
            Self::New => "⌘/Ctrl N",
            Self::Open => "⌘/Ctrl O",
            Self::Save => "⌘/Ctrl S",
            Self::SaveAs => "⇧ ⌘/Ctrl S",
            Self::Quit => "⌘/Ctrl Q",
            Self::Undo => "⌘/Ctrl Z",
            Self::Redo => "⇧ ⌘/Ctrl Z",
            Self::Commands => "⌘/Ctrl K",
            Self::Frame => "F",
            Self::Place => "B",
            Self::Erase => "E",
            Self::Paint => "P",
            Self::Fill => "G",
            Self::Box => "U",
            Self::Picker => "I",
            _ => "",
        }
    }
}
pub const MENUS: &[(&str, &[Op])] = &[
    ("File", &[Op::New, Op::Open, Op::Save, Op::SaveAs, Op::Quit]),
    ("Edit", &[Op::Undo, Op::Redo, Op::Commands]),
    (
        "View",
        &[
            Op::Frame,
            Op::Front,
            Op::Right,
            Op::Top,
            Op::Grid,
            Op::Theme,
        ],
    ),
    (
        "Tools",
        &[
            Op::Place,
            Op::Erase,
            Op::Paint,
            Op::Fill,
            Op::Box,
            Op::Picker,
            Op::MirrorX,
            Op::MirrorY,
            Op::MirrorZ,
        ],
    ),
    ("Layers", &[Op::AddLayer]),
    ("Palette", &[Op::Palette]),
    ("Agent", &[Op::Agent]),
    ("Extensions", &[Op::Extensions]),
    ("Help", &[Op::Help]),
];
pub fn install(cx: &mut App) {
    cx.set_menus(
        MENUS
            .iter()
            .map(|(name, items)| {
                Menu::new(*name).items(items.iter().map(|op| MenuItem::action(op.label(), *op)))
            })
            .collect::<Vec<_>>(),
    );
    cx.bind_keys(
        MENUS
            .iter()
            .flat_map(|(_, items)| items.iter())
            .filter_map(|op| {
                op.key()
                    .map(|key| KeyBinding::new(key, *op, Some("Editor")))
            }),
    );
}
