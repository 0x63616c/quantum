//! Shared action registry for native menus, the Linux menubar and command search.
use crate::shortcuts::{ShortcutPlatform, format_shortcut};
use gpui::{Action, App, KeyBinding, Menu, MenuItem, OsAction};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Action)]
#[action(no_json)]
pub enum Op {
    About,
    Settings,
    CheckUpdates,
    Hide,
    HideOthers,
    ShowAll,
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    Minimize,
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    ZoomWindow,
    ShowPalette,
    New,
    Open,
    Save,
    SaveAs,
    #[cfg(target_family = "wasm")]
    ExportGlb,
    #[cfg(target_family = "wasm")]
    ExportPng,
    Comment,
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
    ManagePalettes,
    Commands,
    ToolPicker,
    Agent,
    Extensions,
    Help,
}
impl Op {
    /// Bundled Lucide asset shared by menus, search and toolbar controls.
    pub fn icon(self) -> &'static str {
        match self {
            Self::About => "cube",
            Self::Settings => "sliders-horizontal",
            Self::CheckUpdates => "download",
            Self::Hide | Self::HideOthers => "eye-off",
            Self::ShowAll => "eye",
            Self::Minimize | Self::ZoomWindow => "square",
            Self::ShowPalette => "palette",
            Self::New => "file-plus",
            Self::Open => "folder-open",
            Self::Save => "save",
            Self::SaveAs => "file-output",
            #[cfg(target_family = "wasm")]
            Self::ExportGlb | Self::ExportPng => "download",
            Self::Comment => "comment",
            Self::Quit => "log-out",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Frame => "cube",
            Self::Front => "panel-top",
            Self::Right => "panel-left",
            Self::Top => "square",
            Self::Grid => "grid",
            Self::Theme => "sun",
            Self::Place => "place",
            Self::Erase => "erase",
            Self::Paint => "paint",
            Self::Fill => "fill",
            Self::Box => "box",
            Self::Picker => "picker",
            Self::MirrorX => "flip-horizontal-2",
            Self::MirrorY => "flip-vertical-2",
            Self::MirrorZ => "flip-horizontal-2",
            Self::AddLayer => "layer",
            Self::Palette => "palette",
            Self::ManagePalettes => "sliders-horizontal",
            Self::Commands => "search",
            Self::ToolPicker => "wrench",
            Self::Agent => "spark",
            Self::Extensions => "puzzle",
            Self::Help => "keyboard",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::About => "About Voxelite",
            Self::Settings => "Settings…",
            Self::CheckUpdates => "Check for Updates…",
            Self::Hide => "Hide Voxelite",
            Self::HideOthers => "Hide Others",
            Self::ShowAll => "Show All",
            Self::Minimize => "Minimize",
            Self::ZoomWindow => "Zoom",
            Self::ShowPalette => "Show Palette",
            Self::New => "New project",
            Self::Open => "Open project…",
            Self::Save => "Save",
            Self::SaveAs => "Save as…",
            #[cfg(target_family = "wasm")]
            Self::ExportGlb => "Export GLB…",
            #[cfg(target_family = "wasm")]
            Self::ExportPng => "Export PNG…",
            Self::Comment => "Add comment pin",
            Self::Quit => "Quit Voxelite",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Frame => "Frame model",
            Self::Front => "Front view",
            Self::Right => "Right view",
            Self::Top => "Top view",
            Self::Grid => "Toggle grid",
            Self::Theme => "Appearance…",
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
            Self::ManagePalettes => "Manage palettes…",
            Self::Commands => "Search commands…",
            Self::ToolPicker => "Choose tool…",
            Self::Agent => "Agent connection",
            Self::Extensions => "Extensions",
            Self::Help => "Keyboard & mouse guide",
        }
    }
    pub fn key(self) -> Option<&'static str> {
        match self {
            Self::Settings => Some("secondary-,"),
            Self::Hide => Some("secondary-h"),
            Self::HideOthers => Some("secondary-alt-h"),
            Self::Minimize => Some("secondary-m"),
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
            Self::Comment => Some("c"),
            Self::Picker => Some("i"),
            Self::MirrorX => Some("shift-x"),
            Self::MirrorY => Some("shift-y"),
            Self::MirrorZ => Some("shift-z"),
            _ => None,
        }
    }
    pub fn disabled_reason(self) -> &'static str {
        match self {
            Self::Undo => "Nothing to undo",
            Self::Redo => "Nothing to redo",
            _ => "File operation in progress",
        }
    }
    pub fn shortcut(self) -> String {
        self.key()
            .map(|key| format_shortcut(key, ShortcutPlatform::current()))
            .unwrap_or_default()
    }

    fn binding(self, platform: ShortcutPlatform) -> Option<KeyBinding> {
        self.key()
            .map(|key| KeyBinding::new(&platform.binding_key(key), self, Some("Editor")))
    }
}
pub const MENUS: &[(&str, &[Op])] = &[
    (
        "File",
        &[
            Op::New,
            Op::Open,
            Op::Save,
            Op::SaveAs,
            #[cfg(target_family = "wasm")]
            Op::ExportGlb,
            #[cfg(target_family = "wasm")]
            Op::ExportPng,
            #[cfg(not(target_family = "wasm"))]
            Op::Quit,
        ],
    ),
    ("Edit", &[Op::Undo, Op::Redo, Op::Commands, Op::Settings]),
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
            Op::ToolPicker,
            Op::Comment,
        ],
    ),
    ("Layers", &[Op::AddLayer]),
    (
        "Palette",
        &[Op::ShowPalette, Op::Palette, Op::ManagePalettes],
    ),
    ("Agent", &[Op::Agent]),
    ("Extensions", &[Op::Extensions]),
    #[cfg(not(target_family = "wasm"))]
    ("Window", &[Op::Minimize, Op::ZoomWindow]),
    ("Help", &[Op::Help, Op::About]),
];
/// Clipboard commands are handled by the focused text editor, never by a hidden view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Action)]
#[action(no_json)]
pub enum ClipboardOp {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

/// Build both initial and live menus from the same action list.
pub fn menus(selected: impl Fn(Op) -> bool, enabled: impl Fn(Op) -> bool) -> Vec<Menu> {
    menus_for_platform(cfg!(target_os = "macos"), selected, enabled)
}

fn menus_for_platform(
    apple: bool,
    selected: impl Fn(Op) -> bool,
    enabled: impl Fn(Op) -> bool,
) -> Vec<Menu> {
    let item = |op: Op| {
        let item = match op {
            Op::Undo => MenuItem::os_action(op.label(), op, OsAction::Undo),
            Op::Redo => MenuItem::os_action(op.label(), op, OsAction::Redo),
            _ => MenuItem::action(op.label(), op),
        };
        item.checked(selected(op)).disabled(!enabled(op))
    };
    let mut menus = Vec::new();
    if apple {
        menus.push(Menu::new("Voxelite").items([
            item(Op::About),
            item(Op::CheckUpdates),
            MenuItem::separator(),
            item(Op::Settings),
            MenuItem::separator(),
            MenuItem::os_submenu("Services", gpui::SystemMenuType::Services),
            MenuItem::separator(),
            item(Op::Hide),
            item(Op::HideOthers),
            item(Op::ShowAll),
            MenuItem::separator(),
            item(Op::Quit),
        ]));
    }
    for (name, ops) in MENUS {
        let mut items: Vec<_> = ops
            .iter()
            .copied()
            .filter(|op| !apple || !matches!(op, Op::Quit | Op::Settings | Op::About))
            .map(item)
            .collect();
        if *name == "Edit" {
            items.splice(
                2..2,
                [
                    MenuItem::separator(),
                    MenuItem::os_action("Cut", ClipboardOp::Cut, OsAction::Cut),
                    MenuItem::os_action("Copy", ClipboardOp::Copy, OsAction::Copy),
                    MenuItem::os_action("Paste", ClipboardOp::Paste, OsAction::Paste),
                    MenuItem::os_action("Select All", ClipboardOp::SelectAll, OsAction::SelectAll),
                    MenuItem::separator(),
                ],
            );
        }
        menus.push(Menu::new(*name).items(items));
    }
    menus
}

pub fn install(cx: &mut App) {
    let platform = ShortcutPlatform::current();
    if cfg!(target_os = "macos") {
        cx.set_menus(menus(|_| false, |_| true));
    }
    cx.bind_keys(
        MENUS
            .iter()
            .flat_map(|(_, items)| items.iter())
            .filter(|op| !matches!(op, Op::Settings | Op::Quit))
            .filter_map(|op| op.binding(platform)),
    );
    #[cfg(not(target_family = "wasm"))]
    cx.bind_keys([KeyBinding::new(
        &platform.binding_key("secondary-q"),
        Op::Quit,
        None,
    )]);
    // Settings remains available while an input or modal has focus.
    cx.bind_keys([KeyBinding::new(
        &platform.binding_key("secondary-,"),
        Op::Settings,
        None,
    )]);
    #[cfg(target_os = "macos")]
    cx.bind_keys([Op::Hide, Op::HideOthers].into_iter().filter_map(|op| {
        op.key()
            .map(|key| KeyBinding::new(&platform.binding_key(key), op, None))
    }));
    cx.bind_keys(
        [
            ("secondary-x", ClipboardOp::Cut),
            ("secondary-c", ClipboardOp::Copy),
            ("secondary-v", ClipboardOp::Paste),
            ("secondary-a", ClipboardOp::SelectAll),
        ]
        .into_iter()
        .map(|(key, op)| KeyBinding::new(&platform.binding_key(key), op, Some("TextField"))),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{KeyContext, Keymap, Keystroke};

    #[test]
    fn mac_menus_preserve_editor_commands_and_system_entries() {
        let menus = menus_for_platform(true, |op| op == Op::Grid, |op| op != Op::Undo);
        assert_eq!(
            menus
                .iter()
                .map(|menu| menu.name.as_ref())
                .collect::<Vec<_>>(),
            [
                "Voxelite",
                "File",
                "Edit",
                "View",
                "Tools",
                "Layers",
                "Palette",
                "Agent",
                "Extensions",
                "Window",
                "Help"
            ]
        );
        assert!(menus[0].items.iter().any(|item| matches!(item, MenuItem::SystemMenu(menu) if menu.menu_type == gpui::SystemMenuType::Services)));
        let items: Vec<_> = menus.iter().flat_map(|menu| &menu.items).collect();
        for op in MENUS.iter().flat_map(|(_, ops)| *ops) {
            assert!(items.iter().any(|item| matches!(item, MenuItem::Action { action, .. } if action.as_any().downcast_ref::<Op>() == Some(op))), "missing {op:?}");
        }
        assert!(items.iter().any(|item| matches!(item, MenuItem::Action { action, disabled: true, os_action: Some(OsAction::Undo), .. } if action.as_any().downcast_ref::<Op>() == Some(&Op::Undo))));
        assert!(items.iter().any(|item| matches!(item, MenuItem::Action { action, checked: true, .. } if action.as_any().downcast_ref::<Op>() == Some(&Op::Grid))));
        for op in [
            ClipboardOp::Cut,
            ClipboardOp::Copy,
            ClipboardOp::Paste,
            ClipboardOp::SelectAll,
        ] {
            assert!(items.iter().any(|item| matches!(item, MenuItem::Action { action, os_action: Some(_), .. } if action.as_any().downcast_ref::<ClipboardOp>() == Some(&op))));
        }
    }

    #[test]
    fn primary_actions_bind_command_on_apple_and_control_elsewhere() {
        for (op, apple, other) in [
            (Op::New, "cmd-n", "ctrl-n"),
            (Op::Open, "cmd-o", "ctrl-o"),
            (Op::Save, "cmd-s", "ctrl-s"),
            (Op::SaveAs, "cmd-shift-s", "ctrl-shift-s"),
            (Op::Quit, "cmd-q", "ctrl-q"),
            (Op::Undo, "cmd-z", "ctrl-z"),
            (Op::Redo, "cmd-shift-z", "ctrl-shift-z"),
            (Op::Commands, "cmd-k", "ctrl-k"),
            (Op::Settings, "cmd-,", "ctrl-,"),
        ] {
            for (platform, accepted, rejected) in [
                (ShortcutPlatform::Apple, apple, other),
                (ShortcutPlatform::WindowsLinux, other, apple),
            ] {
                let binding = op.binding(platform).unwrap();
                let accepted = Keystroke::parse(accepted).unwrap();
                let rejected = Keystroke::parse(rejected).unwrap();
                assert_eq!(binding.match_keystrokes(&[accepted]), Some(false), "{op:?}");
                assert_eq!(binding.match_keystrokes(&[rejected]), None, "{op:?}");
            }
        }
    }

    #[test]
    fn tool_and_mirror_bindings_are_platform_independent() {
        for op in MENUS.iter().flat_map(|(_, items)| *items) {
            let Some(key) = op.key().filter(|key| !key.contains("secondary")) else {
                continue;
            };
            let stroke = Keystroke::parse(key).unwrap();
            for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
                assert_eq!(
                    op.binding(platform)
                        .unwrap()
                        .match_keystrokes(std::slice::from_ref(&stroke)),
                    Some(false),
                    "{op:?}"
                );
            }
        }
    }

    #[test]
    fn editor_shortcuts_are_inactive_while_typing_or_in_modals() {
        for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
            let bindings: Vec<_> = MENUS
                .iter()
                .flat_map(|(_, items)| *items)
                .filter_map(|op| op.binding(platform))
                .collect();
            let keymap = Keymap::new(bindings.clone());
            for binding in bindings {
                let (matches, _) = keymap.bindings_for_input(
                    binding.keystrokes(),
                    &[KeyContext::parse("Editor").unwrap()],
                );
                assert_eq!(matches.len(), 1);
                for context in ["TextField", "CommandPalette", "Modal"] {
                    let (matches, pending) = keymap.bindings_for_input(
                        binding.keystrokes(),
                        &[KeyContext::parse(context).unwrap()],
                    );
                    assert!(matches.is_empty(), "{context}: {binding:?}");
                    assert!(!pending);
                }
            }
        }
    }
}
