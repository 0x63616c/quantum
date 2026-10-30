use super::{FeedbackView, Report, diagnostics, settings};
use gpui::TestAppContext;

#[gpui::test]
fn feedback_requires_a_fresh_preview_after_editing(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    cx.update(|cx| {
        settings::init_at(
            None,
            Ok(Default::default()),
            dir.path().join("settings.json"),
            cx,
        )
    });
    let handle = cx.add_window(|window, cx| FeedbackView::new(None, window, cx));
    handle
        .update(cx, |view, _, cx| {
            view.prepare(cx);
            assert!(view.preview.is_none());
            assert!(view.error.is_some());
            view.input.update(cx, |field, cx| {
                field.set(
                    "A bug /Users/private/model.voxl private@example.com".into(),
                    cx,
                )
            });
        })
        .unwrap();
    cx.run_until_parked();
    handle
        .update(cx, |view, _, cx| {
            view.prepare(cx);
            let preview = view.preview.as_ref().unwrap();
            assert!(!preview.contains("/Users/private"));
            assert!(!preview.contains("private@example.com"));
            assert!(preview.contains("Quantum"));
            view.input
                .update(cx, |field, cx| field.set("Changed after review".into(), cx));
        })
        .unwrap();
    cx.run_until_parked();
    handle
        .update(cx, |view, _, _| assert!(view.preview.is_none()))
        .unwrap();
}

#[gpui::test]
fn browser_url_failure_retains_local_report_and_complete_preview(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    cx.update(|cx| {
        settings::init_at(
            None,
            Ok(Default::default()),
            dir.path().join("settings.json"),
            cx,
        )
    });
    let report = Report::Crash(quantum_crash::CrashReport {
        environment: diagnostics::environment(),
        panic_message: "Test".into(),
        location: None,
        backtrace: "Long frame\n".repeat(5000),
        timestamp_utc: 0,
        log_lines: vec![],
        user_note: None,
        contact_email: None,
        scene_attachment: false,
    });
    let path =
        quantum_crash::write(dir.path(), &report, &diagnostics::redactor().unwrap()).unwrap();
    let pending = path.clone();
    let handle =
        cx.add_window(move |window, cx| FeedbackView::new(Some((pending, report)), window, cx));
    handle
        .update(cx, |view, _, cx| {
            let preview = view.preview.clone().unwrap();
            view.continue_in_browser(cx);
            assert_eq!(view.preview.as_ref().unwrap(), &preview);
            assert!(view.error.as_ref().unwrap().contains("too long"));
        })
        .unwrap();
    assert!(path.exists());
}
