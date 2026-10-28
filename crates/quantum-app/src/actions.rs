//! Command metadata and availability shared by menus, search, help and documentation.
use crate::shortcuts::{ShortcutPlatform, format_shortcut};
use gpui::{Action, KeyBinding, Menu, MenuItem, OsAction};

use quantum_plugin_api::{ApplicationMenuSection, CommandContribution};
use quantum_plugin_host::Host;
use std::sync::LazyLock;

mod bindings;
pub use bindings::install;
#[cfg(test)]
use bindings::key_bindings;

// Dispatch identities only. All presentation metadata belongs to contributions.
macro_rules! operations {
    ($($(#[$attr:meta])* $op:ident => $id:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Action)]
        #[action(no_json)]
        pub enum Op { $($(#[$attr])* $op),+ }

        impl Op {
            #[cfg(test)]
            pub const ALL: &[Self] = &[$($(#[$attr])* Self::$op),+];

            pub(crate) fn command_id(self) -> &'static str {
                match self { $($(#[$attr])* Self::$op => $id),+ }
            }

            pub(crate) fn from_command_id(id: &str) -> Option<Self> {
                match id {
                    $($(#[$attr])* $id => Some(Self::$op),)+
                    _ => None,
                }
            }
        }
    };
}

operations! {
    New => "dev.quantum.core.new",
    Open => "dev.quantum.core.open",
    OpenSample => "dev.quantum.core.open_sample",
    OpenRecent => "dev.quantum.core.open_recent",
    ClearRecents => "dev.quantum.core.clear_recents",
    Welcome => "dev.quantum.core.welcome",
    Save => "dev.quantum.core.save",
    SaveAs => "dev.quantum.core.save_as",
    Quit => "dev.quantum.core.quit",
    Undo => "dev.quantum.core.undo",
    Redo => "dev.quantum.core.redo",
    Commands => "dev.quantum.core.commands",
    Settings => "dev.quantum.core.settings",
    Frame => "dev.quantum.core.frame",
    Front => "dev.quantum.core.front",
    Right => "dev.quantum.core.right",
    Top => "dev.quantum.core.top",
    Grid => "dev.quantum.core.grid",
    DebugOverlay => "dev.quantum.core.debug_overlay",
    Theme => "dev.quantum.core.theme",
    FullScreen => "dev.quantum.core.fullscreen",
    Place => "dev.quantum.tools.place",
    Erase => "dev.quantum.tools.erase",
    Paint => "dev.quantum.tools.paint",
    Fill => "dev.quantum.tools.fill",
    Box => "dev.quantum.tools.box",
    Picker => "dev.quantum.tools.picker",
    MirrorX => "dev.quantum.mirror.mirror_x",
    MirrorY => "dev.quantum.mirror.mirror_y",
    MirrorZ => "dev.quantum.mirror.mirror_z",
    ToolPicker => "dev.quantum.tools.tool_picker",
    Comment => "dev.quantum.comments.comment",
    AddLayer => "dev.quantum.core.add_layer",
    ShowPalette => "dev.quantum.palette.show_palette",
    Palette => "dev.quantum.palette.palette",
    ManagePalettes => "dev.quantum.palette.manage_palettes",
    Agent => "dev.quantum.agent.agent",
    Extensions => "dev.quantum.agent.extensions",
    Minimize => "dev.quantum.core.minimize",
    ZoomWindow => "dev.quantum.core.zoom_window",
    Feedback => "dev.quantum.core.feedback",
    Help => "dev.quantum.core.help",
    About => "dev.quantum.core.about",
    CheckUpdates => "dev.quantum.core.check_updates",
    Hide => "dev.quantum.core.hide",
    HideOthers => "dev.quantum.core.hide_others",
    ShowAll => "dev.quantum.core.show_all",
}

/// A native recent-menu item carries its path, never an unstable list index.
#[derive(Clone, Debug, PartialEq, Eq, Action)]
#[action(no_json)]
pub struct OpenRecentProject {
    pub path: std::path::PathBuf,
}

/// Immutable presentation of the host's enabled commands. The host owns plugin
/// lifecycle; the shell only needs data, so no Send/Sync requirement is imposed
/// on plugin implementations. Rebuild this view when enablement is introduced.
struct ShellCommands {
    entries: Vec<(Op, CommandContribution)>,
}

impl ShellCommands {
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: built-in command has an Op"
    )]
    fn from_host(host: &Host) -> Self {
        let mut entries: Vec<_> = host
            .registry()
            .commands()
            .map(|(_, command)| {
                let op = Op::from_command_id(&command.id).expect("built-in command has an Op");
                (op, command.clone())
            })
            .collect();
        entries.sort_by_key(|(_, command)| command.order);
        Self { entries }
    }

    #[expect(
        clippy::expect_used,
        reason = "internal invariant: command is enabled in the app host"
    )]
    fn command(&self, op: Op) -> &CommandContribution {
        &self
            .entries
            .iter()
            .find(|(_, command)| command.id == op.command_id())
            .expect("command is enabled in the app host")
            .1
    }

    fn available_on(&self, op: Op, platform: ShortcutPlatform) -> bool {
        match op {
            Op::Quit | Op::Minimize | Op::ZoomWindow | Op::FullScreen => true,
            _ if self.command(op).menu.as_deref() == Some("Quantum") => {
                platform == ShortcutPlatform::Apple
            }
            _ => true,
        }
    }

    fn shortcut_sheet(&self, platform: ShortcutPlatform) -> impl Iterator<Item = Op> + '_ {
        self.entries
            .iter()
            .map(|(op, _)| *op)
            .filter(move |op| self.available_on(*op, platform))
    }

    fn search(&self, query: &str, tools_only: bool, platform: ShortcutPlatform) -> Vec<Op> {
        let query = query.to_lowercase();
        self.shortcut_sheet(platform)
            .filter(|op| {
                *op != Op::Commands
                    && (!tools_only || op.is_tool())
                    && (label(self.command(*op)).to_lowercase().contains(&query)
                        || (*op == Op::FullScreen
                            && [
                                "fullscreen",
                                "full screen",
                                "full",
                                "fs",
                                "toggle full",
                                "enter full screen",
                                "exit full screen",
                            ]
                            .iter()
                            .any(|keyword| keyword.contains(query.trim()))))
            })
            .collect()
    }

    fn menu_groups(&self) -> Vec<(&str, Vec<Op>)> {
        let mut menus: Vec<(&str, Vec<Op>)> = Vec::new();
        for op in self.shortcut_sheet(ShortcutPlatform::WindowsLinux) {
            let Some(name) = self.command(op).menu.as_deref() else {
                continue;
            };
            if let Some((_, ops)) = menus.iter_mut().find(|(menu, _)| *menu == name) {
                ops.push(op);
            } else {
                menus.push((name, vec![op]));
            }
        }
        menus
    }

    fn binding(&self, op: Op, platform: ShortcutPlatform) -> Option<KeyBinding> {
        let command = self.command(op);
        command.default_keybinding.as_deref().map(|key| {
            // Nested numeric fields cannot change an ancestor's context.
            // Exclude the focused editor at binding resolution so printable
            // tool shortcuts still reach the platform text input handler.
            let context = match command.context.as_deref() {
                Some("Editor") => Some("Editor && !SharedTextInput"),
                other => other,
            };
            KeyBinding::new(&platform.binding_key(key), op, context)
        })
    }
}

// Label keys are English fallbacks until localization is introduced.
fn label(command: &CommandContribution) -> &str {
    &command.label_key
}

fn command_menu(command: &CommandContribution, apple: bool) -> &str {
    if apple && command.application_section.is_some() {
        "Quantum"
    } else {
        command.menu.as_deref().unwrap_or_default()
    }
}

static SHELL: LazyLock<ShellCommands> =
    LazyLock::new(|| ShellCommands::from_host(&quantum_app::app_host()));

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
    fn command(self) -> &'static CommandContribution {
        SHELL.command(self)
    }

    pub fn label(self) -> &'static str {
        label(self.command())
    }

    /// Window-state label shared by native menus and command palette rows.
    pub fn display_label(self, fullscreen: bool) -> &'static str {
        if self == Self::FullScreen {
            if fullscreen {
                "Exit Full Screen"
            } else {
                "Enter Full Screen"
            }
        } else {
            self.label()
        }
    }

    /// Bundled Lucide asset shared by menus, search and toolbar controls.
    pub fn icon(self) -> &'static str {
        self.command().icon.as_deref().unwrap_or_default()
    }

    pub fn key(self) -> Option<&'static str> {
        self.command().default_keybinding.as_deref()
    }

    pub fn menu(self, apple: bool) -> &'static str {
        command_menu(self.command(), apple)
    }

    #[cfg(test)]
    fn available_on(self, platform: ShortcutPlatform) -> bool {
        SHELL.available_on(self, platform)
    }

    pub fn shortcut(self) -> String {
        self.key()
            .map(|key| format_shortcut(key, ShortcutPlatform::current()))
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn binding(self, platform: ShortcutPlatform) -> Option<KeyBinding> {
        SHELL.binding(self, platform)
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
                        Err("Updates are unavailable in this build.")
                    }
                    Self::Welcome => Ok(()),
                    Self::Feedback
                    | Self::About
                    | Self::Settings
                    | Self::CheckUpdates
                    | Self::Quit
                    | Self::Hide
                    | Self::HideOthers
                    | Self::ShowAll
                    | Self::Minimize
                    | Self::ZoomWindow
                    | Self::FullScreen
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
                Err("Updates are unavailable in this build.")
            }
            _ => Ok(()),
        }
    }

    /// Check the live registry as well as editor state for installed shell actions.
    pub fn availability_in(self, host: &Host, context: CommandContext) -> Result<(), &'static str> {
        if host.registry().get(self.command_id()).is_none() {
            return Err("This command’s plugin is disabled");
        }
        self.availability(context)
    }
}

/// In-window menus use the same placement on every host.
pub static MENUS: LazyLock<Vec<(&str, Vec<Op>)>> = LazyLock::new(|| SHELL.menu_groups());

/// Live items for an in-window menu; disabled plugins contribute no entries.
pub fn menu_items(index: usize, host: &Host) -> Vec<Op> {
    MENUS[index]
        .1
        .iter()
        .copied()
        .filter(|op| host.registry().get(op.command_id()).is_some())
        .collect()
}

pub fn search(query: &str, tools_only: bool) -> Vec<Op> {
    search_on(query, tools_only, ShortcutPlatform::current())
}

fn search_on(query: &str, tools_only: bool, platform: ShortcutPlatform) -> Vec<Op> {
    SHELL.search(query, tools_only, platform)
}

/// Every available command appears, including commands without a keyboard binding.
pub fn shortcut_sheet(platform: ShortcutPlatform) -> impl Iterator<Item = Op> {
    SHELL.shortcut_sheet(platform)
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
        let rows = match op.command().menu.as_deref() {
            Some("Tools") => &mut tools,
            Some("View") => &mut navigate,
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

pub use quantum_ui::ClipboardOp;

/// Build both initial and live menus from the same action list.
pub fn menus(selected: impl Fn(Op) -> bool, enabled: impl Fn(Op) -> bool) -> Vec<Menu> {
    menus_for_platform(cfg!(target_os = "macos"), selected, enabled)
}

/// Build native menus for the active Settings window.
pub fn menus_for_window(
    fullscreen: bool,
    selected: impl Fn(Op) -> bool,
    enabled: impl Fn(Op) -> bool,
) -> Vec<Menu> {
    SHELL.menus(cfg!(target_os = "macos"), fullscreen, selected, enabled)
}

/// Build the editor's native menus from its currently enabled declarations.
pub fn menus_from_host(
    host: &Host,
    fullscreen: bool,
    selected: impl Fn(Op) -> bool,
    enabled: impl Fn(Op) -> bool,
) -> Vec<Menu> {
    ShellCommands::from_host(host).menus(cfg!(target_os = "macos"), fullscreen, selected, enabled)
}

fn menus_for_platform(
    apple: bool,
    selected: impl Fn(Op) -> bool,
    enabled: impl Fn(Op) -> bool,
) -> Vec<Menu> {
    SHELL.menus(apple, false, selected, enabled)
}

impl ShellCommands {
    fn menus(
        &self,
        apple: bool,
        fullscreen: bool,
        selected: impl Fn(Op) -> bool,
        enabled: impl Fn(Op) -> bool,
    ) -> Vec<Menu> {
        let item = |op: Op| {
            let item = match op {
                Op::Undo => {
                    MenuItem::os_action(label(self.command(op)).to_owned(), op, OsAction::Undo)
                }
                Op::Redo => {
                    MenuItem::os_action(label(self.command(op)).to_owned(), op, OsAction::Redo)
                }
                _ => MenuItem::action(op.display_label(fullscreen), op),
            };
            item.checked(selected(op)).disabled(!enabled(op))
        };
        let mut menus = Vec::new();
        if apple {
            let mut items = Vec::new();
            for section in [
                ApplicationMenuSection::About,
                ApplicationMenuSection::Settings,
                ApplicationMenuSection::Visibility,
                ApplicationMenuSection::Quit,
            ] {
                if !items.is_empty() {
                    items.push(MenuItem::separator());
                }
                items.extend(
                    self.shortcut_sheet(ShortcutPlatform::Apple)
                        .filter(|op| self.command(*op).application_section == Some(section))
                        .map(item),
                );
                if section == ApplicationMenuSection::Settings {
                    items.push(MenuItem::separator());
                    items.push(MenuItem::os_submenu(
                        "Services",
                        gpui::SystemMenuType::Services,
                    ));
                }
            }
            menus.push(Menu::new("Quantum").items(items));
        }
        for (name, ops) in self.menu_groups() {
            let mut items: Vec<_> = ops
                .iter()
                .copied()
                .filter(|op| command_menu(self.command(*op), apple) == name)
                .map(item)
                .collect();
            if name == "File" {
                items.retain(|item| {
                    !matches!(item, MenuItem::Action { action, .. }
                    if action.as_any().downcast_ref::<Op>() == Some(&Op::ClearRecents))
                });
                for entry in &mut items {
                    if matches!(entry, MenuItem::Action { action, .. }
                        if action.as_any().downcast_ref::<Op>() == Some(&Op::OpenRecent))
                    {
                        *entry = MenuItem::Submenu(
                            Menu::new(Op::OpenRecent.label()).items([item(Op::ClearRecents)]),
                        );
                    }
                }
            }
            if name == "Edit" {
                items.splice(
                    2..2,
                    [
                        MenuItem::separator(),
                        MenuItem::os_action("Cut", ClipboardOp::Cut, OsAction::Cut),
                        MenuItem::os_action("Copy", ClipboardOp::Copy, OsAction::Copy),
                        MenuItem::os_action("Paste", ClipboardOp::Paste, OsAction::Paste),
                        MenuItem::os_action(
                            "Select All",
                            ClipboardOp::SelectAll,
                            OsAction::SelectAll,
                        ),
                        MenuItem::separator(),
                    ],
                );
            }
            menus.push(Menu::new(name.to_owned()).items(items));
        }
        menus
    }
}

#[cfg(test)]
mod snapshot;

#[cfg(test)]
mod availability_tests;
#[cfg(test)]
mod command_tests;
#[cfg(test)]
#[path = "actions/tests.rs"]
mod test_shell;

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{KeyContext, Keymap};
    #[test]
    fn welcome_and_recent_commands_share_the_registry() {
        let menus = menus_for_platform(true, |_| false, |_| true);
        let file = menus.iter().find(|m| m.name == "File").unwrap();
        let recent = file
            .items
            .iter()
            .find_map(|item| match item {
                MenuItem::Submenu(menu) if menu.name == Op::OpenRecent.label() => Some(menu),
                _ => None,
            })
            .unwrap();
        assert!(
            recent
                .items
                .iter()
                .any(|item| matches!(item, MenuItem::Action { action, .. }
        if action.as_any().downcast_ref::<Op>() == Some(&Op::ClearRecents)))
        );
        let window = menus.iter().find(|m| m.name == "Window").unwrap();
        assert!(
            window
                .items
                .iter()
                .any(|item| matches!(item, MenuItem::Action { action, .. }
        if action.as_any().downcast_ref::<Op>() == Some(&Op::Welcome)))
        );
        for op in [Op::Welcome, Op::OpenRecent, Op::ClearRecents] {
            assert!(search_on(op.label(), false, ShortcutPlatform::Apple).contains(&op));
            assert_eq!(Op::from_command_id(op.command_id()), Some(op));
        }
    }

    #[test]
    fn fullscreen_uses_native_bindings_and_window_state_labels() {
        assert_eq!(Op::FullScreen.display_label(false), "Enter Full Screen");
        assert_eq!(Op::FullScreen.display_label(true), "Exit Full Screen");
        for (platform, key) in [
            (ShortcutPlatform::Apple, "ctrl-cmd-f"),
            (ShortcutPlatform::WindowsLinux, "f11"),
        ] {
            for query in [
                "fullscreen",
                "full screen",
                "full",
                "fs",
                "toggle full",
                "enter full",
                "exit full",
            ] {
                assert_eq!(search_on(query, false, platform), vec![Op::FullScreen]);
            }
            {
                let binding = Op::FullScreen.binding(platform).unwrap();
                assert_eq!(
                    binding.keystrokes(),
                    KeyBinding::new(key, Op::FullScreen, None).keystrokes()
                );
            }
        }
        for fullscreen in [false, true] {
            let menus = SHELL.menus(true, fullscreen, |_| false, |_| true);
            let view = menus
                .iter()
                .find(|menu| menu.name.as_ref() == "View")
                .unwrap();
            assert_eq!(view.items.iter().filter(|item| matches!(item, MenuItem::Action { name, action, .. }
            if action.as_any().downcast_ref::<Op>() == Some(&Op::FullScreen) && name.as_ref() == Op::FullScreen.display_label(fullscreen))).count(), 1);
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
                        Some(
                            Op::Settings | Op::Quit | Op::Hide | Op::HideOthers | Op::FullScreen,
                        ) => true,
                        Some(Op::Welcome) => true,
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
            for binding in &bindings {
                let (matches, _) = keymap.bindings_for_input(
                    binding.keystrokes(),
                    &[
                        KeyContext::parse("Editor").unwrap(),
                        KeyContext::parse("TextField SharedTextInput").unwrap(),
                    ],
                );
                assert!(
                    matches.iter().all(|binding| {
                        binding.predicate().is_none()
                            || binding.action().as_any().is::<ClipboardOp>()
                    }),
                    "document binding leaked into a nested field: {matches:?}"
                );
            }
            for op in Op::ALL {
                assert_eq!(
                bindings
                    .iter()
                    .filter(|binding| binding.action().as_any().downcast_ref::<Op>() == Some(op))
                    .count(),
                usize::from(op.available_on(platform) && op.key().is_some()),
                "{platform:?}: {op:?}"
            );
            }
        }
    }
}
