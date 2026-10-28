//! Native menu routing for the independent Settings window.
use super::*;

impl SettingsView {
    pub(super) fn action(&mut self, op: &Op, window: &mut Window, cx: &mut Context<Self>) {
        if crate::text_field::route_history(*op, window, cx) {
            return;
        }
        if op
            .availability(actions::CommandContext::Settings {
                updates_available: crate::ui::updates_available(),
            })
            .is_err()
        {
            return;
        }
        match op {
            Op::Settings => window.activate_window(),
            Op::About => self.editor_action(Op::About, cx),
            Op::Welcome => {
                cx.defer(|cx| {
                    let _ = crate::ui::welcome::show(cx);
                });
            }
            Op::Feedback => crate::ui::feedback::show_deferred(cx),
            Op::Help => self.navigate(Section::Shortcuts, cx),
            Op::CheckUpdates => {
                self.navigate(Section::Updates, cx);
                crate::ui::check_for_updates();
            }
            Op::Hide => cx.hide(),
            Op::HideOthers => cx.hide_other_apps(),
            Op::ShowAll => cx.unhide_other_apps(),
            Op::Minimize => window.minimize_window(),
            Op::ZoomWindow => window.zoom_window(),
            Op::FullScreen => {
                window.toggle_fullscreen();
            }
            Op::Quit => self.editor_action(Op::Quit, cx),
            _ => {}
        }
        cx.notify();
    }
}
