//! Command metadata and availability shared by menus, search, help and documentation.
use crate::shortcuts::{ShortcutPlatform, format_shortcut};
use gpui::{Action, App, KeyBinding, Menu, MenuItem, OsAction};
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ApplicationSection {
    About,
    Settings,
    Visibility,
    Quit,
}

struct Command {
    label: &'static str,
    shortcut: Option<Shortcut>,
    icon: &'static str,
    menu: &'static str,
    application_section: Option<ApplicationSection>,
}

enum Shortcut {
    Editor(&'static str),
    Global(&'static str),
}

impl Shortcut {
    fn key(&self) -> &'static str {
        match self {
            Self::Editor(key) | Self::Global(key) => key,
        }
    }

    fn context(&self) -> Option<&'static str> {
        match self {
            Self::Editor(_) => Some("Editor"),
            Self::Global(_) => None,
        }
    }
}

// Declaring an operation also registers it everywhere. Group and declaration
// order are the menu/search order; macOS relocates application commands below.
macro_rules! commands {
    ($($menu:literal { $($(#[$attr:meta])* $op:ident =>
        ($label:literal, $key:expr, $icon:literal, $application:expr)),+ $(,)? }),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Action)]
        #[action(no_json)]
        pub enum Op { $($($(#[$attr])* $op),+),+ }

        impl Op {
            pub const ALL: &[Self] = &[$($($(#[$attr])* Self::$op),+),+];

            fn command(self) -> &'static Command {
                match self {
                    $($($(#[$attr])* Self::$op => &Command {
                        label: $label, shortcut: $key, icon: $icon, menu: $menu,
                        application_section: $application,
                    }),+),+
                }
            }
        }

        const MENU_NAMES: &[&str] = &[$($menu),+];
    };
}

commands! {
    "File" {
        New => ("New project", Some(Shortcut::Editor("secondary-n")), "file-plus", None),
        Open => ("Open project…", Some(Shortcut::Editor("secondary-o")), "folder-open", None),
        Save => ("Save", Some(Shortcut::Editor("secondary-s")), "save", None),
        SaveAs => ("Save as…", Some(Shortcut::Editor("secondary-shift-s")), "file-output", None),
        #[cfg(target_family = "wasm")]
        ExportGlb => ("Export GLB…", None, "download", None),
        #[cfg(target_family = "wasm")]
        ExportPng => ("Export PNG…", None, "download", None),
        Quit => ("Quit Voxelite", Some(Shortcut::Global("secondary-q")), "log-out", Some(ApplicationSection::Quit)),
    },
    "Edit" {
        Undo => ("Undo", Some(Shortcut::Editor("secondary-z")), "undo", None),
        Redo => ("Redo", Some(Shortcut::Editor("secondary-shift-z")), "redo", None),
        Commands => ("Search commands…", Some(Shortcut::Editor("secondary-k")), "search", None),
        Settings => ("Settings…", Some(Shortcut::Global("secondary-,")), "settings", Some(ApplicationSection::Settings)),
    },
    "View" {
        Frame => ("Frame model", Some(Shortcut::Editor("f")), "cube", None),
        Front => ("Front view", None, "panel-top", None),
        Right => ("Right view", None, "panel-left", None),
        Top => ("Top view", None, "square", None),
        Grid => ("Toggle grid", None, "grid", None),
        Theme => ("Appearance…", None, "sun", None),
    },
    "Tools" {
        Place => ("Place", Some(Shortcut::Editor("b")), "place", None),
        Erase => ("Erase", Some(Shortcut::Editor("e")), "erase", None),
        Paint => ("Paint", Some(Shortcut::Editor("p")), "paint", None),
        Fill => ("Fill", Some(Shortcut::Editor("g")), "fill", None),
        Box => ("Box", Some(Shortcut::Editor("u")), "box", None),
        Picker => ("Color picker", Some(Shortcut::Editor("i")), "picker", None),
        MirrorX => ("Mirror X", Some(Shortcut::Editor("shift-x")), "flip-horizontal-2", None),
        MirrorY => ("Mirror Y", Some(Shortcut::Editor("shift-y")), "flip-vertical-2", None),
        MirrorZ => ("Mirror Z", Some(Shortcut::Editor("shift-z")), "flip-horizontal-2", None),
        ToolPicker => ("Choose tool…", None, "wrench", None),
        Comment => ("Add comment pin", Some(Shortcut::Editor("c")), "comment", None),
    },
    "Layers" {
        AddLayer => ("Add layer", None, "layer", None),
    },
    "Palette" {
        ShowPalette => ("Show Palette", None, "palette", None),
        Palette => ("Choose palette…", None, "palette", None),
        ManagePalettes => ("Manage palettes…", None, "sliders-horizontal", None),
    },
    "Agent" {
        Agent => ("Agent connection", None, "spark", None),
    },
    "Extensions" {
        Extensions => ("Extensions", None, "puzzle", None),
    },
    "Window" {
        Minimize => ("Minimize", Some(Shortcut::Editor("secondary-m")), "square", None),
        ZoomWindow => ("Zoom", None, "square", None),
    },
    "Help" {
        Help => ("Keyboard & mouse guide", None, "keyboard", None),
        About => ("About Voxelite", None, "cube", Some(ApplicationSection::About)),
    },
    "Voxelite" {
        CheckUpdates => ("Check for Updates…", None, "download", Some(ApplicationSection::About)),
        Hide => ("Hide Voxelite", Some(Shortcut::Global("secondary-h")), "eye-off", Some(ApplicationSection::Visibility)),
        HideOthers => ("Hide Others", Some(Shortcut::Global("secondary-alt-h")), "eye-off", Some(ApplicationSection::Visibility)),
        ShowAll => ("Show All", None, "eye", Some(ApplicationSection::Visibility)),
    },
}

/// Only the state needed to decide command availability; no view or GPUI context.
#[derive(Clone, Copy, Debug, Default)]
pub struct EditorState {
    pub dialog_open: bool,
    pub pending_confirmation: bool,
    pub modal_open: bool,
    pub preview: bool,
    pub history_playing: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub updates_available: bool,
}

#[derive(Clone, Copy)]
pub enum CommandContext {
    Editor(EditorState),
    Settings { updates_available: bool },
}

impl Op {
    pub fn label(self) -> &'static str {
        self.command().label
    }

    /// Bundled Lucide asset shared by menus, search and toolbar controls.
    pub fn icon(self) -> &'static str {
        self.command().icon
    }

    pub fn key(self) -> Option<&'static str> {
        self.command().shortcut.as_ref().map(Shortcut::key)
    }

    pub fn menu(self, apple: bool) -> &'static str {
        if apple && self.command().application_section.is_some() {
            "Voxelite"
        } else {
            self.command().menu
        }
    }

    fn available_on(self, platform: ShortcutPlatform) -> bool {
        match self {
            Self::Quit | Self::Minimize | Self::ZoomWindow => !cfg!(target_family = "wasm"),
            _ if self.command().menu == "Voxelite" => {
                !cfg!(target_family = "wasm") && platform == ShortcutPlatform::Apple
            }
            _ => true,
        }
    }

    pub fn shortcut(self) -> String {
        self.key()
            .map(|key| format_shortcut(key, ShortcutPlatform::current()))
            .unwrap_or_default()
    }

    fn binding(self, platform: ShortcutPlatform) -> Option<KeyBinding> {
        self.command().shortcut.as_ref().map(|shortcut| {
            KeyBinding::new(
                &platform.binding_key(shortcut.key()),
                self,
                shortcut.context(),
            )
        })
    }

    fn is_tool(self) -> bool {
        matches!(
            self,
            Self::Place | Self::Erase | Self::Paint | Self::Fill | Self::Box | Self::Picker
        )
    }

    /// Enabled commands return Ok; disabled commands carry their presentation reason.
    pub fn availability(self, context: CommandContext) -> Result<(), &'static str> {
        let state = match context {
            CommandContext::Editor(state) => state,
            CommandContext::Settings { updates_available } => {
                return match self {
                    Self::CheckUpdates if !updates_available => {
                        Err("Automatic updates are not available in this build.")
                    }
                    Self::About
                    | Self::Settings
                    | Self::CheckUpdates
                    | Self::Quit
                    | Self::Hide
                    | Self::HideOthers
                    | Self::ShowAll
                    | Self::Minimize
                    | Self::ZoomWindow
                    | Self::Help => Ok(()),
                    _ => Err("Return to the editor to use this command"),
                };
            }
        };
        let edits_preview = self.is_tool()
            || matches!(
                self,
                Self::MirrorX
                    | Self::MirrorY
                    | Self::MirrorZ
                    | Self::AddLayer
                    | Self::Comment
                    | Self::ToolPicker
            );
        if state.dialog_open {
            return Err("File operation in progress");
        }
        if state.pending_confirmation {
            return Err("Finish saving or discarding changes first");
        }
        if state.modal_open
            && !matches!(
                self,
                Self::Settings | Self::Quit | Self::Hide | Self::HideOthers | Self::ShowAll
            )
        {
            return Err("Close the dialog to use this command");
        }
        if state.preview
            && (edits_preview
                || (!state.history_playing && matches!(self, Self::Undo | Self::Redo)))
        {
            return Err("Return to live to edit");
        }
        match self {
            Self::Undo if !state.history_playing && !state.can_undo => Err("Nothing to undo"),
            Self::Redo if !state.history_playing && !state.can_redo => Err("Nothing to redo"),
            Self::CheckUpdates if !state.updates_available => {
                Err("Automatic updates are not available in this build.")
            }
            _ => Ok(()),
        }
    }
}

/// In-window menus use the same placement on every host, including Apple browsers.
pub static MENUS: LazyLock<Vec<(&str, Vec<Op>)>> = LazyLock::new(|| {
    MENU_NAMES
        .iter()
        .filter_map(|name| {
            let ops: Vec<_> = Op::ALL
                .iter()
                .copied()
                .filter(|op| {
                    op.available_on(ShortcutPlatform::WindowsLinux) && op.menu(false) == *name
                })
                .collect();
            (!ops.is_empty()).then_some((*name, ops))
        })
        .collect()
});

pub fn search(query: &str, tools_only: bool) -> Vec<Op> {
    search_on(query, tools_only, ShortcutPlatform::current())
}

fn search_on(query: &str, tools_only: bool, platform: ShortcutPlatform) -> Vec<Op> {
    let query = query.to_lowercase();
    shortcut_sheet(platform)
        .filter(|op| {
            *op != Op::Commands
                && (!tools_only || op.is_tool())
                && op.label().to_lowercase().contains(&query)
        })
        .collect()
}

/// Every available command appears, including commands without a keyboard binding.
pub fn shortcut_sheet(platform: ShortcutPlatform) -> impl Iterator<Item = Op> {
    Op::ALL
        .iter()
        .copied()
        .filter(move |op| op.available_on(platform))
}

/// Guide content is independent of the UI's two-column layout and keycap styling.
pub enum GuideShortcut {
    Action(Op),
    Keys(&'static [&'static str]),
    Gesture(&'static str),
}

pub struct GuideSection {
    pub title: &'static str,
    pub rows: Vec<(&'static str, GuideShortcut)>,
}

/// Include every available bound command, grouped by its registry menu.
/// Pointer gestures and cursor keys are handled directly by the viewport.
pub fn keyboard_guide_sections(platform: ShortcutPlatform) -> [Vec<GuideSection>; 2] {
    let mut tools = Vec::new();
    let mut navigate = vec![
        ("Orbit", GuideShortcut::Gesture("Right-drag")),
        ("Pan", GuideShortcut::Gesture("Middle-drag")),
        ("Pan (alternate)", GuideShortcut::Keys(&["shift-rightdrag"])),
        ("Zoom", GuideShortcut::Gesture("Scroll")),
    ];
    let mut file_history = Vec::new();
    for op in shortcut_sheet(platform).filter(|op| op.key().is_some()) {
        // Keep the guide's existing prose where it differs from a menu label.
        let label = match op {
            Op::Frame => "Frame model · isometric",
            Op::Commands => "Search commands",
            Op::Open => "Open project",
            _ => op.label(),
        };
        // Use the declared group even when macOS relocates an application item.
        let rows = match op.command().menu {
            "Tools" => &mut tools,
            "View" => &mut navigate,
            _ => &mut file_history,
        };
        rows.push((label, GuideShortcut::Action(op)));
    }
    tools.push(("Apply at pointer", GuideShortcut::Gesture("Click")));
    [
        vec![
            GuideSection {
                title: "Tools",
                rows: tools,
            },
            GuideSection {
                title: "Navigate",
                rows: navigate,
            },
        ],
        vec![
            GuideSection {
                title: "Box & cursor",
                rows: vec![
                    (
                        "Set opposite corners",
                        GuideShortcut::Gesture("Click, then click"),
                    ),
                    ("Cancel box", GuideShortcut::Keys(&["escape"])),
                    (
                        "Move cursor · X/Z",
                        GuideShortcut::Keys(&["left", "right", "up", "down"]),
                    ),
                    (
                        "Move cursor · Y",
                        GuideShortcut::Keys(&["pageup", "pagedown"]),
                    ),
                    ("Apply at cursor", GuideShortcut::Keys(&["enter"])),
                ],
            },
            GuideSection {
                title: "Colors",
                rows: vec![
                    ("Sample at pointer", GuideShortcut::Keys(&["alt-click"])),
                    ("Previous / next color", GuideShortcut::Keys(&["[", "]"])),
                ],
            },
            GuideSection {
                title: "File & history",
                rows: file_history,
            },
        ],
    ]
}

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
        let mut items = Vec::new();
        for section in [
            ApplicationSection::About,
            ApplicationSection::Settings,
            ApplicationSection::Visibility,
            ApplicationSection::Quit,
        ] {
            if !items.is_empty() {
                items.push(MenuItem::separator());
            }
            items.extend(
                Op::ALL
                    .iter()
                    .copied()
                    .filter(|op| {
                        op.available_on(ShortcutPlatform::Apple)
                            && op.command().application_section == Some(section)
                    })
                    .map(item),
            );
            if section == ApplicationSection::Settings {
                items.push(MenuItem::separator());
                items.push(MenuItem::os_submenu(
                    "Services",
                    gpui::SystemMenuType::Services,
                ));
            }
        }
        menus.push(Menu::new("Voxelite").items(items));
    }
    for (name, ops) in MENUS.iter() {
        let mut items: Vec<_> = ops
            .iter()
            .copied()
            .filter(|op| op.menu(apple) == *name)
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
    cx.bind_keys(key_bindings(platform));
}

fn key_bindings(platform: ShortcutPlatform) -> Vec<KeyBinding> {
    shortcut_sheet(platform)
        .filter_map(|op| op.binding(platform))
        .chain(
            [
                ("secondary-x", ClipboardOp::Cut),
                ("secondary-c", ClipboardOp::Copy),
                ("secondary-v", ClipboardOp::Paste),
                ("secondary-a", ClipboardOp::SelectAll),
            ]
            .into_iter()
            .map(|(key, op)| KeyBinding::new(&platform.binding_key(key), op, Some("TextField"))),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AssetSource, KeyContext, Keymap, Keystroke};
    use std::collections::HashSet;

    #[test]
    fn every_command_has_unique_metadata_and_appears_in_the_sheet() {
        let mut labels = HashSet::new();
        for op in Op::ALL {
            assert!(!op.label().is_empty(), "{op:?}");
            assert!(
                labels.insert(op.label().to_lowercase()),
                "duplicate label: {op:?}"
            );
            assert!(!op.icon().is_empty(), "{op:?}");
            assert!(
                crate::assets::Assets
                    .load(&format!("icons/{}.svg", op.icon()))
                    .unwrap()
                    .is_some(),
                "icon is not bundled: {op:?}"
            );
        }
        for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
            let mut keys = HashSet::new();
            let sheet: Vec<_> = shortcut_sheet(platform).collect();
            for op in Op::ALL {
                assert_eq!(
                    sheet.iter().filter(|entry| *entry == op).count(),
                    usize::from(op.available_on(platform)),
                    "{op:?}"
                );
                if let Some(binding) = op.binding(platform) {
                    assert!(
                        keys.insert(binding.keystrokes().to_vec()),
                        "duplicate shortcut: {op:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn sheet_includes_comment_and_all_mirror_shortcuts() {
        for (op, apple, other) in [
            (Op::Comment, "C", "C"),
            (Op::MirrorX, "⇧X", "Shift+X"),
            (Op::MirrorY, "⇧Y", "Shift+Y"),
            (Op::MirrorZ, "⇧Z", "Shift+Z"),
        ] {
            for (platform, expected) in [
                (ShortcutPlatform::Apple, apple),
                (ShortcutPlatform::WindowsLinux, other),
            ] {
                assert!(shortcut_sheet(platform).any(|entry| entry == op));
                assert_eq!(format_shortcut(op.key().unwrap(), platform), expected);
            }
        }
    }

    #[test]
    fn guide_includes_every_available_bound_command_once() {
        for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
            let columns = keyboard_guide_sections(platform);
            let commands: Vec<_> = columns
                .iter()
                .flatten()
                .flat_map(|section| &section.rows)
                .filter_map(|(_, binding)| match binding {
                    GuideShortcut::Action(op) => Some(*op),
                    _ => None,
                })
                .collect();
            let expected: Vec<_> = shortcut_sheet(platform)
                .filter(|op| op.key().is_some())
                .collect();
            assert_eq!(commands.len(), expected.len(), "duplicate or missing rows");
            for op in expected {
                assert_eq!(
                    commands.iter().filter(|entry| **entry == op).count(),
                    1,
                    "{op:?}"
                );
            }

            let tools = &columns[0][0];
            assert_eq!(tools.title, "Tools");
            for op in [Op::Comment, Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
                assert!(tools.rows.iter().any(|(_, binding)| {
                    matches!(binding, GuideShortcut::Action(entry) if *entry == op)
                }));
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn readme_shortcuts() -> String {
        use std::fmt::Write;
        let mut table = String::from("| Action | Apple | Windows / Linux |\n| --- | --- | --- |\n");
        for op in shortcut_sheet(ShortcutPlatform::Apple).filter(|op| op.key().is_some()) {
            let key = op.key().unwrap();
            let other = if op.available_on(ShortcutPlatform::WindowsLinux) {
                format_shortcut(key, ShortcutPlatform::WindowsLinux)
            } else {
                "—".into()
            };
            writeln!(
                table,
                "| {} | {} | {} |",
                op.label(),
                format_shortcut(key, ShortcutPlatform::Apple),
                other
            )
            .unwrap();
        }
        table
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn readme_shortcuts_match_registry() {
        const START: &str = "<!-- BEGIN GENERATED COMMAND SHORTCUTS -->\n";
        const END: &str = "<!-- END GENERATED COMMAND SHORTCUTS -->";
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
        let readme = std::fs::read_to_string(&path)
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(
            readme.matches(START).count(),
            1,
            "missing or duplicate start marker"
        );
        assert_eq!(
            readme.matches(END).count(),
            1,
            "missing or duplicate end marker"
        );
        let (prefix, rest) = readme.split_once(START).unwrap();
        let (table, suffix) = rest.split_once(END).unwrap();
        let expected = readme_shortcuts();
        if std::env::var("UPDATE_README").as_deref() == Ok("1") {
            std::fs::write(path, format!("{prefix}{START}{expected}{END}{suffix}")).unwrap();
        } else {
            assert_eq!(
                table, expected,
                "README shortcuts drifted. Regenerate with: UPDATE_README=1 cargo test -p voxelite-app --bin voxelite readme_shortcuts_match_registry"
            );
        }
    }

    #[test]
    fn editor_availability_preserves_live_preview_and_playback_rules() {
        let ready = EditorState {
            can_undo: true,
            can_redo: true,
            updates_available: true,
            ..EditorState::default()
        };
        let preview_blocked = [
            Op::Undo,
            Op::Redo,
            Op::Place,
            Op::Erase,
            Op::Paint,
            Op::Fill,
            Op::Box,
            Op::Picker,
            Op::MirrorX,
            Op::MirrorY,
            Op::MirrorZ,
            Op::AddLayer,
            Op::Comment,
            Op::ToolPicker,
        ];
        for op in Op::ALL {
            assert_eq!(
                op.availability(CommandContext::Editor(ready)),
                Ok(()),
                "{op:?}"
            );
            let preview = EditorState {
                preview: true,
                ..ready
            };
            assert_eq!(
                op.availability(CommandContext::Editor(preview)),
                if preview_blocked.contains(op) {
                    Err("Return to live to edit")
                } else {
                    Ok(())
                },
                "{op:?}"
            );
            let playing = EditorState {
                history_playing: true,
                can_undo: false,
                can_redo: false,
                ..preview
            };
            assert_eq!(
                op.availability(CommandContext::Editor(playing)).is_ok(),
                !preview_blocked.contains(op) || [Op::Undo, Op::Redo].contains(op),
                "{op:?}"
            );
            for state in [ready, preview, playing] {
                assert!(
                    op.availability(CommandContext::Editor(EditorState {
                        dialog_open: true,
                        ..state
                    }))
                    .is_err(),
                    "{op:?}"
                );
            }
        }
        let empty = CommandContext::Editor(EditorState::default());
        assert_eq!(Op::Undo.availability(empty), Err("Nothing to undo"));
        assert_eq!(Op::Redo.availability(empty), Err("Nothing to redo"));
        assert!(Op::CheckUpdates.availability(empty).is_err());
        let undo_only = CommandContext::Editor(EditorState {
            can_redo: false,
            ..ready
        });
        assert_eq!(Op::Undo.availability(undo_only), Ok(()));
        assert_eq!(Op::Redo.availability(undo_only), Err("Nothing to redo"));
        let redo_only = CommandContext::Editor(EditorState {
            can_undo: false,
            ..ready
        });
        assert_eq!(Op::Undo.availability(redo_only), Err("Nothing to undo"));
        assert_eq!(Op::Redo.availability(redo_only), Ok(()));
    }

    #[test]
    fn settings_availability_keeps_editor_commands_disabled() {
        let available = [
            Op::About,
            Op::Settings,
            Op::CheckUpdates,
            Op::Quit,
            Op::Hide,
            Op::HideOthers,
            Op::ShowAll,
            Op::Minimize,
            Op::ZoomWindow,
            Op::Help,
        ];
        for op in Op::ALL {
            for updates_available in [false, true] {
                assert_eq!(
                    op.availability(CommandContext::Settings { updates_available })
                        .is_ok(),
                    available.contains(op) && (*op != Op::CheckUpdates || updates_available),
                    "{op:?}"
                );
            }
        }
    }

    #[test]
    fn disabled_reasons_identify_the_actual_blocker() {
        for preview in [false, true] {
            let state = EditorState {
                preview,
                ..EditorState::default()
            };
            assert_eq!(
                Op::CheckUpdates.availability(CommandContext::Editor(state)),
                Err("Automatic updates are not available in this build.")
            );
            for op in Op::ALL {
                assert_eq!(
                    op.availability(CommandContext::Editor(EditorState {
                        dialog_open: true,
                        ..state
                    })),
                    Err("File operation in progress"),
                    "{op:?}"
                );
                assert_eq!(
                    op.availability(CommandContext::Editor(EditorState {
                        pending_confirmation: true,
                        ..state
                    })),
                    Err("Finish saving or discarding changes first"),
                    "{op:?}"
                );
            }
        }
    }

    #[test]
    fn modal_availability_matches_dispatch_restrictions() {
        let state = CommandContext::Editor(EditorState {
            modal_open: true,
            can_undo: true,
            can_redo: true,
            updates_available: true,
            ..EditorState::default()
        });
        for op in Op::ALL {
            let expected = if [
                Op::Settings,
                Op::Quit,
                Op::Hide,
                Op::HideOthers,
                Op::ShowAll,
            ]
            .contains(op)
            {
                Ok(())
            } else {
                Err("Close the dialog to use this command")
            };
            assert_eq!(op.availability(state), expected, "{op:?}");
        }
    }

    #[test]
    fn search_includes_every_supported_command_except_itself() {
        for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
            let search = search_on("", false, platform);
            for op in Op::ALL {
                assert_eq!(
                    search.iter().filter(|entry| *entry == op).count(),
                    usize::from(op.available_on(platform) && *op != Op::Commands),
                    "{platform:?}: {op:?}"
                );
            }
        }
        assert_eq!(
            search_on("updates", false, ShortcutPlatform::Apple),
            if cfg!(target_family = "wasm") {
                vec![]
            } else {
                vec![Op::CheckUpdates]
            }
        );
        assert!(search_on("updates", false, ShortcutPlatform::WindowsLinux).is_empty());
    }

    #[test]
    fn search_preserves_tool_order_and_matches_labels_case_insensitively() {
        assert_eq!(
            search("", true),
            [
                Op::Place,
                Op::Erase,
                Op::Paint,
                Op::Fill,
                Op::Box,
                Op::Picker
            ]
        );
        assert_eq!(
            search("mIRroR", false),
            [Op::MirrorX, Op::MirrorY, Op::MirrorZ]
        );
        assert_eq!(search("comment", false), [Op::Comment]);
        assert!(search("comment", true).is_empty());
        assert!(!search("", false).contains(&Op::Commands));
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
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
        for op in shortcut_sheet(ShortcutPlatform::Apple) {
            assert_eq!(items.iter().filter(|item| matches!(item, MenuItem::Action { action, .. } if action.as_any().downcast_ref::<Op>() == Some(&op))).count(), 1, "missing or duplicated {op:?}");
            let menu = menus
                .iter()
                .find(|menu| menu.name.as_ref() == op.menu(true))
                .unwrap();
            assert!(menu.items.iter().any(|item| matches!(item, MenuItem::Action { action, .. } if action.as_any().downcast_ref::<Op>() == Some(&op))), "wrong menu: {op:?}");
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
        for op in MENUS.iter().flat_map(|(_, items)| items) {
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
    fn installed_shortcuts_respect_editor_global_and_text_field_contexts() {
        for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
            let bindings = key_bindings(platform);
            let keymap = Keymap::new(bindings.clone());
            for binding in &bindings {
                for context in [
                    "Editor",
                    "TextField",
                    "CommandPalette",
                    "Modal",
                    "Settings",
                    "Palette",
                ] {
                    let expected = match binding.action().as_any().downcast_ref::<Op>() {
                        Some(Op::Settings | Op::Quit | Op::Hide | Op::HideOthers) => true,
                        Some(_) => context == "Editor",
                        None => context == "TextField",
                    };
                    let (matches, pending) = keymap.bindings_for_input(
                        binding.keystrokes(),
                        &[KeyContext::parse(context).unwrap()],
                    );
                    assert_eq!(
                        matches.len(),
                        usize::from(expected),
                        "{context}: {binding:?}"
                    );
                    if expected {
                        assert!(
                            matches[0].action().partial_eq(binding.action()),
                            "{context}: {binding:?}"
                        );
                    }
                    assert!(!pending);
                }
            }
            for op in Op::ALL {
                assert_eq!(
                    bindings.iter().filter(|binding| binding.action().as_any().downcast_ref::<Op>() == Some(op)).count(),
                    usize::from(op.available_on(platform) && op.key().is_some()),
                    "{platform:?}: {op:?}"
                );
            }
        }
    }
}
