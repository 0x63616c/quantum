//! Exercise the production open_window -> synchronous draw -> native menus path.
use super::{Editor, EditorView, Launcher, Op, Sample, settings, startup};
use gpui::{OwnedMenuItem, TestAppContext, WindowHandle};

fn opened(cx: &mut TestAppContext) -> WindowHandle<EditorView> {
    cx.run_until_parked();
    cx.read(|cx| {
        let launcher = cx.global::<Launcher>();
        assert!(
            launcher.welcome.is_none(),
            "Welcome must close after success"
        );
        let handle = launcher.editor.expect("editor must open");
        let view = handle.read(cx).unwrap();
        assert!(!view.error, "editor must open without an error");
        handle
    })
}

fn history_enabled(cx: &TestAppContext, enabled: [bool; 2]) {
    cx.read(|cx| {
        let menus = cx.get_menus().unwrap();
        for (op, enabled) in [(Op::Undo, enabled[0]), (Op::Redo, enabled[1])] {
            assert!(
                menus
                    .iter()
                    .flat_map(|menu| &menu.items)
                    .any(|item| matches!(item,
                        OwnedMenuItem::Action { action, disabled, .. }
                            if action.as_any().downcast_ref::<Op>() == Some(&op)
                                && *disabled != enabled
                    )),
                "{op:?} enabled={enabled}"
            );
        }
    });
}

#[gpui::test]
fn welcome_sample_opens_editor(cx: &mut TestAppContext) {
    let (welcome, _dir) = super::tests::window_fixture(cx, 0);
    welcome
        .update(cx, |view, _, cx| view.open_sample(Sample::Autumn, cx))
        .unwrap();
    let editor = opened(cx);
    cx.read(|cx| {
        assert!(welcome.read(cx).is_err());
        let view = editor.read(cx).unwrap();
        assert_eq!(view.editor.snapshot().name, Sample::Autumn.name());
        assert!(view.editor.path.is_none());
    });
}

#[gpui::test]
fn welcome_new_finishes_into_editor(cx: &mut TestAppContext) {
    let (welcome, _dir) = super::tests::window_fixture(cx, 0);
    welcome
        .update(cx, |view, window, cx| view.run(Op::New, None, window, cx))
        .unwrap();
    let editor = opened(cx);
    cx.read(|cx| {
        assert!(welcome.read(cx).is_err());
        assert!(editor.read(cx).unwrap().editor.path.is_none());
    });
    history_enabled(cx, [false, false]);
}

#[gpui::test]
fn welcome_loaded_project_finishes_into_editor(cx: &mut TestAppContext) {
    let (welcome, _dir) = super::tests::window_fixture(cx, 0);
    // A parsed project enters the same finish continuation as the file picker.
    // Keep it pathless so this test never writes the user's recent-file history.
    let session = quantum_format::session::DocumentSession::new(
        quantum_format::from_bytes(Sample::Snow.bytes()).unwrap(),
        None,
        quantum_core::Limits::default(),
    )
    .unwrap();
    welcome
        .update(cx, |view, _, cx| {
            view.finish(Editor::from_session(session), cx)
        })
        .unwrap();
    let editor = opened(cx);
    cx.read(|cx| {
        assert!(welcome.read(cx).is_err());
        assert_eq!(
            editor.read(cx).unwrap().editor.snapshot().name,
            Sample::Snow.name()
        );
    });
}

#[gpui::test]
fn startup_without_welcome_opens_editor(cx: &mut TestAppContext) {
    let (welcome, _dir) = super::tests::window_fixture(cx, 0);
    welcome
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    cx.update(|cx| {
        cx.global_mut::<Launcher>().welcome = None;
        cx.global_mut::<settings::SettingsState>()
            .values
            .show_welcome = false;
        startup(cx).unwrap();
    });
    opened(cx);
    history_enabled(cx, [false, false]);
}

#[gpui::test]
fn settings_opens_with_first_frame_menus(cx: &mut TestAppContext) {
    let (welcome, _dir) = super::tests::window_fixture(cx, 0);
    welcome
        .update(cx, |view, _, cx| view.finish(Editor::blank(), cx))
        .unwrap();
    let editor = opened(cx);
    editor
        .update(cx, |view, window, cx| {
            view.open_settings(settings::Section::General, window, cx);
        })
        .unwrap();
    cx.run_until_parked();
    cx.read(|cx| {
        let settings = editor
            .read(cx)
            .unwrap()
            .settings_window
            .expect("Settings must open");
        assert!(settings.read(cx).is_ok());
    });
    history_enabled(cx, [false, false]);
}

#[gpui::test]
fn editor_menu_history_follows_field_focus(cx: &mut TestAppContext) {
    let (welcome, _dir) = super::tests::window_fixture(cx, 0);
    welcome
        .update(cx, |view, _, cx| view.finish(Editor::blank(), cx))
        .unwrap();
    let editor = opened(cx);
    history_enabled(cx, [false, false]);
    editor
        .update(cx, |view, window, cx| {
            assert!(view.editor.apply_at([1, 1, 1]).unwrap());
            view.invoke(Op::Commands, window, cx);
        })
        .unwrap();
    cx.run_until_parked();
    history_enabled(cx, [true, true]);
    let revision = cx.read(|cx| editor.read(cx).unwrap().editor.engine.revision());
    cx.simulate_input(editor.into(), "sample");
    // Native menus dispatch shell Op actions, not field key bindings.
    for (op, expected) in [
        (Op::Undo, ""),
        (Op::Undo, ""),
        (Op::Redo, "sample"),
        (Op::Redo, "sample"),
    ] {
        editor
            .update(cx, |_, window, cx| window.dispatch_action(Box::new(op), cx))
            .unwrap();
        cx.run_until_parked();
        cx.read(|cx| {
            let view = editor.read(cx).unwrap();
            assert_eq!(view.command_input.read(cx).text(), expected);
            assert_eq!(view.editor.engine.revision(), revision);
        });
    }
    editor
        .update(cx, |view, window, cx| {
            window.focus(&view.focus, cx);
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();
    history_enabled(cx, [true, false]);
}
