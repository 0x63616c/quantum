//! Application import for the shared, domain-independent editor.
pub use quantum_ui::TextField;

/// Native Edit menus dispatch shell history actions rather than field bindings.
/// Route these through the focused component before consulting document state.
pub fn route_history(
    op: crate::actions::Op,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> bool {
    use quantum_ui::editing::Command;
    let command = match op {
        crate::actions::Op::Undo => Command::Undo,
        crate::actions::Op::Redo => Command::Redo,
        _ => return false,
    };
    if !focused(window, cx) {
        return false;
    }
    window.dispatch_action(Box::new(quantum_ui::TextFieldAction { command }), cx);
    cx.stop_propagation();
    true
}

/// Includes persistent fields created by controlled numeric components.
/// Safe during open_window's synchronous first draw, before any dispatch tree.
pub fn focused(window: &gpui::Window, cx: &gpui::App) -> bool {
    TextField::any_focused(window, cx)
}
