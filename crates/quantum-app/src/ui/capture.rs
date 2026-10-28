//! Linux screenshot fixtures, inert unless explicitly enabled by the capture script.
use super::*;

thread_local! {
    static POSITION: Cell<Option<Option<Pos>>> = const { Cell::new(None) };
}

pub(super) fn status_fixture() -> Option<Option<Pos>> {
    POSITION.with(Cell::get)
}

impl EditorView {
    pub(super) fn capture_status(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        if std::env::var_os("QUANTUM_CAPTURE_STATUS").is_none()
            || event.keystroke.key != "f12"
            || event.keystroke.modifiers
                != (Modifiers {
                    control: true,
                    shift: true,
                    ..Modifiers::none()
                })
        {
            return false;
        }
        thread_local! { static STEP: Cell<usize> = const { Cell::new(0) }; }
        let positions = [
            Some([1, 2, 27]),
            Some([-1, -20, -327]),
            Some([123, 456, 789]),
            Some([i32::MIN, i32::MAX, 1_000_000_000]),
            None,
        ];
        STEP.with(|step| {
            POSITION.with(|position| position.set(Some(positions[step.get() % positions.len()])));
            step.set(step.get() + 1);
        });
        cx.stop_propagation();
        cx.notify();
        true
    }
}
