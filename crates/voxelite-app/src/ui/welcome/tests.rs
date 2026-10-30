use super::{Selection, grouped_count, queue_preview, reconcile_selection};
use std::{collections::VecDeque, path::PathBuf};
use voxelite_app::samples::Sample;

#[test]
fn selection_tracks_paths_and_survives_reorder_removal_and_empty_history() {
    let paths: Vec<PathBuf> = (0..12).map(|i| format!("{i}.voxl").into()).collect();
    let selection = Some(Selection::Recent(paths[11].clone()));
    let mut reordered = paths.clone();
    reordered.swap(0, 11);
    assert_eq!(
        reconcile_selection(&paths, &reordered, selection.clone()),
        selection
    );
    assert_eq!(
        reconcile_selection(&paths, &paths[..11], selection.clone()),
        Some(Selection::Recent(paths[10].clone()))
    );
    assert_eq!(
        reconcile_selection(&paths, &[], selection),
        Some(Selection::Sample(Sample::Autumn))
    );
    assert_eq!(reconcile_selection(&[], &paths, None), None);
    let sample = Some(Selection::Sample(Sample::Snow));
    assert_eq!(reconcile_selection(&paths, &[], sample.clone()), sample);
}

#[test]
fn arrows_visit_every_recent_and_sample_in_display_order_and_clamp() {
    for count in [0, 1, 5, 12] {
        let paths: Vec<PathBuf> = (0..count).map(|i| format!("{i}.voxl").into()).collect();
        let expected: Vec<_> = paths
            .iter()
            .cloned()
            .map(Selection::Recent)
            .chain(Sample::ALL.into_iter().map(Selection::Sample))
            .collect();
        let mut selection = Selection::at(&paths, 0);
        assert_eq!(selection.step(&paths, false), selection);
        for item in &expected {
            assert_eq!(&selection, item);
            selection = selection.step(&paths, true);
        }
        assert_eq!(&selection, expected.last().unwrap());
        for item in expected.iter().rev() {
            assert_eq!(&selection, item);
            selection = selection.step(&paths, false);
        }
        assert_eq!(selection, expected[0]);
    }
}

#[cfg(feature = "test-support")]
fn window_fixture(
    cx: &mut gpui::TestAppContext,
    count: usize,
) -> (gpui::WindowHandle<super::WelcomeView>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let paths: Vec<PathBuf> = (0..count)
        .map(|i| dir.path().join(format!("{i}.voxl")))
        .collect();
    cx.update(|cx| {
        crate::actions::install(cx);
        super::settings::init_at(
            None,
            Ok(Default::default()),
            dir.path().join("settings.json"),
            cx,
        );
        cx.set_global(super::Launcher {
            editor: None,
            welcome: None,
            socket: None,
            projects: paths
                .iter()
                .map(|path| voxelite_app::recents::RecentProject {
                    path: path.clone(),
                    name: "Project".into(),
                    modified: std::time::SystemTime::UNIX_EPOCH,
                })
                .collect(),
            error: None,
            work: None,
            notification: None,
            continuation: None,
            revision: 0,
            preview_work: None,
            preview_queue: VecDeque::new(),
        });
    });
    let window = cx.add_window(|window, cx| {
        let mut view = super::WelcomeView::initial(cx);
        view.project_paths = paths;
        window.focus(&view.focus, cx);
        view
    });
    cx.run_until_parked();
    (window, dir)
}

#[cfg(feature = "test-support")]
fn keys(
    cx: &mut gpui::TestAppContext,
    handle: gpui::WindowHandle<super::WelcomeView>,
    input: &str,
) {
    for key in input.split_whitespace() {
        use gpui::{AppContext, InputEvent};
        // simulate_keystrokes emits key-down only; buttons activate on key-up.
        cx.update_window(handle.into(), |_, window, cx| window.draw(cx).clear(cx))
            .unwrap();
        cx.simulate_keystrokes(handle.into(), key);
        let _ = cx.update_window(handle.into(), |_, window, cx| {
            window.dispatch_event(
                gpui::KeyUpEvent {
                    keystroke: gpui::Keystroke::parse(key).unwrap(),
                }
                .to_platform_input(),
                cx,
            );
        });
        cx.run_until_parked();
    }
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn shutdown_drains_ordered_history_writes_without_ui_callbacks(cx: &mut gpui::TestAppContext) {
    let (_handle, dir) = window_fixture(cx, 0);
    let path = dir.path().join("saved.voxl");
    std::fs::write(&path, b"project").unwrap();
    cx.update(|cx| {
        super::observe_quit(cx);
        super::history_at(
            super::HistoryChange::Record(path.clone(), "First".into()),
            dir.path().into(),
            cx,
        );
        super::history_at(super::HistoryChange::Clear, dir.path().into(), cx);
        super::history_at(
            super::HistoryChange::Record(path.clone(), "Last".into()),
            dir.path().into(),
            cx,
        );
    });
    cx.quit();
    let projects = voxelite_app::recents::Recents::new(dir.path())
        .load()
        .unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "Last");
    assert_eq!(projects[0].path, path.canonicalize().unwrap());
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn tab_cycles_sidebar_toggle_and_list_in_both_directions(cx: &mut gpui::TestAppContext) {
    let (handle, dir) = window_fixture(cx, 0);
    for index in [0, 1, 2, 3, 4, 0] {
        keys(cx, handle, "tab");
        handle
            .update(cx, |view, window, _| {
                let focus = if index < 4 {
                    &view.sidebar_focus[index]
                } else {
                    &view.list_focus
                };
                assert!(focus.is_focused(window), "Tab stop {index}");
            })
            .unwrap();
    }
    for index in [4, 3, 2, 1, 0, 4] {
        keys(cx, handle, "shift-tab");
        handle
            .update(cx, |view, window, _| {
                let focus = if index < 4 {
                    &view.sidebar_focus[index]
                } else {
                    &view.list_focus
                };
                assert!(focus.is_focused(window), "reverse Tab stop {index}");
            })
            .unwrap();
    }
    keys(cx, handle, "shift-tab space");
    assert!(
        !voxelite_app::preferences::Preferences::load(&dir.path().join("settings.json"))
            .unwrap()
            .show_welcome
    );
    keys(cx, handle, "enter");
    assert!(
        voxelite_app::preferences::Preferences::load(&dir.path().join("settings.json"))
            .unwrap()
            .show_welcome
    );
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn real_keys_cross_recents_and_samples_without_stealing_sidebar_activation(
    cx: &mut gpui::TestAppContext,
) {
    let (handle, _dir) = window_fixture(cx, 1);
    keys(cx, handle, "down down down down down");
    handle
        .update(cx, |view, window, _| {
            assert_eq!(view.selected_item(), Selection::Sample(Sample::Snow));
            assert!(view.list_focus.is_focused(window));
        })
        .unwrap();
    keys(cx, handle, "up up up up up");
    handle
        .update(cx, |view, _, _| {
            assert!(matches!(view.selected_item(), Selection::Recent(_)))
        })
        .unwrap();
    keys(cx, handle, "ctrl-down alt-up");
    handle
        .update(cx, |view, _, _| {
            assert!(matches!(view.selected_item(), Selection::Recent(_)))
        })
        .unwrap();
    // Tab wraps from the list to New; arrows there must leave list selection alone.
    keys(cx, handle, "tab down tab tab enter");
    handle
        .update(cx, |view, window, _| {
            assert_eq!(view.selected_item(), Selection::Sample(Sample::Autumn));
            assert!(view.list_focus.is_focused(window));
        })
        .unwrap();
    keys(cx, handle, "down");
    keys(
        cx,
        handle,
        &crate::shortcuts::ShortcutPlatform::current().binding_key("secondary-shift-o"),
    );
    handle
        .update(cx, |view, window, _| {
            assert_eq!(view.selected_item(), Selection::Sample(Sample::Autumn));
            assert!(view.list_focus.is_focused(window));
        })
        .unwrap();
    keys(cx, handle, "escape");
    cx.read(|cx| assert!(handle.read(cx).is_err()));
}

#[cfg(feature = "test-support")]
#[gpui::test]
fn enter_on_a_removed_file_preserves_welcome_and_reports_the_error(cx: &mut gpui::TestAppContext) {
    let (handle, _dir) = window_fixture(cx, 1);
    keys(cx, handle, "enter");
    handle
        .update(cx, |view, _, _| {
            assert!(!view.busy);
            assert!(view.error.is_some());
        })
        .unwrap();
}

#[test]
fn preview_backlog_coalesces_saves_and_bounds_distinct_paths() {
    let mut queue = VecDeque::new();
    for i in 0..100 {
        queue_preview(&mut queue, format!("{i}.voxl").into());
    }
    assert_eq!(queue.len(), voxelite_app::recents::CAPACITY);
    for _ in 0..100 {
        queue_preview(&mut queue, "99.voxl".into());
    }
    assert_eq!(queue.len(), voxelite_app::recents::CAPACITY);
    assert_eq!(queue.back(), Some(&PathBuf::from("99.voxl")));
    assert_eq!(queue.front(), Some(&PathBuf::from("88.voxl")));
}

#[test]
fn sample_counts_use_thousands_separators() {
    for (count, label) in [
        (0, "0"),
        (999, "999"),
        (1000, "1,000"),
        (27817, "27,817"),
        (1000000, "1,000,000"),
    ] {
        assert_eq!(grouped_count(count), label);
    }
}
