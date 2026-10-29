//! Field binding precedence and input-context regressions.
use super::TextField;
use super::{ClipboardOp, Command, TextFieldAction, editing, key_bindings};
use gpui::{Context, Entity, IntoElement, Render, TestAppContext, Window};
use gpui::{KeyBinding, KeyContext, Keymap, Keystroke};

struct FocusProbe(Entity<TextField>);

impl Render for FocusProbe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Runs while the first dispatch tree is still being constructed.
        assert!(self.0.read(cx).focused(window));
        assert!(TextField::any_focused(window, cx));
        self.0.clone()
    }
}

#[gpui::test]
fn first_frame_focus_uses_live_fields_and_releases_registration(cx: &mut TestAppContext) {
    let handle = cx.add_window(|window, cx| {
        assert!(!TextField::any_focused(window, cx));
        let field = TextField::new("First frame", cx);
        field.update(cx, |field, cx| field.focus(window, cx));
        assert!(TextField::any_focused(window, cx));
        FocusProbe(field)
    });
    handle
        .update(cx, |view, window, cx| {
            assert!(TextField::any_focused(window, cx));
            let field_focus = view.0.read(cx).focus.clone();
            window.blur(cx);
            assert!(!TextField::any_focused(window, cx));
            window.focus(&cx.focus_handle(), cx);
            assert!(!TextField::any_focused(window, cx));
            window.focus(&field_focus, cx);
            assert!(TextField::any_focused(window, cx));
        })
        .unwrap();
    // A different window's field must never make an unfocused window match.
    let other = cx.add_empty_window();
    other.update(|window, cx| {
        assert!(!TextField::any_focused(window, cx));
        let field = TextField::new("Released", cx);
        field.update(cx, |field, cx| field.focus(window, cx));
        assert!(TextField::any_focused(window, cx));
        // Retaining a stale focus handle must not retain field registration.
        let focus = field.read(cx).focus.clone();
        drop(field);
        window.defer(cx, move |window, cx| {
            window.focus(&focus, cx);
            assert!(!TextField::any_focused(window, cx));
        });
    });
    cx.run_until_parked();
}

#[test]
fn installed_bindings_parse_and_only_resolve_in_a_field() {
    for mac in [true, false] {
        let bindings = key_bindings(mac);
        let keymap = Keymap::new(bindings.clone());
        for (binding, expected) in bindings.iter().zip(editing::bindings(mac)) {
            for (context, count) in [("TextField SharedTextInput", 1), ("Editor", 0)] {
                let (matched, pending) = keymap.bindings_for_input(
                    binding.keystrokes(),
                    &[KeyContext::parse(context).unwrap()],
                );
                assert!(!pending, "{}", expected.key);
                assert_eq!(matched.len(), count, "{} in {context}", expected.key);
                if let Some(matched) = matched.first() {
                    assert_eq!(
                        matched.action().as_any().downcast_ref::<TextFieldAction>(),
                        Some(&TextFieldAction {
                            command: expected.command
                        }),
                    );
                }
            }
        }
    }
}

#[test]
fn field_undo_bindings_take_precedence_over_document_bindings() {
    for mac in [true, false] {
        let primary = if mac { "cmd-z" } else { "ctrl-z" };
        // A distinct action represents a document's global Undo binding.
        let mut bindings = vec![KeyBinding::new(primary, ClipboardOp::Cut, None)];
        bindings.extend(key_bindings(mac));
        let keymap = Keymap::new(bindings);
        let (matched, _) = keymap.bindings_for_input(
            &[Keystroke::parse(primary).unwrap()],
            &[
                KeyContext::parse("Editor").unwrap(),
                KeyContext::parse("TextField").unwrap(),
            ],
        );
        assert_eq!(
            matched[0]
                .action()
                .as_any()
                .downcast_ref::<TextFieldAction>(),
            Some(&TextFieldAction {
                command: Command::Undo
            }),
        );
    }
}
