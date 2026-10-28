use super::{SearchResult, Section, SectionHistory, search_sections};
use crate::actions::Op;

#[cfg(feature = "test-support")]
fn init_writer(cx: &gpui::TestAppContext, path: std::path::PathBuf) {
    cx.update(|cx| super::init_at(None, Ok(Default::default()), path, cx));
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn show_at_launch_round_trips_through_the_background_writer(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    init_writer(cx, path.clone());
    for enabled in [false, true, false] {
        cx.update(|cx| super::change(cx, |p| p.show_welcome = enabled));
        cx.run_until_parked();
        assert_eq!(
            super::Preferences::load(&path).unwrap().show_welcome,
            enabled
        );
        cx.read(|cx| {
            assert_eq!(
                cx.global::<super::SettingsState>().values.show_welcome,
                enabled
            )
        });
    }
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn rapid_changes_coalesce_into_the_latest_complete_snapshot(cx: &mut gpui::TestAppContext) {
    use std::sync::{Arc, Mutex};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    init_writer(cx, path.clone());
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let saved = snapshots.clone();
    cx.update(|cx| {
        cx.global_mut::<super::persistence::SettingsWrites>().save = Arc::new(move |values| {
            saved.lock().unwrap().push(values.clone());
            values.save(&path)
        });
        super::change(cx, |p| p.show_welcome = false);
        super::change(cx, |p| p.grid = false);
        super::change(cx, |p| p.show_welcome = true);
    });
    cx.run_until_parked();
    let snapshots = snapshots.lock().unwrap();
    assert_eq!(snapshots.len(), 1);
    assert!(snapshots[0].show_welcome);
    assert!(!snapshots[0].grid);
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn quit_drains_pending_settings_without_reentering_the_app(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    init_writer(cx, path.clone());
    cx.update(|cx| {
        super::change(cx, |p| p.grid = false);
        super::change(cx, |p| p.show_welcome = false);
    });
    // Do not run the scheduler first: quit must drain an unstarted worker.
    cx.quit();
    let loaded = super::Preferences::load(&path).unwrap();
    assert!(!loaded.show_welcome);
    assert!(!loaded.grid);
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn failed_write_restores_last_durable_toggle_and_can_retry(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    init_writer(cx, path.clone());
    cx.update(|cx| super::change(cx, |p| p.show_welcome = false));
    cx.run_until_parked();
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    cx.update(|cx| super::change(cx, |p| p.show_welcome = true));
    cx.run_until_parked();
    cx.read(|cx| {
        let state = cx.global::<super::SettingsState>();
        assert!(!state.values.show_welcome);
        assert!(state.error.as_ref().unwrap().contains("not saved"));
    });
    std::fs::remove_dir(&path).unwrap();
    cx.update(|cx| super::change(cx, |p| p.show_welcome = true));
    cx.run_until_parked();
    assert!(super::Preferences::load(&path).unwrap().show_welcome);
    cx.read(|cx| assert!(cx.global::<super::SettingsState>().error.is_none()));
}

#[test]
fn edits_during_io_coalesce_and_drain_even_after_an_older_write_fails() {
    use super::{Preferences, WriteQueue, drain_writes};
    use std::sync::{Arc, Mutex};
    let queue = Arc::new(Mutex::new(WriteQueue {
        committed: Preferences::default(),
        pending: Some(Preferences {
            show_welcome: false,
            ..Default::default()
        }),
        running: true,
        error: None,
    }));
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let saved = snapshots.clone();
    let edits = queue.clone();
    drain_writes(&queue, &move |value| {
        let mut saved = saved.lock().unwrap();
        saved.push(value.clone());
        if saved.len() == 1 {
            let mut queue = edits.lock().unwrap();
            queue.pending = Some(Preferences::default());
            queue.pending = Some(Preferences {
                grid: false,
                ..Default::default()
            });
            return Err(voxelite_app::editor::EditorError::Gesture(
                "disk failure".into(),
            ));
        }
        Ok(())
    });
    let queue = queue.lock().unwrap();
    assert!(!queue.running);
    assert!(queue.pending.is_none());
    assert!(queue.error.is_none());
    assert!(queue.committed.show_welcome);
    assert!(!queue.committed.grid);
    assert_eq!(snapshots.lock().unwrap().len(), 2);
}

#[test]
fn empty_search_keeps_all_sections_without_setting_rows() {
    let results = search_sections("  ");
    assert_eq!(
        results.iter().map(|r| r.section).collect::<Vec<_>>(),
        Section::ALL
    );
    assert!(results.iter().all(|r| r.settings.is_empty()));
}

#[test]
fn search_finds_individual_settings_by_label_and_description() {
    for (query, section, title) in [
        (" ground GRID ", Section::Viewport, "Ground grid"),
        (
            "average frame time",
            Section::Viewport,
            "Render performance",
        ),
        ("smooth edges", Section::Viewport, "Antialiasing"),
        ("ao strength", Section::Viewport, "Ambient occlusion"),
        ("navigation hints", Section::General, "Navigation help"),
        ("reduce motion", Section::General, "Animate camera"),
        ("assistant", Section::Agent, "Project connection"),
        ("reset settings", Section::Advanced, "Restore defaults"),
        ("system dark", Section::Appearance, "Appearance"),
    ] {
        assert_eq!(
            search_sections(query),
            vec![SearchResult {
                section,
                settings: vec![title]
            }],
            "{query}"
        );
    }
}

#[test]
fn search_includes_parent_sections_and_individual_shortcuts() {
    let results = search_sections("save");
    let shortcuts = results
        .iter()
        .find(|r| r.section == Section::Shortcuts)
        .unwrap();
    assert!(shortcuts.settings.contains(&Op::Save.label()));
    assert!(shortcuts.settings.contains(&Op::SaveAs.label()));
    assert!(!shortcuts.settings.contains(&Op::Undo.label()));
    assert!(search_sections("unmatched-xyz").is_empty());
    assert_eq!(
        search_sections("viewport ground")[0].settings,
        vec!["Ground grid"]
    );
}

#[test]
fn history_moves_back_and_forward_and_stops_at_boundaries() {
    let mut history = SectionHistory::new(Section::General);
    assert!(!history.can_back());
    assert!(!history.can_forward());
    history.back();
    history.forward();
    assert_eq!(history.current(), Section::General);
    history.visit(Section::Appearance);
    history.visit(Section::Viewport);
    history.back();
    assert_eq!(history.current(), Section::Appearance);
    assert!(history.can_back());
    assert!(history.can_forward());
    history.back();
    history.back();
    assert_eq!(history.current(), Section::General);
    history.forward();
    history.forward();
    history.forward();
    assert_eq!(history.current(), Section::Viewport);
    assert!(!history.can_forward());
}

#[test]
fn history_preserves_forward_on_reselection_and_truncates_on_new_navigation() {
    let mut history = SectionHistory::new(Section::General);
    history.visit(Section::Appearance);
    history.visit(Section::Viewport);
    history.back();
    history.visit(Section::Appearance);
    assert!(history.can_forward());
    assert_eq!(history.entries.len(), 3);
    history.visit(Section::Agent);
    assert!(!history.can_forward());
    assert_eq!(
        history.entries,
        vec![Section::General, Section::Appearance, Section::Agent]
    );
    history.visit(Section::Agent);
    history.back();
    assert_eq!(history.current(), Section::Appearance);
}
