use std::{
    backtrace::Backtrace,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{CrashReport, Environment, Error, Redactor, Report};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_REPORT_BYTES: u64 = 4 * 1024 * 1024;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Atomically write a redacted report (private permissions on Unix). The caller
/// owns directory privacy/retention. Temporary files are never listed as pending.
pub fn write(directory: &Path, report: &Report, redactor: &Redactor) -> Result<PathBuf, Error> {
    let report = redactor.report(report)?;
    let bytes = serde_json::to_vec(&report)?;
    if bytes.len() as u64 > MAX_REPORT_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "report exceeds size limit",
        )
        .into());
    }
    fs::create_dir_all(directory)?;
    let name = format!(
        "crash-{}-{}-{}",
        now(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let temporary = directory.join(format!("{name}.tmp"));
    let final_path = directory.join(format!("{name}.json"));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, &final_path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    Ok(final_path)
}

/// List regular report files in stable filename order; absent directory is empty.
/// This is an app-owned crash directory, not an arbitrary untrusted directory.
pub fn list_pending(directory: &Path) -> Result<Vec<PathBuf>, Error> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_file()
            && path.extension().is_some_and(|s| s == "json")
            && path
                .file_name()
                .is_some_and(|s| s.to_string_lossy().starts_with("crash-"))
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Load a size-bounded local report and reapply the current redaction policy.
pub fn load(path: &Path, redactor: &Redactor) -> Result<Report, Error> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_REPORT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_REPORT_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "report exceeds size limit",
        )
        .into());
    }
    redactor.report(&serde_json::from_slice(&bytes)?)
}

/// Delete a reviewed report after explicit dismissal. Missing files are harmless.
pub fn dismiss(path: &Path) -> Result<(), Error> {
    match fs::remove_file(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result.map_err(Into::into),
    }
}

/// Delete a report after successful sending, only when explicitly called by the app.
pub fn mark_sent(path: &Path) -> Result<(), Error> {
    dismiss(path)
}

/// Precomputed panic-hook state. No caller callbacks or logging locks run in the hook.
pub struct HookConfig {
    /// App-owned private directory for pending reports.
    pub directory: PathBuf,
    /// Platform information supplied before installing the hook.
    pub environment: Environment,
    /// Compiled privacy policy.
    pub redactor: Redactor,
    /// Caller-supplied recent log snapshot; not a live log collector.
    pub log_lines: Vec<String>,
    /// Maximum trailing lines retained (also capped at 100).
    pub max_log_lines: usize,
}

fn bounded(text: &str) -> String {
    text.chars().take(16 * 1024).collect()
}

/// Install once during normal startup, never while unwinding. Creates the crash
/// directory before changing the process-global hook, returning setup I/O failures.
/// Chains the previous hook even when capture fails. Does not install fatal signal
/// handlers. Allocation failure, abort, stack overflow and a panicking previous
/// hook cannot be recovered; this is best-effort Rust panic capture, not a guarantee.
/// Backtrace symbolization necessarily allocates; regex compilation and log bounding
/// happen at installation, with no locks/caller callbacks or network in capture.
pub fn install_panic_hook(mut config: HookConfig) -> Result<(), Error> {
    fs::create_dir_all(&config.directory)?;
    let keep = config.max_log_lines.min(100);
    let skip = config.log_lines.len().saturating_sub(keep);
    config.log_lines = config
        .log_lines
        .iter()
        .skip(skip)
        .map(|s| config.redactor.redact(&bounded(s)))
        .collect();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("Non-string panic payload");
        let report = Report::Crash(CrashReport {
            environment: config.environment.clone(),
            panic_message: bounded(message),
            location: info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())),
            backtrace: bounded(&Backtrace::force_capture().to_string()),
            timestamp_utc: now(),
            log_lines: config.log_lines.clone(),
            user_note: None,
            contact_email: None,
            scene_attachment: false,
        });
        let _ = write(&config.directory, &report, &config.redactor);
        previous(info);
    }));
    Ok(())
}
