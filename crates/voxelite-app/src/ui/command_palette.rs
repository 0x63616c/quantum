//! Command palette state and input policy. Search stays in the command registry.
use super::EditorView;
use crate::{
    actions::{self, Op},
    shortcuts::{ShortcutPlatform, navigation_key},
};
use gpui::{Context, Keystroke, ScrollHandle, Window};

#[derive(Default)]
pub(super) struct CommandPalette {
    open: bool,
    tools_only: bool,
    query: String,
    index: usize,
    scroll: ScrollHandle,
}

impl CommandPalette {
    pub(super) fn open(&mut self, tools_only: bool) {
        self.tools_only = tools_only;
        self.open = true;
        self.query.clear();
        self.index = 0;
    }

    pub(super) fn close(&mut self) {
        self.open = false;
    }
    pub(super) fn is_open(&self) -> bool {
        self.open
    }
    pub(super) fn tools_only(&self) -> bool {
        self.tools_only
    }
    pub(super) fn query(&self) -> &str {
        &self.query
    }
    pub(super) fn index(&self) -> usize {
        self.index
    }
    pub(super) fn scroll(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub(super) fn filtered_actions(&self, host: &voxelite_plugin_host::Host) -> Vec<Op> {
        actions::search(&self.query, self.tools_only)
            .into_iter()
            .filter(|op| host.registry().get(op.command_id()).is_some())
            .collect()
    }

    // Escape and focus routing belong to the surrounding modal shell. Return
    // Enter's command without closing: invoke still owns availability checks.
    fn key(
        &mut self,
        stroke: &Keystroke,
        platform: ShortcutPlatform,
        host: &voxelite_plugin_host::Host,
    ) -> Option<Op> {
        let key = navigation_key(stroke).unwrap_or_default();
        let count = self.filtered_actions(host).len();
        self.index = self.index.min(count.saturating_sub(1));
        match key {
            "backspace" => {
                self.query.pop();
                self.index = 0;
            }
            "down" => self.index = (self.index + 1).min(count.saturating_sub(1)),
            "up" => self.index = self.index.saturating_sub(1),
            "enter" => return self.filtered_actions(host).get(self.index).copied(),
            _ => {
                if let Some(text) = &stroke.key_char
                    && self.query.len() + text.len() <= 128
                    && !text.chars().any(char::is_control)
                    && platform.allows_text(stroke.modifiers)
                {
                    self.query.push_str(text);
                    self.index = 0;
                }
            }
        }
        None
    }

    fn scroll_to_selection(&self) {
        self.scroll.scroll_to_item(self.index);
    }
}

impl EditorView {
    pub(super) fn command_palette_key(
        &mut self,
        stroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let op = self.commands.key(
            stroke,
            ShortcutPlatform::current(),
            &self.editor.plugin_host(),
        );
        if let Some(op) = op {
            self.invoke(op, window, cx);
        }
        // Invoking Commands/ToolPicker can reopen/reset the palette. Scroll
        // after dispatch so the newly selected row wins, as before extraction.
        self.commands.scroll_to_selection();
        cx.stop_propagation();
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandPalette, Op, ShortcutPlatform};
    use gpui::{Keystroke, Modifiers, point, px};

    #[test]
    fn disabled_palette_commands_disappear_from_search_and_keyboard_dispatch() {
        let mut host = voxelite_app::app_host();
        let mut palette = CommandPalette::default();
        palette.open(false);
        palette.query = "palette".into();
        assert_eq!(palette.filtered_actions(&host).len(), 3);
        host.disable("dev.voxelite.palette").unwrap();
        assert!(palette.filtered_actions(&host).is_empty());
        assert_eq!(
            palette.key(
                &Keystroke::parse("enter").unwrap(),
                ShortcutPlatform::current(),
                &host
            ),
            None
        );
        host.enable("dev.voxelite.palette").unwrap();
        assert_eq!(palette.filtered_actions(&host).len(), 3);
    }

    fn key(palette: &mut CommandPalette, key: &str) -> Option<Op> {
        palette.key(
            &Keystroke::parse(key).unwrap(),
            ShortcutPlatform::current(),
            &voxelite_app::app_host(),
        )
    }

    fn text(
        palette: &mut CommandPalette,
        text: &str,
        modifiers: Modifiers,
        platform: ShortcutPlatform,
    ) {
        palette.key(
            &Keystroke {
                key: "text".into(),
                key_char: Some(text.into()),
                modifiers,
            },
            platform,
            &voxelite_app::app_host(),
        );
    }

    #[test]
    fn opening_resets_query_and_selection_but_preserves_scroll_handle() {
        let mut palette = CommandPalette::default();
        assert!(!palette.is_open());
        palette.open(false);
        text(
            &mut palette,
            "view",
            Modifiers::none(),
            ShortcutPlatform::current(),
        );
        key(&mut palette, "down");
        palette.scroll().set_offset(point(px(0.), px(-100.)));
        palette.close();
        assert_eq!(palette.query(), "view");
        palette.open(true);
        assert!(palette.is_open());
        assert!(palette.tools_only());
        assert_eq!(palette.query(), "");
        assert_eq!(palette.index(), 0);
        assert_eq!(palette.scroll().offset(), point(px(0.), px(-100.)));
        assert_eq!(
            palette.filtered_actions(&voxelite_app::app_host()),
            crate::actions::search("", true)
        );
    }

    #[test]
    fn navigation_clamps_and_enter_returns_registry_order_without_closing() {
        let mut palette = CommandPalette::default();
        palette.open(false);
        let matches = palette.filtered_actions(&voxelite_app::app_host());
        assert_eq!(matches, crate::actions::search("", false));
        key(&mut palette, "up");
        assert_eq!(palette.index(), 0);
        assert_eq!(key(&mut palette, "enter"), Some(matches[0]));
        assert!(palette.is_open());
        for _ in 0..matches.len() + 3 {
            key(&mut palette, "down");
        }
        assert_eq!(palette.index(), matches.len() - 1);
        assert_eq!(key(&mut palette, "enter"), matches.last().copied());
        key(&mut palette, "up");
        assert_eq!(palette.index(), matches.len() - 2);
    }

    #[test]
    fn query_edits_reset_selection_and_empty_results_do_not_dispatch() {
        let mut palette = CommandPalette::default();
        palette.open(false);
        key(&mut palette, "down");
        text(
            &mut palette,
            "zzzzzz",
            Modifiers::none(),
            ShortcutPlatform::current(),
        );
        assert_eq!(palette.index(), 0);
        assert!(
            palette
                .filtered_actions(&voxelite_app::app_host())
                .is_empty()
        );
        key(&mut palette, "down");
        key(&mut palette, "up");
        assert_eq!(palette.index(), 0);
        assert_eq!(key(&mut palette, "enter"), None);
        for _ in 0..7 {
            key(&mut palette, "backspace");
        }
        assert_eq!(palette.query(), "");
        assert!(
            !palette
                .filtered_actions(&voxelite_app::app_host())
                .is_empty()
        );
    }

    #[test]
    fn unicode_uses_byte_limit_and_backspace_removes_one_scalar() {
        let mut palette = CommandPalette::default();
        let platform = ShortcutPlatform::current();
        text(&mut palette, &"é".repeat(63), Modifiers::none(), platform);
        text(&mut palette, "abc", Modifiers::none(), platform);
        assert_eq!(palette.query().len(), 126);
        text(&mut palette, "é", Modifiers::none(), platform);
        assert_eq!(palette.query().len(), 128);
        text(&mut palette, "x", Modifiers::none(), platform);
        assert_eq!(palette.query().len(), 128);
        key(&mut palette, "backspace");
        assert_eq!(palette.query(), "é".repeat(63));
        text(&mut palette, "x\n", Modifiers::none(), platform);
        assert_eq!(palette.query().len(), 126);
    }

    #[test]
    fn text_modifiers_and_modified_navigation_preserve_host_policy() {
        for (platform, modifiers, allowed) in [
            (ShortcutPlatform::Apple, Modifiers::alt(), true),
            (ShortcutPlatform::Apple, Modifiers::control(), false),
            (ShortcutPlatform::Apple, Modifiers::command(), false),
            (
                ShortcutPlatform::WindowsLinux,
                Modifiers {
                    control: true,
                    alt: true,
                    ..Modifiers::none()
                },
                true,
            ),
            (ShortcutPlatform::WindowsLinux, Modifiers::control(), false),
            (ShortcutPlatform::WindowsLinux, Modifiers::alt(), false),
            (ShortcutPlatform::WindowsLinux, Modifiers::shift(), true),
            (
                ShortcutPlatform::Apple,
                Modifiers {
                    function: true,
                    ..Modifiers::none()
                },
                false,
            ),
        ] {
            let mut palette = CommandPalette::default();
            text(&mut palette, "é", modifiers, platform);
            assert_eq!(palette.query(), if allowed { "é" } else { "" });
        }
        let mut palette = CommandPalette::default();
        palette.open(false);
        key(&mut palette, "down");
        key(&mut palette, "shift-up");
        key(&mut palette, "ctrl-down");
        assert_eq!(palette.index(), 1);
        assert_eq!(key(&mut palette, "shift-enter"), None);
    }

    #[test]
    fn dispatch_can_reset_selection_before_scroll_and_disabled_action_stays_open() {
        let mut palette = CommandPalette::default();
        palette.open(false);
        text(
            &mut palette,
            "Undo",
            Modifiers::none(),
            ShortcutPlatform::current(),
        );
        assert_eq!(key(&mut palette, "enter"), Some(Op::Undo));
        // A rejected invocation leaves the palette/query untouched.
        palette.scroll_to_selection();
        assert!(palette.is_open());
        assert_eq!(palette.query(), "Undo");
        // Commands/ToolPicker close and reopen via invoke/run before scrolling.
        palette.close();
        palette.open(true);
        palette.scroll_to_selection();
        assert_eq!(palette.index(), 0);
        assert_eq!(palette.query(), "");
    }
}
