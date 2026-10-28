//! Command palette state and input policy. Search stays in the command registry.
use super::EditorView;
use crate::{
    actions::{self, Op},
    shortcuts::navigation_key,
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
    pub(super) fn set_query(&mut self, text: &str) {
        if self.query != text {
            self.query = text.to_owned();
            self.index = 0;
        }
    }
    pub(super) fn index(&self) -> usize {
        self.index
    }
    pub(super) fn scroll(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub(super) fn filtered_actions(&self, host: &quantum_plugin_host::Host) -> Vec<Op> {
        actions::search(&self.query, self.tools_only)
            .into_iter()
            .filter(|op| host.registry().get(op.command_id()).is_some())
            .collect()
    }

    // Escape and focus routing belong to the surrounding modal shell. Return
    // Enter's command without closing: invoke still owns availability checks.
    fn key(&mut self, stroke: &Keystroke, host: &quantum_plugin_host::Host) -> Option<Op> {
        let key = navigation_key(stroke).unwrap_or_default();
        let count = self.filtered_actions(host).len();
        self.index = self.index.min(count.saturating_sub(1));
        match key {
            "down" => self.index = (self.index + 1).min(count.saturating_sub(1)),
            "up" => self.index = self.index.saturating_sub(1),
            "enter" => return self.filtered_actions(host).get(self.index).copied(),
            _ => {}
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
        let op = self.commands.key(stroke, &self.editor.plugin_host());
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
    use super::*;
    fn key(p: &mut CommandPalette, k: &str) -> Option<Op> {
        p.key(&Keystroke::parse(k).unwrap(), &quantum_app::app_host())
    }
    #[test]
    fn opening_resets_query_and_selection() {
        let mut p = CommandPalette::default();
        p.open(false);
        p.set_query("view");
        key(&mut p, "down");
        p.close();
        assert_eq!(p.query, "view");
        p.open(true);
        assert!(p.is_open());
        assert!(p.tools_only());
        assert_eq!(p.query, "");
        assert_eq!(p.index(), 0);
    }
    #[test]
    fn query_changes_reset_results_and_empty_results_never_dispatch() {
        let mut p = CommandPalette::default();
        p.open(false);
        key(&mut p, "down");
        p.set_query("zzzzzz");
        assert_eq!(p.index(), 0);
        assert_eq!(key(&mut p, "enter"), None);
        // Text arrives only from TextField's native input handler.
        key(&mut p, "backspace");
        assert_eq!(p.query, "zzzzzz");
        p.set_query("Undo");
        assert_eq!(key(&mut p, "enter"), Some(Op::Undo));
    }
    #[test]
    fn navigation_clamps_and_disabled_commands_disappear() {
        let mut p = CommandPalette::default();
        p.open(false);
        let host = quantum_app::app_host();
        let matches = p.filtered_actions(&host);
        key(&mut p, "up");
        assert_eq!(p.index(), 0);
        for _ in 0..matches.len() + 3 {
            key(&mut p, "down");
        }
        assert_eq!(p.index(), matches.len() - 1);
        assert_eq!(key(&mut p, "enter"), matches.last().copied());
        let mut host = host;
        p.set_query("palette");
        assert_eq!(p.filtered_actions(&host).len(), 3);
        host.disable("dev.quantum.palette").unwrap();
        assert!(p.filtered_actions(&host).is_empty());
        assert_eq!(p.key(&Keystroke::parse("enter").unwrap(), &host), None);
    }
}
