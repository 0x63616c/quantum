//! Shell policy for opt-in diagnostics; transport is deliberately user initiated.
use std::path::PathBuf;
use voxelite_crash::{Environment, FeedbackKind, FeedbackReport, Redactor, Report};

/// Failure compiling privacy rules or reading local report storage.
#[derive(Debug, thiserror::Error)]
pub enum DiagnosticsError {
    /// Privacy rules could not be compiled; do not capture unredacted reports.
    #[error("could not compile diagnostics privacy rules: {0}")]
    Privacy(String),
    /// Local crash-report storage failed.
    #[error(transparent)]
    Storage(#[from] voxelite_crash::Error),
}

/// Directory containing pending local panic reports.
pub fn directory() -> PathBuf {
    crate::local_settings::data_dir().join("crashes")
}

/// Compile privacy rules before capturing a panic or preparing a preview.
pub fn redactor() -> Result<Redactor, DiagnosticsError> {
    Redactor::new(std::env::var("HOME").ok().as_deref())
        .map_err(|error| DiagnosticsError::Privacy(error.to_string()))
}

/// Collect only product, operating-system version and architecture metadata.
pub fn environment() -> Environment {
    let (program, argument) = if cfg!(target_os = "macos") {
        ("sw_vers", "-productVersion")
    } else {
        ("uname", "-r")
    };
    let version = std::process::Command::new(program)
        .arg(argument)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Unavailable".into());
    Environment {
        app_version: env!("CARGO_PKG_VERSION").into(),
        os_name: std::env::consts::OS.into(),
        os_version: version,
        arch: std::env::consts::ARCH.into(),
    }
}

/// Compose feedback without collecting email, scene data or document metadata.
pub fn feedback(environment: Environment, text: String, kind: FeedbackKind) -> Report {
    Report::Feedback(FeedbackReport {
        environment,
        what_happened: text,
        kind,
        contact_email: None,
    })
}

/// Read all valid pending reports. Corrupt reports stay on disk for recovery.
pub fn pending() -> Result<Vec<(PathBuf, Report)>, DiagnosticsError> {
    let redactor = redactor()?;
    Ok(voxelite_crash::list_pending(&directory())?
        .into_iter()
        .filter_map(|path| match voxelite_crash::load(&path, &redactor) {
            Ok(report @ Report::Crash(_)) => Some((path, report)),
            Ok(_) => None,
            Err(_) => {
                eprintln!("Could not read a pending crash report; retained locally.");
                None
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feedback_preview_and_browser_preserve_consent_and_template_fields() {
        let report = feedback(
            environment(),
            "Open /Users/test/private.voxl test@example.com".into(),
            FeedbackKind::Feature,
        );
        let redactor = Redactor::new(None).unwrap();
        let payload = report.payload("feat: Feedback", &redactor).unwrap();
        assert!(!payload.body.contains("test@example.com"));
        assert!(!payload.body.contains("/Users/test"));
        assert!(payload.contact_email.is_none());
        let url = payload.github_url(16_000).unwrap();
        assert!(url.contains("template=feature.yml"));
        assert!(url.contains("details="));
        assert!(url.contains("why="));
        assert!(url.contains("labels=type%3Afeat"));
        assert!(payload.github_url(100).is_err());
    }
}
