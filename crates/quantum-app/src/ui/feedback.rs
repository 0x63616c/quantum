//! A document-independent report window, available from Welcome and the editor.
use super::*;
use quantum_app::diagnostics;
use quantum_crash::{FeedbackKind, Report};
use quantum_ui::{TextField, TextFieldEvent};

const PRIVACY: &str = "Nothing is sent automatically. Review the complete redacted report below. Continue opens GitHub in your browser; you must sign in and submit the public issue there. No relay is configured. Email and scene attachments are unavailable. Home paths and free-text emails are redacted; check for other private details.";

#[derive(Default)]
struct ReportWindow(Option<WindowHandle<FeedbackView>>);
impl Global for ReportWindow {}

#[derive(Clone, Debug, Action, PartialEq, Eq)]
#[action(no_json)]
struct CloseReport;

pub(super) fn show_deferred(cx: &mut App) {
    open_later(None, cx);
}

// A deferred effect can flush from an X11 portal callback while the platform
// client is borrowed. Open on the executor's next turn, outside that callback.
fn open_later(pending: Option<(PathBuf, Report)>, cx: &mut App) {
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(std::time::Duration::from_millis(1))
            .await;
        cx.update(|cx| {
            if show(pending, cx).is_err() {
                eprintln!("Could not open report window; local crash reports are retained.");
            }
        });
    })
    .detach();
}

/// Called after either Welcome or an editor has opened, never by the panic hook.
pub(crate) fn offer_pending(cx: &mut App) {
    match diagnostics::pending() {
        Ok(reports) => {
            if let Some(report) = reports.into_iter().next() {
                open_later(Some(report), cx);
            }
        }
        Err(_) => eprintln!("Could not list pending crash reports; retained locally."),
    }
}

fn show(pending: Option<(PathBuf, Report)>, cx: &mut App) -> anyhow::Result<()> {
    if !cx.has_global::<ReportWindow>() {
        cx.set_global(ReportWindow::default());
        cx.bind_keys([KeyBinding::new(
            &ShortcutPlatform::current().binding_key("secondary-w"),
            CloseReport,
            Some("ReportWindow"),
        )]);
    }
    if let Some(handle) = cx.global::<ReportWindow>().0
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        return Ok(());
    }
    let bounds = Bounds::centered(
        None,
        size(px(metrics::REPORT_WIDTH), px(metrics::REPORT_HEIGHT)),
        cx,
    );
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Quantum Feedback".into()),
                ..Default::default()
            }),
            app_id: Some(quantum_app::APP_ID.into()),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| FeedbackView::new(pending, window, cx)),
    )?;
    cx.global_mut::<ReportWindow>().0 = Some(handle);
    Ok(())
}

struct FeedbackView {
    pending: Option<PathBuf>,
    report: Report,
    input: Entity<TextField>,
    preview: Option<String>,
    error: Option<String>,
    focus: FocusHandle,
    endpoint: Option<String>,
    sending: bool,
    capture_only: bool,
}

impl FeedbackView {
    fn new(
        pending: Option<(PathBuf, Report)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (path, report) = match pending {
            Some((path, report)) => (Some(path), report),
            None => (
                None,
                diagnostics::feedback(diagnostics::environment(), String::new(), FeedbackKind::Bug),
            ),
        };
        let input = TextField::design("What happened, or what would you like to change?", cx);
        cx.subscribe(&input, |this, _, event, cx| {
            if matches!(event, TextFieldEvent::Changed) {
                this.preview = None;
                this.error = None;
                cx.notify();
            }
        })
        .detach();
        cx.observe_global::<settings::SettingsState>(|_, cx| cx.notify())
            .detach();
        cx.observe_window_appearance(window, |_, _, cx| cx.notify())
            .detach();
        cx.observe_window_activation(window, |_, _, cx| {
            super::sync_reduced_motion(cx);
            cx.notify();
        })
        .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let mut view = Self {
            pending: path,
            report,
            input,
            preview: None,
            error: None,
            focus,
            endpoint: diagnostics::endpoint(),
            sending: false,
            capture_only: false,
        };
        if view.is_crash() {
            view.prepare(cx);
        } else {
            view.input.update(cx, |field, cx| field.focus(window, cx));
        }
        view
    }

    fn is_crash(&self) -> bool {
        matches!(self.report, Report::Crash(_))
    }

    fn prepare(&mut self, cx: &mut Context<Self>) {
        let note = self.input.read(cx).text().to_owned();
        match &mut self.report {
            Report::Crash(report) => report.user_note = (!note.trim().is_empty()).then_some(note),
            Report::Feedback(report) => {
                if note.trim().is_empty() {
                    self.error = Some("Describe what happened before previewing.".into());
                    cx.notify();
                    return;
                }
                report.what_happened = note;
            }
        }
        match diagnostics::redactor()
            .map_err(|e| e.to_string())
            .and_then(|redactor| self.report.markdown(&redactor).map_err(|e| e.to_string()))
        {
            Ok(preview) => {
                self.preview = Some(preview);
                self.error = None;
            }
            Err(_) => {
                self.error = Some("Could not prepare a private preview. Nothing was sent.".into())
            }
        }
        cx.notify();
    }

    fn continue_in_browser(&mut self, cx: &mut Context<Self>) {
        if self.endpoint.is_some() {
            self.submit(cx);
            return;
        }
        if self.preview.is_none() {
            return;
        }
        let result = diagnostics::redactor()
            .map_err(|e| e.to_string())
            .and_then(|redactor| {
                self.report
                    .payload(
                        if self.is_crash() {
                            "fix: Quantum crash report"
                        } else {
                            match &self.report {
                                Report::Feedback(r) if r.kind == FeedbackKind::Feature => {
                                    "feat: Quantum feedback"
                                }
                                _ => "fix: Quantum feedback",
                            }
                        },
                        &redactor,
                    )
                    .and_then(|payload| payload.github_url(8_000))
                    .map_err(|e| e.to_string())
            });
        match result {
            Ok(url) => {
                cx.open_url(&url);
                self.error = Some("Opened GitHub. Submit there to send. Local crash reports are kept until you choose Delete report.".into());
            }
            Err(_) => self.error = Some("This report is too long for a browser link. Copy or save the complete preview, then paste it into a new GitHub issue. Nothing was sent.".into()),
        }
        cx.notify();
    }

    fn delete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = &self.pending {
            if quantum_crash::dismiss(path).is_err() {
                self.error = Some("Could not delete the local report. It has been kept.".into());
                cx.notify();
                return;
            }
            cx.global_mut::<ReportWindow>().0 = None;
            window.remove_window();
            offer_pending(cx);
        }
    }

    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(preview) = self.preview.clone() else {
            return;
        };
        let receiver = cx.prompt_for_new_path(&std::env::temp_dir(), Some("quantum-report.md"));
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some(path))) => {
                        this.error = Some(
                            match quantum_format::storage::safe_write(path, preview.as_bytes()) {
                                Ok(()) => "Saved the redacted preview locally.".into(),
                                Err(_) => {
                                    "Could not save the report. The original is retained.".into()
                                }
                            },
                        );
                    }
                    Ok(Ok(None)) => {}
                    _ => this.error = Some("Could not show the Save dialog.".into()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

mod menu;
mod relay;
mod view;

pub(super) fn fixture(crash: bool, relay: bool, cx: &mut App) {
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(std::time::Duration::from_millis(1))
            .await;
        cx.update(move |cx| {
            let result = (|| -> anyhow::Result<()> {
                let environment = quantum_crash::Environment {
                    app_version: env!("CARGO_PKG_VERSION").into(),
                    os_name: "macOS".into(),
                    os_version: "13.0".into(),
                    arch: "aarch64".into(),
                };
                let pending = if crash {
                    let report = Report::Crash(quantum_crash::CrashReport {
                        environment,
                        panic_message: "Example panic for UI review".into(),
                        location: Some("src/example.rs:12:5".into()),
                        backtrace: "0: quantum::example\n1: quantum::main".into(),
                        timestamp_utc: 0,
                        log_lines: vec!["WARN: Example diagnostic line".into()],
                        user_note: None,
                        contact_email: None,
                        scene_attachment: false,
                    });
                    let redactor = diagnostics::redactor().map_err(anyhow::Error::new)?;
                    let path = quantum_crash::write(&diagnostics::directory(), &report, &redactor)?;
                    Some((path, report))
                } else {
                    None
                };
                show(pending, cx)?;
                if relay && let Some(handle) = cx.global::<ReportWindow>().0 {
                    handle.update(cx, |view, _, _| {
                        view.capture_only = true;
                        view.endpoint =
                            Some("https://quantum-feedback.worldwidewebb.co/reports".into())
                    })?;
                }
                if !crash && let Some(handle) = cx.global::<ReportWindow>().0 {
                    handle.update(cx, |view, _, cx| {
                        view.input.update(cx, |field, cx| {
                            field.set(
                            "The palette closes when I choose a color. I expected it to stay open."
                                .into(),
                            cx,
                        )
                        });
                        let entity = cx.entity();
                        cx.defer(move |cx| entity.update(cx, |view, cx| view.prepare(cx)));
                    })?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                eprintln!("Could not open report fixture: {error}");
            }
        });
    })
    .detach();
}

#[cfg(all(test, feature = "test-support"))]
mod tests;
