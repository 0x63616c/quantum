use super::*;

fn three_steps() -> Editor {
    let mut editor = Editor::blank().unwrap();
    for x in 0..3 {
        editor.apply_at([x, 0, 0]).unwrap();
    }
    editor.engine.mark_saved();
    editor
}

#[test]
fn playback_restarts_at_end_visits_every_state_and_stops_without_mutation() {
    let mut e = three_steps();
    let live = e.engine.snapshot();
    let revision = e.engine.revision();
    let head = e.engine.head();
    let entries: Vec<_> = e.timeline_entries().collect();
    let start = Instant::now();

    assert!(e.play_history(start).unwrap());
    assert!(e.history_playing());
    assert_eq!(e.timeline_index(), 0);
    assert!(e.display_snapshot().layers[0].voxels.is_empty());
    assert!(!e.tick_history(start + Duration::from_millis(249)).unwrap());
    for (index, entry) in entries.iter().enumerate().skip(1) {
        let now = start + HISTORY_STEP_INTERVAL * index as u32;
        assert!(e.tick_history(now).unwrap());
        assert_eq!(e.timeline_index(), index);
        assert_eq!(
            e.display_snapshot(),
            &e.engine.preview(*entry).unwrap().content
        );
        assert_eq!(e.history_playing(), index + 1 < entries.len());
    }
    assert!(!e.tick_history(start + Duration::from_secs(10)).unwrap());
    assert_eq!(e.engine.snapshot(), live);
    assert_eq!(e.engine.revision(), revision);
    assert_eq!(e.engine.head(), head);
    assert!(!e.engine.is_dirty());
    assert_eq!(e.timeline_entries().collect::<Vec<_>>(), entries);
    // The final preview also restarts, not only the live head.
    assert!(e.play_history(start + Duration::from_secs(11)).unwrap());
    assert_eq!(e.timeline_index(), 0);
}

#[test]
fn pause_resume_starts_at_current_entry_with_a_fresh_delay_and_no_catch_up() {
    let mut e = three_steps();
    let first = e.timeline_entries().nth(1).unwrap();
    e.preview_history(first).unwrap();
    let start = Instant::now();
    e.play_history(start).unwrap();
    assert_eq!(e.timeline_index(), 1);
    e.pause_history();
    assert!(!e.tick_history(start + Duration::from_secs(2)).unwrap());
    assert_eq!(e.timeline_index(), 1);

    let resume = start + Duration::from_secs(3);
    e.play_history(resume).unwrap();
    assert!(!e.tick_history(resume + Duration::from_millis(249)).unwrap());
    let delayed = resume + Duration::from_secs(20);
    assert!(e.tick_history(delayed).unwrap());
    assert_eq!(e.timeline_index(), 2);
    assert!(!e.tick_history(delayed).unwrap());
    assert!(
        !e.tick_history(delayed + Duration::from_millis(249))
            .unwrap()
    );
    assert!(e.tick_history(delayed + HISTORY_STEP_INTERVAL).unwrap());
    assert!(!e.history_playing());
}

#[test]
fn playback_from_undone_live_head_preserves_the_redo_stack() {
    let mut e = three_steps();
    e.undo().unwrap();
    let redo = e.engine.redo_head();
    let live = e.engine.snapshot();
    let revision = e.engine.revision();
    let start = Instant::now();
    e.play_history(start).unwrap();
    assert_eq!(e.timeline_index(), 2);
    assert!(e.tick_history(start + HISTORY_STEP_INTERVAL).unwrap());
    assert_eq!(e.timeline_index(), 3);
    assert!(!e.history_playing());
    assert_eq!(e.engine.redo_head(), redo);
    assert_eq!(e.engine.snapshot(), live);
    assert_eq!(e.engine.revision(), revision);
    e.return_live();
    e.redo().unwrap();
    assert_eq!(e.snapshot().layers[0].voxels.len(), 3);
}

#[test]
fn user_navigation_and_even_rejected_edit_attempts_pause_playback() {
    let actions: &[fn(&mut Editor)] = &[
        |e| e.preview_history(None).unwrap(), // Same selected entry.
        |e| {
            let last = e.engine.head();
            e.preview_history(last).unwrap();
        },
        |e| assert!(e.preview_history(Some(Uuid::nil())).is_err()),
        |e| assert!(e.preview_version(Uuid::nil()).is_err()),
        |e| assert!(e.apply_at([5, 0, 0]).is_err()),
        |e| assert!(e.add_layer().is_err()),
        |e| assert!(e.save_version("During playback".into()).is_err()),
        |e| assert!(e.undo().is_err()),
        |e| assert!(e.redo().is_err()),
        |e| assert!(e.revert_step(e.engine.head().unwrap()).is_err()),
        |e| {
            let ray = e.camera.ray([0.5, 0.5], 1.0).unwrap();
            let _ = e.click(ray, false);
        },
        Editor::return_live,
    ];
    for action in actions {
        let mut e = three_steps();
        let start = Instant::now();
        let live = e.engine.snapshot();
        let revision = e.engine.revision();
        e.play_history(start).unwrap();
        action(&mut e);
        assert!(!e.history_playing());
        assert!(!e.tick_history(start + Duration::from_secs(1)).unwrap());
        assert_eq!(e.engine.snapshot(), live);
        assert_eq!(e.engine.revision(), revision);
        assert!(!e.engine.is_dirty());
    }
}

#[test]
fn version_preview_and_restore_interrupt_playback() {
    let mut e = three_steps();
    let version = e.save_version("Finished".into()).unwrap();
    let start = Instant::now();
    e.play_history(start).unwrap();
    e.preview_version(version).unwrap();
    assert!(!e.history_playing());
    assert_eq!(
        e.preview.as_ref().unwrap().source,
        PreviewSource::Version(version)
    );
    // A named version has no timeline cursor: playback begins at the first state.
    e.play_history(start).unwrap();
    assert_eq!(e.timeline_index(), 0);
    e.restore_preview().unwrap();
    assert!(!e.history_playing());
    assert!(e.preview.is_none());
    assert!(e.snapshot().layers[0].voxels.is_empty());
    assert!(!e.tick_history(start + Duration::from_secs(1)).unwrap());
    e.undo().unwrap();
    assert_eq!(e.snapshot().layers[0].voxels.len(), 3);
}

#[test]
fn live_changes_stop_playback_and_keep_the_captured_preview() {
    for notify in [false, true] {
        let mut e = three_steps();
        let start = Instant::now();
        e.play_history(start).unwrap();
        let preview = e.display_snapshot().clone();
        let revision = e.engine.revision();
        let head = e.engine.head().unwrap();
        e.engine.undo(revision, head).unwrap();
        if notify {
            e.agent_updated();
            assert!(!e.history_playing());
        }
        assert!(!e.tick_history(start + HISTORY_STEP_INTERVAL).unwrap());
        assert!(!e.history_playing());
        assert_eq!(e.display_snapshot(), &preview);
        assert_eq!(e.engine.revision(), revision + 1);
        assert!(e.restore_preview().is_err());
    }
}

#[test]
fn empty_and_evicted_history_use_only_retained_states() {
    let start = Instant::now();
    let mut e = Editor::blank().unwrap();
    assert!(!e.play_history(start).unwrap());
    assert!(!e.history_playing());
    assert!(e.preview.is_none());

    // Leave room for growing snapshots, but force early entries out.
    e.apply_at([0, 0, 0]).unwrap();
    let first_bytes = e.engine.history_bytes();
    e.apply_at([1, 0, 0]).unwrap();
    let budget = (e.engine.history_bytes() - first_bytes) * 3;
    e = Editor::blank().unwrap();
    e.engine = Engine::with_limits(
        e.engine.document().clone(),
        Limits {
            history_bytes: budget,
            ..Limits::default()
        },
    )
    .unwrap();
    for x in 0..6 {
        e.apply_at([x, 0, 0]).unwrap();
    }
    assert!(!e.engine.initial_retained());
    let entries: Vec<_> = e.timeline_entries().collect();
    assert!(entries.len() >= 2);
    assert!(entries.iter().all(Option::is_some));
    assert!(e.play_history(start).unwrap());
    assert_eq!(
        e.preview.as_ref().unwrap().source,
        PreviewSource::History(entries[0])
    );
    for index in 1..entries.len() {
        assert!(
            e.tick_history(start + HISTORY_STEP_INTERVAL * index as u32)
                .unwrap()
        );
        assert_eq!(e.timeline_index(), index);
    }
    assert!(!e.history_playing());
}
