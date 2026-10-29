//! Convert errors only at presentation boundaries; domain errors remain typed.
use std::error::Error;

/// Build a terminal diagnostic retaining every message in the source chain.
/// This accepts borrowed errors so recoverable UI failures remain available.
pub fn report(error: &dyn Error) -> miette::Report {
    let mut messages = vec![error.to_string()];
    let mut source = error.source();
    while let Some(error) = source {
        messages.push(error.to_string());
        source = error.source();
    }
    let mut messages = messages.into_iter().rev();
    let mut report = miette::Report::msg(messages.next().unwrap_or_default());
    for message in messages {
        report = report.wrap_err(message);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::EditorError;

    #[test]
    fn file_report_keeps_path_and_cause_without_terminal_escapes() {
        let error = EditorError::Open {
            path: "missing.voxl".into(),
            source: Box::new(EditorError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "sample file is missing",
            ))),
        };
        let report = report(&error);
        let mut output = String::new();
        miette::NarratableReportHandler::new()
            .render_report(&mut output, report.as_ref())
            .unwrap();
        assert!(output.contains("Could not open missing.voxl"), "{output}");
        assert!(output.contains("Caused by:"), "{output}");
        assert!(output.contains("sample file is missing"), "{output}");
        assert!(!output.contains('\u{1b}'));
        assert!(matches!(error, EditorError::Open { .. }));
    }

    #[test]
    fn plugin_report_retains_identity_and_quarantine_context() {
        let error = EditorError::Host(quantum_plugin_host::HostError::Faulted {
            plugin_id: quantum_plugin_api::PluginId::new("test.plugin").unwrap(),
            phase: quantum_plugin_host::Phase::Export,
            message: "sample plugin failure".into(),
        });
        let output = format!("{:?}", report(&error));
        assert!(output.contains("test.plugin"), "{output}");
        assert!(output.contains("sample plugin failure"), "{output}");
        assert!(matches!(error, EditorError::Host(_)));
    }
}
