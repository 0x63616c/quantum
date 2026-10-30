//! Application import for the shared, domain-independent editor.
pub use voxelite_ui::TextField;

/// Native Edit menus dispatch shell history actions rather than field bindings.
/// Route these through the focused component before consulting document state.
pub fn route_history(
    op: crate::actions::Op,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> bool {
    use voxelite_ui::editing::Command;
    let command = match op {
        crate::actions::Op::Undo => Command::Undo,
        crate::actions::Op::Redo => Command::Redo,
        _ => return false,
    };
    if !focused(window) {
        return false;
    }
    window.dispatch_action(Box::new(voxelite_ui::TextFieldAction { command }), cx);
    cx.stop_propagation();
    true
}

/// Includes persistent fields created by controlled numeric components.
pub fn focused(window: &gpui::Window) -> bool {
    window
        .context_stack()
        .iter()
        .any(|c| c.contains("SharedTextInput"))
}
