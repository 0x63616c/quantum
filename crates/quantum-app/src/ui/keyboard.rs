//! Focus-aware keyboard routing for the editor shell.
use super::*;
use quantum_ui::Space;

use crate::shortcuts::EditorKey;

impl EditorView {
    pub(super) fn key_down(
        &mut self,
        e: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.guarded_key(e, window, cx) {
            return;
        }
        let key = EditorKey::navigation(&e.keystroke);
        if self.field_key(key, e, window, cx) {
            return;
        }
        if key == EditorKey::Escape {
            self.escape(window, cx);
            return;
        }
        if self.palettes.open || self.agent.open {
            cx.stop_propagation();
            return;
        }
        if self.palette_open {
            self.palette_chooser_key(key, cx);
            return;
        }
        if self.commands.is_open() {
            self.command_palette_key(&e.keystroke, window, cx);
            return;
        }
        if let Some(menu) = self.menu {
            self.menu_key(menu, key, window, cx);
            return;
        }
        if self.info.is_some() || self.save_flow.needs_confirmation() || self.palette_open {
            cx.stop_propagation();
            return;
        }
        self.viewport_key(key, e.is_held, cx);
    }

    fn guarded_key(
        &mut self,
        e: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        #[cfg(target_os = "linux")]
        if self.capture_status(e, cx) {
            return true;
        }
        if self.save_flow.picker.waiting() {
            if EditorKey::navigation(&e.keystroke) == EditorKey::Escape {
                self.cancel_save_picker(cx);
                window.focus(&self.focus, cx);
            }
            cx.stop_propagation();
            return true;
        }
        self.reconcile_palette_panel(window, cx);
        if self.save_flow.needs_confirmation() {
            self.pending_key(&e.keystroke, cx);
            return true;
        }
        // The palette dialog owns cancellation, text commits and grid navigation.
        // Its descendants must receive these keys before viewport shortcuts.
        if self.palettes.open && !self.save_flow.dialog_open() {
            return true;
        }
        if self.palette_focus.is_focused(window) {
            let delta = match EditorKey::from_key(&e.keystroke.key) {
                EditorKey::Left => Some(-1),
                EditorKey::Right => Some(1),
                EditorKey::Up => Some(-8),
                EditorKey::Down => Some(8),
                _ => None,
            };
            if let Some(delta) = delta {
                if let Some(next) = self
                    .palette_request(PaletteAction::StepSwatch(delta), cx)
                    .and_then(|out| out.selected)
                {
                    self.swatch_scroll.set_offset(point(
                        px(Space::S0.px()),
                        -px((next / 8) as f32 * metrics::SWATCH_PITCH),
                    ));
                    cx.notify();
                }
                cx.stop_propagation();
                return true;
            }
            if !matches!(
                EditorKey::from_key(&e.keystroke.key),
                EditorKey::Escape | EditorKey::Tab
            ) {
                // Choosing a brush color must never apply the viewport tool.
                return true;
            }
        }
        if self.save_flow.dialog_open() {
            cx.stop_propagation();
            return true;
        }
        false
    }

    fn field_key(
        &self,
        key: EditorKey,
        e: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        // Shared editors own their keys and native text input, including fields
        // created inside controlled NumericField elements.
        if crate::text_field::focused(window, cx) {
            return true;
        }

        if key == EditorKey::Tab && !self.save_flow.needs_confirmation() && self.menu.is_none() {
            if self.palettes.open
                || self.agent.open
                || self.info.is_some()
                || self.commands.is_open()
            {
                Dialog::trap_tab(&self.modal_focus, e.keystroke.modifiers.shift, window, cx);
            } else if e.keystroke.modifiers.shift {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            cx.stop_propagation();
            return true;
        }
        if key != EditorKey::Escape
            && (self.palettes.field_focused(window, cx)
                || self.discussion.text.read(cx).focused(window)
                || self.timeline.name.read(cx).focused(window))
        {
            return true;
        }
        if !self.focus.is_focused(window)
            && !self.commands.is_open()
            && matches!(key, EditorKey::Enter | EditorKey::Space)
        {
            return true;
        }
        false
    }

    fn escape(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.discussion_action(quantum_plugin_api::CommentsAction::Cancel, cx);
        if self.editor.preview.is_some() {
            self.editor.return_live();
            self.refresh_preview(cx);
        }
        self.agent.open = false;

        self.palettes.open = false;
        window.focus(&self.focus, cx);
        self.menu = None;
        self.commands.close();
        self.palette_open = false;
        self.info = None;
        self.save_flow.cancel_confirmation();
        self.editor.box_start = None;
        self.drag = None;
        self.overlay_drag = None;
        self.overlay_hover = None;
        self.hover = None;
        self.status = HintText::default();
        self.error = false;
        cx.stop_propagation();
        cx.notify();
    }

    fn palette_chooser_key(&mut self, key: EditorKey, cx: &mut Context<Self>) {
        let choices = self.palette_choices();
        if choices.is_empty() {
            self.palette_open = false;
            return;
        }
        match key {
            EditorKey::Down => self.palette_cursor = (self.palette_cursor + 1) % choices.len(),
            EditorKey::Up => {
                self.palette_cursor = (self.palette_cursor + choices.len() - 1) % choices.len()
            }
            EditorKey::Enter => {
                self.palette_request(PaletteAction::ChoosePreset(self.palette_cursor), cx);
                self.palette_open = false;
            }
            _ => {}
        }
        self.palette_scroll.scroll_to_item(self.palette_cursor);
        cx.stop_propagation();
        cx.notify();
    }

    fn menu_key(
        &mut self,
        menu: usize,
        key: EditorKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let items = crate::actions::menu_items(menu, &self.editor.plugin_host());
        self.menu_index = self.menu_index.min(items.len().saturating_sub(1));
        match key {
            EditorKey::Down if !items.is_empty() => {
                self.menu_index = (self.menu_index + 1) % items.len()
            }
            EditorKey::Up if !items.is_empty() => {
                self.menu_index = (self.menu_index + items.len() - 1) % items.len()
            }
            EditorKey::Right => {
                self.menu = Some((menu + 1) % MENUS.len());
                self.menu_index = 0;
            }
            EditorKey::Left => {
                self.menu = Some((menu + MENUS.len() - 1) % MENUS.len());
                self.menu_index = 0;
            }
            EditorKey::Enter if !items.is_empty() => {
                self.invoke(items[self.menu_index], window, cx)
            }
            _ => {}
        }
        cx.stop_propagation();
        cx.notify();
    }

    fn viewport_key(&mut self, key: EditorKey, is_held: bool, cx: &mut Context<Self>) {
        if self.panel == SidebarTab::History && key == EditorKey::Space {
            if !is_held {
                self.toggle_history(cx);
            }
            cx.stop_propagation();
            return;
        }
        if self.panel == SidebarTab::History && matches!(key, EditorKey::Left | EditorKey::Right) {
            let count = self.editor.timeline_entries().count();
            let index = self.editor.timeline_index();
            self.scrub(
                if key == EditorKey::Left {
                    index.saturating_sub(1)
                } else {
                    (index + 1).min(count.saturating_sub(1))
                },
                cx,
            );
            cx.stop_propagation();
            return;
        }
        let movement = match key {
            EditorKey::Left => Some((0, -1)),
            EditorKey::Right => Some((0, 1)),
            EditorKey::Up => Some((2, -1)),
            EditorKey::Down => Some((2, 1)),
            EditorKey::PageUp => Some((1, 1)),
            EditorKey::PageDown => Some((1, -1)),
            _ => None,
        };
        if let Some((a, delta)) = movement {
            self.cursor[a] = (self.cursor[a] + delta).clamp(
                self.editor.display_snapshot().bounds.min[a],
                self.editor.display_snapshot().bounds.max[a] - 1,
            );
            self.hover = Some(self.cursor);
            cx.stop_propagation();
        }
        if key == EditorKey::Enter && self.editor.comments_state.placing {
            self.anchor_comment(self.cursor, cx);
            return;
        }
        if key == EditorKey::Enter {
            let r = self.editor.apply_at(self.cursor);
            self.report(r, "Tool applied", cx);
            cx.stop_propagation();
        }
        if key == EditorKey::PreviousSwatch || key == EditorKey::NextSwatch {
            self.palette_request(
                PaletteAction::StepSwatch(if key == EditorKey::NextSwatch { 1 } else { -1 }),
                cx,
            );
            cx.stop_propagation();
        }
        cx.notify();
    }
}
