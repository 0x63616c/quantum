use std::{
    backtrace::Backtrace,
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
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

/// Cloneable live log tail. Retains at most 100 lines, each bounded to 16,384 characters.
/// The app pushes lines during normal operation; capture never waits on its lock.
#[derive(Clone, Debug)]
pub struct LogTail {
    lines: Arc<Mutex<VecDeque<String>>>,
    capacity: usize,
}

impl LogTail {
    /// Create a tail with the requested capacity, capped at 100 (zero disables logs).
    pub fn new(capacity: usize) -> Self {
        Self {
            lines: Arc::new(Mutex::new(VecDeque::new())),
            capacity: capacity.min(100),
        }
    }

    /// Append a bounded line during normal operation. No redaction occurs until capture.
    pub fn push(&self, line: &str) {
        if self.capacity == 0 {
            return;
        }
        let line = bounded(line);
        let mut lines = self.lines.lock().unwrap_or_else(|e| e.into_inner());
        if lines.len() == self.capacity {
            lines.pop_front();
        }
        lines.push_back(line);
    }

    pub(crate) fn capture(&self, redactor: &Redactor) -> Vec<String> {
        match self.lines.try_lock() {
            Ok(lines) => lines.iter().map(|s| redactor.redact(s)).collect(),
            Err(_) => Vec::new(),
        }
    }
}

/// Precomputed panic-hook state with a shared live log tail.
pub struct HookConfig {
    /// App-owned private directory for pending reports.
    pub directory: PathBuf,
    /// Platform information supplied before installing the hook.
    pub environment: Environment,
    /// Compiled privacy policy.
    pub redactor: Redactor,
    /// Live bounded log handle; the app retains a clone to push recent lines.
    pub log_tail: LogTail,
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
/// happen before capture. Log capture uses try_lock and skips contended/poisoned
/// buffers; no caller callbacks or network run during capture.
pub fn install_panic_hook(config: HookConfig) -> Result<(), Error> {
    fs::create_dir_all(&config.directory)?;
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
            log_lines: config.log_tail.capture(&config.redactor),
            user_note: None,
            contact_email: None,
            scene_attachment: false,
        });
        let _ = write(&config.directory, &report, &config.redactor);
        previous(info);
    }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_tail_bounds_capacity_and_lines_and_skips_unavailable_lock() {
        let tail = LogTail::new(200);
        let clone = tail.clone();
        for i in 0..101 {
            clone.push(&i.to_string());
        }
        let redactor = Redactor::new(None).unwrap();
        let lines = tail.capture(&redactor);
        assert_eq!(lines.len(), 100);
        assert_eq!(lines[0], "1");
        clone.push(&"雪".repeat(20_000));
        assert_eq!(
            tail.capture(&redactor).last().unwrap().chars().count(),
            16_384
        );
        let guard = tail.lines.lock().unwrap();
        assert!(tail.capture(&redactor).is_empty());
        drop(guard);
        let _ = std::panic::catch_unwind(|| {
            let _guard = tail.lines.lock().unwrap();
            panic!("poison test");
        });
        assert!(tail.capture(&redactor).is_empty());
        let disabled = LogTail::new(0);
        disabled.push("ignored");
        assert!(disabled.capture(&redactor).is_empty());
    }
}
