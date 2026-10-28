//! Native action installation and platform keyboard bindings.
use super::*;
use gpui::App;

pub fn install(cx: &mut App) {
    let platform = ShortcutPlatform::current();
    if cfg!(target_os = "macos") {
        cx.set_menus(menus(
            |_| false,
            |op| op != Op::CheckUpdates || crate::ui::updates_available(),
        ));
    }
    cx.bind_keys(key_bindings(platform));
    quantum_ui::install_text_field_bindings_for_platform(platform == ShortcutPlatform::Apple, cx);
}

pub(super) fn key_bindings(platform: ShortcutPlatform) -> Vec<KeyBinding> {
    SHELL.key_bindings(platform)
}

impl ShellCommands {
    pub(super) fn key_bindings(&self, platform: ShortcutPlatform) -> Vec<KeyBinding> {
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
