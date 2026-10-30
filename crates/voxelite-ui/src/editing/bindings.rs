//! The authoritative binding table, shared by GPUI installation and tests.
use super::Unit;
/// Platform clipboard operation; the adapter owns the system clipboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipboardCommand {
    /// Copy and remove the selection.
    Cut,
    /// Copy the selection.
    Copy,
    /// Insert normalized clipboard text.
    Paste,
    /// Select the whole line.
    SelectAll,
}
/// A renderer-independent field command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Boundary, forward direction, and whether to extend selection.
    Move(Unit, bool, bool),
    /// Boundary and forward direction.
    Delete(Unit, bool),
    /// Upward direction, Shift, Option/fine. Moves to line ends in text fields.
    Vertical(bool, bool, bool),
    /// System clipboard operation.
    Clipboard(ClipboardCommand),
    /// Restore a field transaction; always consume, including with empty history.
    Undo,
    /// Reapply a field transaction.
    Redo,
    /// Commit the value.
    Commit,
    /// Apply the field's explicit cancellation policy and blur.
    Escape,
    /// Move focus; true means backwards. The containing focus scope handles Tab.
    Tab(bool),
    /// Open the editing context menu.
    Menu,
}
/// One complete key binding in GPUI keystroke notation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    /// Key chord.
    pub key: &'static str,
    /// Field command.
    pub command: Command,
}
/// Bindings present on every platform.
pub const COMMON_BINDINGS: &[Binding] = &[
    Binding {
        key: "left",
        command: Command::Move(Unit::Grapheme, false, false),
    },
    Binding {
        key: "shift-left",
        command: Command::Move(Unit::Grapheme, false, true),
    },
    Binding {
        key: "right",
        command: Command::Move(Unit::Grapheme, true, false),
    },
    Binding {
        key: "shift-right",
        command: Command::Move(Unit::Grapheme, true, true),
    },
    Binding {
        key: "home",
        command: Command::Move(Unit::Line, false, false),
    },
    Binding {
        key: "shift-home",
        command: Command::Move(Unit::Line, false, true),
    },
    Binding {
        key: "end",
        command: Command::Move(Unit::Line, true, false),
    },
    Binding {
        key: "shift-end",
        command: Command::Move(Unit::Line, true, true),
    },
    Binding {
        key: "up",
        command: Command::Vertical(true, false, false),
    },
    Binding {
        key: "shift-up",
        command: Command::Vertical(true, true, false),
    },
    Binding {
        key: "alt-up",
        command: Command::Vertical(true, false, true),
    },
    Binding {
        key: "alt-shift-up",
        command: Command::Vertical(true, true, true),
    },
    Binding {
        key: "down",
        command: Command::Vertical(false, false, false),
    },
    Binding {
        key: "shift-down",
        command: Command::Vertical(false, true, false),
    },
    Binding {
        key: "alt-down",
        command: Command::Vertical(false, false, true),
    },
    Binding {
        key: "alt-shift-down",
        command: Command::Vertical(false, true, true),
    },
    Binding {
        key: "backspace",
        command: Command::Delete(Unit::Grapheme, false),
    },
    Binding {
        key: "delete",
        command: Command::Delete(Unit::Grapheme, true),
    },
    Binding {
        key: "enter",
        command: Command::Commit,
    },
    Binding {
        key: "escape",
        command: Command::Escape,
    },
    Binding {
        key: "tab",
        command: Command::Tab(false),
    },
    Binding {
        key: "shift-tab",
        command: Command::Tab(true),
    },
    Binding {
        key: "menu",
        command: Command::Menu,
    },
    Binding {
        key: "shift-f10",
        command: Command::Menu,
    },
];
/// macOS Command, Option and Emacs editing bindings.
pub const MAC_BINDINGS: &[Binding] = &[
    Binding {
        key: "alt-left",
        command: Command::Move(Unit::Word, false, false),
    },
    Binding {
        key: "alt-shift-left",
        command: Command::Move(Unit::Word, false, true),
    },
    Binding {
        key: "alt-right",
        command: Command::Move(Unit::Word, true, false),
    },
    Binding {
        key: "alt-shift-right",
        command: Command::Move(Unit::Word, true, true),
    },
    Binding {
        key: "alt-backspace",
        command: Command::Delete(Unit::Word, false),
    },
    Binding {
        key: "alt-delete",
        command: Command::Delete(Unit::Word, true),
    },
    Binding {
        key: "cmd-left",
        command: Command::Move(Unit::Line, false, false),
    },
    Binding {
        key: "cmd-shift-left",
        command: Command::Move(Unit::Line, false, true),
    },
    Binding {
        key: "cmd-right",
        command: Command::Move(Unit::Line, true, false),
    },
    Binding {
        key: "cmd-shift-right",
        command: Command::Move(Unit::Line, true, true),
    },
    Binding {
        key: "cmd-backspace",
        command: Command::Delete(Unit::Line, false),
    },
    Binding {
        key: "cmd-delete",
        command: Command::Delete(Unit::Line, true),
    },
    Binding {
        key: "cmd-x",
        command: Command::Clipboard(ClipboardCommand::Cut),
    },
    Binding {
        key: "cmd-c",
        command: Command::Clipboard(ClipboardCommand::Copy),
    },
    Binding {
        key: "cmd-v",
        command: Command::Clipboard(ClipboardCommand::Paste),
    },
    Binding {
        key: "cmd-a",
        command: Command::Clipboard(ClipboardCommand::SelectAll),
    },
    Binding {
        key: "cmd-z",
        command: Command::Undo,
    },
    Binding {
        key: "cmd-shift-z",
        command: Command::Redo,
    },
    Binding {
        key: "ctrl-a",
        command: Command::Move(Unit::Line, false, false),
    },
    Binding {
        key: "ctrl-shift-a",
        command: Command::Move(Unit::Line, false, true),
    },
    Binding {
        key: "ctrl-e",
        command: Command::Move(Unit::Line, true, false),
    },
    Binding {
        key: "ctrl-shift-e",
        command: Command::Move(Unit::Line, true, true),
    },
    Binding {
        key: "ctrl-b",
        command: Command::Move(Unit::Grapheme, false, false),
    },
    Binding {
        key: "ctrl-shift-b",
        command: Command::Move(Unit::Grapheme, false, true),
    },
    Binding {
        key: "ctrl-f",
        command: Command::Move(Unit::Grapheme, true, false),
    },
    Binding {
        key: "ctrl-shift-f",
        command: Command::Move(Unit::Grapheme, true, true),
    },
    Binding {
        key: "ctrl-k",
        command: Command::Delete(Unit::Line, true),
    },
    Binding {
        key: "ctrl-d",
        command: Command::Delete(Unit::Grapheme, true),
    },
    Binding {
        key: "ctrl-h",
        command: Command::Delete(Unit::Grapheme, false),
    },
];
/// Linux, Windows and browser Control bindings.
pub const OTHER_BINDINGS: &[Binding] = &[
    Binding {
        key: "ctrl-left",
        command: Command::Move(Unit::Word, false, false),
    },
    Binding {
        key: "ctrl-shift-left",
        command: Command::Move(Unit::Word, false, true),
    },
    Binding {
        key: "ctrl-right",
        command: Command::Move(Unit::Word, true, false),
    },
    Binding {
        key: "ctrl-shift-right",
        command: Command::Move(Unit::Word, true, true),
    },
    Binding {
        key: "ctrl-backspace",
        command: Command::Delete(Unit::Word, false),
    },
    Binding {
        key: "ctrl-delete",
        command: Command::Delete(Unit::Word, true),
    },
    Binding {
        key: "ctrl-x",
        command: Command::Clipboard(ClipboardCommand::Cut),
    },
    Binding {
        key: "ctrl-c",
        command: Command::Clipboard(ClipboardCommand::Copy),
    },
    Binding {
        key: "ctrl-v",
        command: Command::Clipboard(ClipboardCommand::Paste),
    },
    Binding {
        key: "ctrl-a",
        command: Command::Clipboard(ClipboardCommand::SelectAll),
    },
    Binding {
        key: "ctrl-z",
        command: Command::Undo,
    },
    Binding {
        key: "ctrl-shift-z",
        command: Command::Redo,
    },
    Binding {
        key: "ctrl-y",
        command: Command::Redo,
    },
    Binding {
        key: "ctrl-home",
        command: Command::Move(Unit::Line, false, false),
    },
    Binding {
        key: "ctrl-end",
        command: Command::Move(Unit::Line, true, false),
    },
    Binding {
        key: "ctrl-shift-home",
        command: Command::Move(Unit::Line, false, true),
    },
    Binding {
        key: "ctrl-shift-end",
        command: Command::Move(Unit::Line, true, true),
    },
];
/// Iterate the full binding table for an explicit platform.
pub fn bindings(mac: bool) -> impl Iterator<Item = &'static Binding> {
    COMMON_BINDINGS
        .iter()
        .chain(if mac { MAC_BINDINGS } else { OTHER_BINDINGS })
}
