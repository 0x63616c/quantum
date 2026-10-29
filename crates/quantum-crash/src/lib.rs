//! Opt-in diagnostics. Capturing and preparing reports never invokes transport.

use serde::{Deserialize, Serialize};

mod privacy;
mod relay;
mod storage;
pub use privacy::Redactor;
pub use relay::{RelayPayload, RelayRequest, RelayTransport, SendError};
pub use storage::{
    HookConfig, LogTail, dismiss, install_panic_hook, list_pending, load, mark_sent, write,
};

/// Failure preparing or storing a report.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Local filesystem failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Invalid report JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Invalid or unsafe endpoint/header configuration.
    #[error("invalid relay configuration")]
    Configuration,
    /// The complete encoded fallback cannot fit the requested byte limit.
    #[error("prefilled issue URL exceeds length cap")]
    UrlTooLong,
}

/// Caller-supplied platform metadata; no OS commands or environment collection.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Environment {
    /// Quantum version.
    pub app_version: String,
    /// Operating system name.
    pub os_name: String,
    /// Operating system version, supplied by the shell.
    pub os_version: String,
    /// CPU architecture.
    pub arch: String,
}

/// Captured panic data. Use a redactor at every persistence/egress boundary.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashReport {
    /// Application and platform versions.
    pub environment: Environment,
    /// Panic payload, or a generic description for non-string payloads.
    pub panic_message: String,
    /// Source location, if available.
    pub location: Option<String>,
    /// Text produced by `std::backtrace::Backtrace`.
    pub backtrace: String,
    /// Seconds since the Unix epoch (UTC).
    pub timestamp_utc: u64,
    /// Caller-supplied trailing log lines; never read from disk by this crate.
    pub log_lines: Vec<String>,
    /// Optional user description.
    pub user_note: Option<String>,
    /// Explicit opt-in contact address. `None` means no contact consent.
    pub contact_email: Option<String>,
    /// Explicit scene consent flag only; no scene bytes are collected or sent.
    pub scene_attachment: bool,
}

/// Feedback classification and GitHub template choice.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackKind {
    /// Unexpected behavior.
    Bug,
    /// Requested behavior.
    Feature,
}

/// A manually composed feedback report.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedbackReport {
    /// Application and platform versions.
    pub environment: Environment,
    /// What happened or what the user wants.
    pub what_happened: String,
    /// Classification.
    pub kind: FeedbackKind,
    /// Explicit opt-in contact address; free-text emails remain redacted.
    pub contact_email: Option<String>,
}

/// Common persistence and rendering envelope.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "report", rename_all = "snake_case")]
pub enum Report {
    /// Local panic diagnostic.
    Crash(CrashReport),
    /// Manually composed feedback.
    Feedback(FeedbackReport),
}

impl Report {
    /// Redacted, stable Markdown preview, identical to the relay issue body.
    /// All user text is quoted to prevent injected Markdown sections/HTML.
    pub fn markdown(&self, redactor: &Redactor) -> Result<String, Error> {
        let report = redactor.report(self)?;
        let (environment, details, scene) = match &report {
            Self::Crash(r) => (
                &r.environment,
                format!(
                    "## Panic\n\n{}\n## Location\n\n{}\n## Backtrace\n\n{}\n## Recent logs\n\n{}\n## User note\n\n{}\n## Timestamp (Unix seconds, UTC)\n\n{}\n",
                    quote(&r.panic_message),
                    quote(r.location.as_deref().unwrap_or("Unavailable")),
                    quote(&r.backtrace),
                    quote(&r.log_lines.join("\n")),
                    quote(r.user_note.as_deref().unwrap_or("None")),
                    r.timestamp_utc
                ),
                r.scene_attachment,
            ),
            Self::Feedback(r) => (
                &r.environment,
                format!("## What happened\n\n{}", quote(&r.what_happened)),
                false,
            ),
        };
        Ok(format!(
            "## What is sent\n\nThis preview becomes a public GitHub issue via the configured relay or browser.\nIncludes version/OS/architecture and the details below.\nContact email (if provided) is separate from the public issue; the configured relay's privacy policy controls its handling. The browser fallback excludes contact email entirely.\nScene consent: {}; scene contents are never included by this crate.\nHome paths and free-text emails are redacted. Review all text before confirming.\n\n## Environment\n\n{}\n{}",
            scene,
            quote(&format!(
                "Quantum {}\n{} {}\n{}",
                environment.app_version,
                environment.os_name,
                environment.os_version,
                environment.arch
            )),
            details
        ))
    }

    /// Prepare the versioned relay payload without sending anything.
    pub fn payload(&self, title: &str, redactor: &Redactor) -> Result<RelayPayload, Error> {
        let kind = match self {
            Self::Feedback(r) => r.kind,
            Self::Crash(_) => FeedbackKind::Bug,
        };
        Ok(RelayPayload {
            contact_email: match redactor.report(self)? {
                Self::Crash(r) => r.contact_email,
                Self::Feedback(r) => r.contact_email,
            },
            schema_version: 1,
            title: redactor.redact(title),
            body: self.markdown(redactor)?,
            labels: vec![
                match kind {
                    FeedbackKind::Bug => "type:fix",
                    FeedbackKind::Feature => "type:feat",
                }
                .into(),
            ],
            kind,
        })
    }
}

fn quote(text: &str) -> String {
    text.lines()
        .map(|line| {
            format!(
                "> {}\n",
                line.replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
