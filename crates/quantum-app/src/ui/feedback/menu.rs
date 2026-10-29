//! Keep native commands usable without leaking document shortcuts into a report.
use super::*;

fn available(op: Op, window: &Window, cx: &App) -> bool {
    matches!(
        op,
        Op::Feedback
            | Op::Welcome
            | Op::Quit
            | Op::Hide
            | Op::HideOthers
            | Op::ShowAll
            | Op::Minimize
            | Op::ZoomWindow
            | Op::FullScreen
    ) || (matches!(op, Op::Undo | Op::Redo) && crate::text_field::focused(window, cx))
}

impl FeedbackView {
    pub(super) fn sync_menu(&self, window: &Window, cx: &mut Context<Self>) {
        if crate::actions::sync_native_menus(window) {
            cx.set_menus(crate::actions::menus_for_window(
                window.is_fullscreen(),
                |_| false,
                |op| available(op, window, cx),
            ));
        }
    }

    pub(super) fn menu_action(&mut self, op: Op, window: &mut Window, cx: &mut Context<Self>) {
        if crate::text_field::route_history(op, window, cx) || !available(op, window, cx) {
            return;
        }
        match op {
            // The launcher forwards Quit to the editor's unsaved-change policy.
            Op::Quit => welcome::request(Op::Quit, None, cx),
            Op::Welcome => welcome::show_deferred(false, cx),
            Op::Feedback => window.activate_window(),
            Op::Hide => cx.hide(),
            Op::HideOthers => cx.hide_other_apps(),
            Op::ShowAll => cx.unhide_other_apps(),
            Op::Minimize => window.minimize_window(),
            Op::ZoomWindow => window.zoom_window(),
            Op::FullScreen => window.toggle_fullscreen(),
            _ => {}
        }
    }
}
