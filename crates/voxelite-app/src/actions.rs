//! Command metadata and availability shared by menus, search, help and documentation.
use crate::shortcuts::{ShortcutPlatform, format_shortcut};
use gpui::{Action, App, KeyBinding, Menu, MenuItem, OsAction};
use std::sync::LazyLock;
use voxelite_plugin_api::{ApplicationMenuSection, CommandContribution};
use voxelite_plugin_host::Host;

// Dispatch identities only. All presentation metadata belongs to contributions.
macro_rules! operations {
    ($($(#[$attr:meta])* $op:ident => $id:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Action)]
        #[action(no_json)]
        pub enum Op { $($(#[$attr])* $op),+ }

        impl Op {
            #[cfg(test)]
            pub const ALL: &[Self] = &[$($(#[$attr])* Self::$op),+];

            fn command_id(self) -> &'static str {
                match self { $($(#[$attr])* Self::$op => $id),+ }
            }

            fn from_command_id(id: &str) -> Option<Self> {
                match id {
                    $($(#[$attr])* $id => Some(Self::$op),)+
                    _ => None,
                }
            }
        }
    };
}

operations! {
    New => "dev.voxelite.core.new",
    Open => "dev.voxelite.core.open",
    Save => "dev.voxelite.core.save",
    SaveAs => "dev.voxelite.core.save_as",
    #[cfg(target_family = "wasm")]
    ExportGlb => "dev.voxelite.core.export_glb",
    #[cfg(target_family = "wasm")]
    ExportPng => "dev.voxelite.core.export_png",
    Quit => "dev.voxelite.core.quit",
    Undo => "dev.voxelite.core.undo",
    Redo => "dev.voxelite.core.redo",
    Commands => "dev.voxelite.core.commands",
    Settings => "dev.voxelite.core.settings",
    Frame => "dev.voxelite.core.frame",
    Front => "dev.voxelite.core.front",
    Right => "dev.voxelite.core.right",
    Top => "dev.voxelite.core.top",
    Grid => "dev.voxelite.core.grid",
    Theme => "dev.voxelite.core.theme",
    Place => "dev.voxelite.tools.place",
    Erase => "dev.voxelite.tools.erase",
    Paint => "dev.voxelite.tools.paint",
    Fill => "dev.voxelite.tools.fill",
    Box => "dev.voxelite.tools.box",
    Picker => "dev.voxelite.tools.picker",
    MirrorX => "dev.voxelite.mirror.mirror_x",
    MirrorY => "dev.voxelite.mirror.mirror_y",
    MirrorZ => "dev.voxelite.mirror.mirror_z",
    ToolPicker => "dev.voxelite.tools.tool_picker",
    Comment => "dev.voxelite.comments.comment",
    AddLayer => "dev.voxelite.core.add_layer",
    ShowPalette => "dev.voxelite.palette.show_palette",
    Palette => "dev.voxelite.palette.palette",
    ManagePalettes => "dev.voxelite.palette.manage_palettes",
    Agent => "dev.voxelite.agent.agent",
    Extensions => "dev.voxelite.agent.extensions",
    Minimize => "dev.voxelite.core.minimize",
    ZoomWindow => "dev.voxelite.core.zoom_window",
    Help => "dev.voxelite.core.help",
    About => "dev.voxelite.core.about",
    CheckUpdates => "dev.voxelite.core.check_updates",
    Hide => "dev.voxelite.core.hide",
    HideOthers => "dev.voxelite.core.hide_others",
    ShowAll => "dev.voxelite.core.show_all",
}

/// Immutable presentation of the host's enabled commands. The host owns plugin
/// lifecycle; the shell only needs data, so no Send/Sync requirement is imposed
/// on plugin implementations. Rebuild this view when enablement is introduced.
struct ShellCommands {
    entries: Vec<(Op, CommandContribution)>,
}

impl ShellCommands {
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
            Op::Quit | Op::Minimize | Op::ZoomWindow => !cfg!(target_family = "wasm"),
            _ if self.command(op).menu.as_deref() == Some("Voxelite") => {
                !cfg!(target_family = "wasm") && platform == ShortcutPlatform::Apple
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
                    && label(self.command(*op)).to_lowercase().contains(&query)
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
        command
            .default_keybinding
            .as_deref()
            .map(|key| KeyBinding::new(&platform.binding_key(key), op, command.context.as_deref()))
    }
}

// Label keys are English fallbacks until localization is introduced.
fn label(command: &CommandContribution) -> &str {
    &command.label_key
}

fn command_menu(command: &CommandContribution, apple: bool) -> &str {
    if apple && command.application_section.is_some() {
        "Voxelite"
    } else {
        command.menu.as_deref().unwrap_or_default()
    }
}

static SHELL: LazyLock<ShellCommands> =
    LazyLock::new(|| ShellCommands::from_host(&voxelite_app::app_host()));

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

    /// Check the live registry as well as editor state for installed shell actions.
    pub fn availability_in(self, host: &Host, context: CommandContext) -> Result<(), &'static str> {
        if host.registry().get(self.command_id()).is_none() {
            return Err("This command’s plugin is disabled");
        }
        self.availability(context)
    }
}

/// In-window menus use the same placement on every host, including Apple browsers.
pub static MENUS: LazyLock<Vec<(&str, Vec<Op>)>> = LazyLock::new(|| SHELL.menu_groups());

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
    SHELL.menus(apple, selected, enabled)
}

impl ShellCommands {
    fn menus(
        &self,
        apple: bool,
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
                _ => MenuItem::action(label(self.command(op)).to_owned(), op),
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
            menus.push(Menu::new("Voxelite").items(items));
        }
        for (name, ops) in self.menu_groups() {
            let mut items: Vec<_> = ops
                .iter()
                .copied()
                .filter(|op| command_menu(self.command(*op), apple) == name)
                .map(item)
                .collect();
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

pub fn install(cx: &mut App) {
    let platform = ShortcutPlatform::current();
    if cfg!(target_os = "macos") {
        cx.set_menus(menus(|_| false, |_| true));
    }
    cx.bind_keys(key_bindings(platform));
}

fn key_bindings(platform: ShortcutPlatform) -> Vec<KeyBinding> {
    SHELL.key_bindings(platform)
}

impl ShellCommands {
    fn key_bindings(&self, platform: ShortcutPlatform) -> Vec<KeyBinding> {
        self.shortcut_sheet(platform)
            .filter_map(|op| self.binding(op, platform))
            .chain(
                [
                    ("secondary-x", ClipboardOp::Cut),
                    ("secondary-c", ClipboardOp::Copy),
                    ("secondary-v", ClipboardOp::Paste),
                    ("secondary-a", ClipboardOp::SelectAll),
                ]
                .into_iter()
                .map(|(key, op)| {
                    KeyBinding::new(&platform.binding_key(key), op, Some("TextField"))
                }),
            )
            .collect()
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod snapshot;

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AssetSource, KeyContext, Keymap, Keystroke};
    use std::collections::HashSet;

    #[test]
    fn every_op_maps_to_exactly_one_host_command_and_back() {
        let host = voxelite_app::app_host();
        let commands: Vec<_> = host.registry().commands().collect();
        assert_eq!(commands.len(), Op::ALL.len());
        let mut ids = HashSet::new();
        for op in Op::ALL {
            assert!(ids.insert(op.command_id()), "duplicate mapping: {op:?}");
            assert_eq!(Op::from_command_id(op.command_id()), Some(*op));
            assert_eq!(
                commands
                    .iter()
                    .filter(|(_, command)| command.id == op.command_id())
                    .count(),
                1,
                "{op:?}"
            );
        }
        for (owner, command) in commands {
            let op = Op::from_command_id(&command.id).expect("command must have a dispatch Op");
            assert_eq!(op.command_id(), command.id);
            assert!(command.id.starts_with(&format!("{owner}.")));
        }
        assert_eq!(Op::from_command_id("dev.voxelite.unknown.command"), None);
    }

    #[test]
    fn disabling_plugins_removes_their_commands_from_every_shell_list() {
        let mut host = voxelite_app::app_host();
        let plugins: Vec<_> = host
            .plugins()
            .filter(|(manifest, _)| manifest.can_disable)
            .map(|(manifest, _)| manifest.id.to_string())
            .collect();
        for plugin in plugins {
            let owned: Vec<_> = host
                .registry()
                .commands()
                .filter(|(owner, _)| owner.as_str() == plugin)
                .map(|(_, command)| Op::from_command_id(&command.id).unwrap())
                .collect();
            assert!(!owned.is_empty());
            host.disable(&plugin).unwrap();
            let shell = ShellCommands::from_host(&host);
            for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
                assert_eq!(
                    shell.shortcut_sheet(platform).collect::<Vec<_>>(),
                    SHELL
                        .shortcut_sheet(platform)
                        .filter(|op| !owned.contains(op))
                        .collect::<Vec<_>>()
                );
                for tools_only in [false, true] {
                    assert_eq!(
                        shell.search("", tools_only, platform),
                        SHELL
                            .search("", tools_only, platform)
                            .into_iter()
                            .filter(|op| !owned.contains(op))
                            .collect::<Vec<_>>()
                    );
                }
                let expected = SHELL
                    .key_bindings(platform)
                    .into_iter()
                    .filter(|binding| {
                        binding
                            .action()
                            .as_any()
                            .downcast_ref::<Op>()
                            .is_none_or(|op| !owned.contains(op))
                    })
                    .collect::<Vec<_>>();
                let actual = shell.key_bindings(platform);
                assert_eq!(actual.len(), expected.len());
                for (actual, expected) in actual.iter().zip(&expected) {
                    assert!(actual.action().partial_eq(expected.action()));
                    assert_eq!(actual.keystrokes(), expected.keystrokes());
                    assert_eq!(actual.predicate(), expected.predicate());
                }
                let menus = shell.menus(platform == ShortcutPlatform::Apple, |_| false, |_| true);
                let menu_ops: Vec<_> = menus
                    .iter()
                    .flat_map(|menu| &menu.items)
                    .filter_map(|item| match item {
                        MenuItem::Action { action, .. } => {
                            action.as_any().downcast_ref::<Op>().copied()
                        }
                        _ => None,
                    })
                    .collect();
                assert!(owned.iter().all(|op| !menu_ops.contains(op)));
                for op in shell.shortcut_sheet(platform) {
                    assert_eq!(
                        menu_ops.iter().filter(|entry| **entry == op).count(),
                        1,
                        "{op:?}"
                    );
                }
                assert!(menus.iter().all(|menu| !menu.items.is_empty()));
            }
            assert!(
                shell
                    .menu_groups()
                    .iter()
                    .flat_map(|(_, ops)| ops)
                    .all(|op| !owned.contains(op))
            );
            host.enable(&plugin).unwrap();
            assert_eq!(ShellCommands::from_host(&host).entries, SHELL.entries);
        }
    }

    #[test]
    fn live_registry_disables_installed_mirror_actions_without_disabling_tools() {
        let mut host = voxelite_app::app_host();
        let context = CommandContext::Editor(EditorState::default());
        for op in [Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
            assert!(op.availability_in(&host, context).is_ok());
        }
        host.disable("dev.voxelite.mirror").unwrap();
        for op in [Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
            assert_eq!(
                op.availability_in(&host, context),
                Err("This command’s plugin is disabled")
            );
        }
        for op in [
            Op::Place,
            Op::Paint,
            Op::Erase,
            Op::Box,
            Op::Fill,
            Op::Picker,
            Op::New,
        ] {
            assert!(op.availability_in(&host, context).is_ok());
        }
        host.enable("dev.voxelite.mirror").unwrap();
        for op in [Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
            assert!(op.availability_in(&host, context).is_ok());
        }
    }

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
