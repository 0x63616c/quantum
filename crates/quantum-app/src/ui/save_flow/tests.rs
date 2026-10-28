use super::{CloseRequest, Op, PendingAction, SaveChoice, SaveFlow, ShortcutPlatform};
use gpui::Keystroke;
use quantum_app::{editor::Editor, samples::Sample, save_format::SaveFormat};

#[test]
fn untouched_samples_require_confirmation_until_a_project_save_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    for sample in Sample::ALL {
        let mut editor = Editor::from_session(sample.session().unwrap()).unwrap();
        assert!(!editor.engine.is_dirty());
        for op in [Op::New, Op::Open, Op::Quit] {
            let mut flow = SaveFlow::default();
            assert!(flow.confirm_if_dirty(op, || editor.needs_save()));
            assert_eq!(
                flow.resolve(SaveChoice::Cancel),
                Some(PendingAction::Cancelled)
            );
            assert!(editor.path.is_none());
            assert!(flow.confirm_if_dirty(op, || editor.needs_save()));
            assert_eq!(
                flow.resolve(SaveChoice::DontSave),
                Some(PendingAction::Run(op))
            );
        }
        let mut flow = SaveFlow::default();
        assert!(matches!(
            flow.request_close(|| editor.needs_save()),
            CloseRequest::Confirm
        ));
        flow.cancel_confirmation();
        editor
            .save(&dir.path().join(format!("{}.voxl", sample.name())))
            .unwrap();
        assert!(matches!(
            flow.request_close(|| editor.needs_save()),
            CloseRequest::Allow
        ));
    }
}

#[test]
fn sample_picker_cancellation_and_failures_preserve_the_unsaved_copy_and_original() {
    let sample = Sample::Lighthouse;
    let original = sample.session().unwrap().engine.document().content.clone();
    let dir = tempfile::tempdir().unwrap();
    for op in [Op::Save, Op::SaveAs, Op::New, Op::Open, Op::Quit] {
        for outcome in 0..5 {
            let mut editor = Editor::from_session(sample.session().unwrap()).unwrap();
            editor
                .execute(
                    "Rename copy",
                    quantum_core::Command::EditDocument {
                        name: Some("My lighthouse".into()),
                        bounds: None,
                    },
                )
                .unwrap();
            let mut flow = SaveFlow::default();
            if op == Op::Open {
                flow.sample = Some(Sample::Snow);
            }
            if flow.confirm_if_dirty(op, || editor.needs_save()) {
                assert_eq!(flow.resolve(SaveChoice::Save), Some(PendingAction::Save));
            }
            let request = flow.picker.begin();
            flow.begin_dialog();
            flow.before_picker = Some(("Previous status".into(), false));
            let mut status = "Waiting for file dialog…".into();
            let mut error = false;
            match outcome {
                0 => assert!(flow.cancel_picker(&mut status, &mut error)),
                1..=3 => {
                    let result = match outcome {
                        1 => Ok(None),
                        2 => Ok(Some((std::path::PathBuf::new(), SaveFormat::Project))),
                        _ => Err(anyhow::anyhow!("picker failed")),
                    };
                    assert!(
                        flow.complete_picker(request, result, &mut status, &mut error)
                            .is_none()
                    );
                }
                _ => {
                    // A valid picker response can still fail to write. No receipt
                    // means the current session must remain pathless and unsaved.
                    let (path, _) = flow
                        .complete_picker(
                            request,
                            Ok(Some((dir.path().to_path_buf(), SaveFormat::Project))),
                            &mut status,
                            &mut error,
                        )
                        .unwrap();
                    assert!(editor.prepare_save(&path).write().is_err());
                    flow.finish_dialog();
                    assert_eq!(flow.complete_save(false), None);
                }
            }
            assert!(!flow.dialog_open());
            assert!(!flow.picker.waiting());
            assert!(flow.before_picker.is_none());
            assert!(flow.sample.is_none());
            assert!(flow.open_path.is_none());
            assert_eq!(flow.complete_save(true), None);
            assert!(editor.path.is_none());
            assert!(editor.needs_save());
            assert_eq!(editor.snapshot().name, "My lighthouse");
            assert_eq!(
                sample.session().unwrap().engine.document().content,
                original
            );
            // Late native replies cannot turn Cancel into a save, including
            // after the user has started a fresh picker.
            let retry = flow.picker.begin();
            let late = dir.path().join("late.voxl");
            assert!(
                flow.complete_picker(
                    request,
                    Ok(Some((late.clone(), SaveFormat::Project))),
                    &mut status,
                    &mut error
                )
                .is_none()
            );
            assert!(flow.picker.waiting());
            assert!(!late.exists());
            assert!(flow.picker.complete(retry));
        }
    }
}

#[test]
fn sample_save_as_adopts_only_the_successfully_written_path() {
    let dir = tempfile::tempdir().unwrap();
    for sample in Sample::ALL {
        let mut editor = Editor::from_session(sample.session().unwrap()).unwrap();
        let original = editor.snapshot().clone();
        let mut flow = SaveFlow::default();
        let request = flow.picker.begin();
        flow.begin_dialog();
        let destination = dir.path().join(format!("{}.voxl", sample.name()));
        let (path, format) = flow
            .complete_picker(
                request,
                Ok(Some((destination.clone(), SaveFormat::Project))),
                &mut "Waiting…".into(),
                &mut false,
            )
            .unwrap();
        assert!(matches!(format, SaveFormat::Project));
        assert!(flow.dialog_open());
        assert!(editor.path.is_none());
        assert!(editor.needs_save());
        editor.finish_save(editor.prepare_save(&path).write().unwrap());
        flow.finish_dialog();
        assert_eq!(flow.complete_save(true), None);
        assert_eq!(editor.path, Some(destination.canonicalize().unwrap()));
        assert!(!editor.needs_save());
        assert!(flow.sample.is_none());
        assert_eq!(Editor::open(&destination).unwrap().snapshot(), &original);
        assert_eq!(
            sample.session().unwrap().engine.document().content,
            original
        );
    }
}

fn key(flow: &mut SaveFlow, stroke: &str) -> Option<SaveChoice> {
    flow.key(
        &Keystroke::parse(stroke).unwrap(),
        ShortcutPlatform::current(),
    )
}

#[test]
fn only_dirty_destructive_commands_require_confirmation_and_default_to_save() {
    let mut flow = SaveFlow::default();
    for op in [Op::New, Op::Open, Op::Quit] {
        assert!(!flow.confirm_if_dirty(op, || false));
        assert!(!flow.needs_confirmation());
        assert!(flow.confirm_if_dirty(op, || true));
        assert_eq!(flow.choice_index(), 2);
        key(&mut flow, "left");
        assert_eq!(flow.choice_index(), 1);
        assert!(flow.confirm_if_dirty(op, || true));
        assert_eq!(flow.choice_index(), 2);
        flow.cancel_confirmation();
    }
    for op in [Op::Save, Op::SaveAs, Op::Undo, Op::Commands] {
        assert!(!flow.confirm_if_dirty(op, || true));
        assert!(!flow.needs_confirmation());
    }
}

#[test]
fn recent_path_survives_save_but_is_discarded_on_cancel_and_failure() {
    let mut flow = SaveFlow::default();
    let path = std::path::PathBuf::from("project.voxl");
    flow.open_path = Some(path.clone());
    flow.confirm_if_dirty(Op::Open, || true);
    assert_eq!(flow.resolve(SaveChoice::Save), Some(PendingAction::Save));
    assert_eq!(flow.complete_save(true), Some(Op::Open));
    assert_eq!(flow.open_path.take(), Some(path.clone()));
    for cancel in 0..4 {
        flow.open_path = Some(path.clone());
        flow.confirm_if_dirty(Op::Open, || true);
        match cancel {
            0 => flow.cancel_confirmation(),
            1 => {
                flow.resolve(SaveChoice::Cancel);
            }
            2 => {
                flow.resolve(SaveChoice::Save);
                flow.complete_save(false);
            }
            _ => {
                flow.resolve(SaveChoice::Save);
                flow.cancel_save();
            }
        }
        assert!(flow.open_path.is_none());
    }
}

#[test]
fn sample_open_is_retained_after_save_and_cleared_on_every_cancel_path() {
    use quantum_app::samples::Sample;
    let mut flow = SaveFlow {
        sample: Some(Sample::Lighthouse),
        ..Default::default()
    };
    flow.confirm_if_dirty(Op::Open, || true);
    flow.resolve(SaveChoice::Save);
    assert_eq!(flow.complete_save(true), Some(Op::Open));
    assert_eq!(flow.sample.take(), Some(Sample::Lighthouse));
    for cancel in 0..4 {
        flow.sample = Some(Sample::Snow);
        flow.confirm_if_dirty(Op::Open, || true);
        match cancel {
            0 => flow.cancel_confirmation(),
            1 => {
                flow.resolve(SaveChoice::Cancel);
            }
            2 => {
                flow.resolve(SaveChoice::Save);
                flow.complete_save(false);
            }
            _ => {
                flow.resolve(SaveChoice::Save);
                flow.cancel_save();
            }
        }
        assert!(flow.sample.is_none());
    }
}

#[test]
fn dont_save_runs_once_cancel_does_nothing_and_save_defers_each_operation() {
    for op in [Op::New, Op::Open, Op::Quit] {
        for choice in [SaveChoice::DontSave, SaveChoice::Cancel, SaveChoice::Save] {
            let mut flow = SaveFlow::default();
            flow.confirm_if_dirty(op, || true);
            let expected = match choice {
                SaveChoice::DontSave => PendingAction::Run(op),
                SaveChoice::Cancel => PendingAction::Cancelled,
                SaveChoice::Save => PendingAction::Save,
            };
            assert_eq!(flow.resolve(choice), Some(expected));
            assert!(!flow.needs_confirmation());
            assert_eq!(flow.resolve(choice), None);
            assert_eq!(
                flow.complete_save(true),
                (choice == SaveChoice::Save).then_some(op)
            );
            assert_eq!(flow.complete_save(true), None);
        }
    }
}

#[test]
fn failed_save_consumes_continuation_and_keeps_later_save_from_running_it() {
    let mut flow = SaveFlow::default();
    flow.confirm_if_dirty(Op::Quit, || true);
    assert_eq!(flow.resolve(SaveChoice::Save), Some(PendingAction::Save));
    flow.begin_dialog();
    assert!(flow.dialog_open());
    flow.finish_dialog();
    assert_eq!(flow.complete_save(false), None);
    assert!(!flow.dialog_open());
    assert!(!flow.needs_confirmation());
    flow.begin_dialog();
    flow.finish_dialog();
    assert_eq!(flow.complete_save(true), None);
}

#[test]
fn cancelling_or_failing_path_chooser_aborts_the_entire_deferred_operation() {
    for op in [Op::New, Op::Open, Op::Quit] {
        let mut flow = SaveFlow::default();
        flow.confirm_if_dirty(op, || true);
        flow.resolve(SaveChoice::Save);
        flow.begin_dialog();
        let request = flow.picker.begin();
        let mut status = "Waiting for file dialog…".into();
        let mut error = false;
        flow.before_picker = Some(("Previous status".into(), true));
        assert!(flow.cancel_picker(&mut status, &mut error));
        assert_eq!(status.plain_text(), Some("Previous status"));
        assert!(error);
        assert!(flow.before_picker.is_none());
        assert!(!flow.cancel_picker(&mut status, &mut error));
        assert!(!flow.picker.complete(request));
        assert!(!flow.dialog_open());
        assert!(!flow.needs_confirmation());
        assert_eq!(flow.complete_save(true), None);
    }
}

#[test]
fn path_selection_to_write_keeps_dialog_gate_and_continuation_until_completion() {
    let mut flow = SaveFlow::default();
    flow.confirm_if_dirty(Op::Open, || true);
    flow.resolve(SaveChoice::Save);
    flow.begin_dialog(); // Path chooser.
    flow.begin_dialog(); // Selected path starts the write without unblocking.
    assert!(flow.dialog_open());
    assert!(matches!(flow.request_close(|| true), CloseRequest::Blocked));
    flow.finish_dialog();
    assert_eq!(flow.complete_save(true), Some(Op::Open));
    // The continued Open operation starts its own dialog.
    flow.begin_dialog();
    assert!(matches!(
        flow.request_close(|| false),
        CloseRequest::Blocked
    ));
    flow.finish_dialog();
    assert_eq!(flow.complete_save(true), None);
}

#[test]
fn close_obeys_dialog_gate_then_dirty_state_and_preserves_existing_pending_state() {
    let mut flow = SaveFlow::default();
    assert!(matches!(flow.request_close(|| false), CloseRequest::Allow));
    assert!(matches!(flow.request_close(|| true), CloseRequest::Confirm));
    assert_eq!(flow.choice_index(), 2);
    flow.begin_dialog();
    assert!(matches!(
        flow.request_close(|| false),
        CloseRequest::Blocked
    ));
    assert!(matches!(flow.request_close(|| true), CloseRequest::Blocked));
    assert_eq!(flow.pending, Some(Op::Quit));
    flow.finish_dialog();
    // As before, a clean close allows closing without rewriting a pending op.
    assert!(matches!(flow.request_close(|| false), CloseRequest::Allow));
    assert_eq!(flow.pending, Some(Op::Quit));
    flow.cancel_confirmation();
    assert_eq!(flow.resolve(SaveChoice::Save), None);
}

#[test]
fn confirmation_navigation_wraps_and_enter_escape_have_the_original_outcomes() {
    let mut flow = SaveFlow::default();
    flow.confirm_if_dirty(Op::New, || true);
    assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::Save));
    key(&mut flow, "right");
    assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::DontSave));
    key(&mut flow, "tab");
    assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::Cancel));
    key(&mut flow, "shift-tab");
    key(&mut flow, "left");
    assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::Save));
    assert_eq!(key(&mut flow, "escape"), Some(SaveChoice::Cancel));
    assert_eq!(key(&mut flow, "shift-right"), None);
    assert_eq!(flow.choice_index(), 2);
    assert_eq!(key(&mut flow, "shift-enter"), None);
}

#[test]
fn dont_save_shortcut_requires_exact_host_primary_modifier() {
    for (platform, primary, wrong) in [
        (ShortcutPlatform::Apple, "cmd-d", "ctrl-d"),
        (ShortcutPlatform::WindowsLinux, "ctrl-d", "cmd-d"),
    ] {
        let mut flow = SaveFlow::default();
        flow.confirm_if_dirty(Op::Quit, || true);
        assert_eq!(
            flow.key(&Keystroke::parse(primary).unwrap(), platform),
            Some(SaveChoice::DontSave)
        );
        for stroke in [wrong, "d", "ctrl-alt-d", "cmd-shift-d"] {
            assert_eq!(flow.key(&Keystroke::parse(stroke).unwrap(), platform), None);
        }
    }
}

#[test]
fn unrelated_dialog_completion_and_confirmation_dismissal_do_not_consume_save() {
    let mut flow = SaveFlow::default();
    flow.confirm_if_dirty(Op::New, || true);
    flow.resolve(SaveChoice::Save);
    flow.begin_dialog();
    flow.finish_dialog();
    flow.cancel_confirmation();
    assert_eq!(flow.complete_save(true), Some(Op::New));
    // Save completion does not mutate an independently held gate.
    flow.begin_dialog();
    assert_eq!(flow.complete_save(false), None);
    assert!(flow.dialog_open());
}
