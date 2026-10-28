//! Field binding precedence and input-context regressions.
use super::{ClipboardOp, Command, TextFieldAction, editing, key_bindings};
use gpui::{KeyBinding, KeyContext, Keymap, Keystroke};

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
