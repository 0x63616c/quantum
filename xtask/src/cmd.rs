//! Commands are argument vectors, never shell expressions. Execution is injectable.

use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fmt, io,
    io::{Read, Write},
    path::{Path, PathBuf},
    process,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

#[derive(Clone)]
struct Value {
    value: OsString,
    secret: bool,
}

/// A command specification. Display and Debug redact marked arguments and values.
#[derive(Clone)]
pub struct Command {
    program: OsString,
    args: Vec<Value>,
    env: BTreeMap<OsString, Option<Value>>,
    directory: Option<PathBuf>,
    allow_in_dry_run: bool,
    input: Option<Vec<u8>>,
    during_cleanup: bool,
}

impl Command {
    /// Start a command without invoking a shell.
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Self {
            program: program.as_ref().into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            directory: None,
            allow_in_dry_run: false,
            input: None,
            during_cleanup: false,
        }
    }

    /// Append one literal argument.
    pub fn arg(mut self, value: impl AsRef<OsStr>) -> Self {
        self.args.push(Value {
            value: value.as_ref().into(),
            secret: false,
        });
        self
    }

    /// Append literal arguments, preserving their boundaries and order.
    pub fn args(mut self, values: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self {
        for value in values {
            self = self.arg(value);
        }
        self
    }

    /// Append a secret value. Pass its option name separately with [`Self::arg`].
    pub fn secret_arg(mut self, value: impl AsRef<OsStr>) -> Self {
        self.args.push(Value {
            value: value.as_ref().into(),
            secret: true,
        });
        self
    }

    /// Set a child environment variable without changing the parent environment.
    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.env.insert(
            key.as_ref().into(),
            Some(Value {
                value: value.as_ref().into(),
                secret: false,
            }),
        );
        self
    }

    /// Set a child environment variable whose value must be redacted in diagnostics.
    pub fn secret_env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.env.insert(
            key.as_ref().into(),
            Some(Value {
                value: value.as_ref().into(),
                secret: true,
            }),
        );
        self
    }

    /// Supply secret bytes over a pipe, never argv or the environment. Input is
    /// omitted from Debug/Display and redacted from captured error diagnostics.
    pub fn secret_stdin(mut self, input: impl Into<Vec<u8>>) -> Self {
        self.input = Some(input.into());
        self
    }

    /// Unredacted stdin for runner implementations. Never log these bytes.
    pub fn input(&self) -> Option<&[u8]> {
        self.input.as_deref()
    }

    /// Allow restoration commands to finish after cancellation was requested.
    /// Only use for cleanup, such as deleting a temporary signing keychain.
    pub fn during_cleanup(mut self) -> Self {
        self.during_cleanup = true;
        self
    }

    /// Remove an inherited child environment variable.
    pub fn env_remove(mut self, key: impl AsRef<OsStr>) -> Self {
        self.env.insert(key.as_ref().into(), None);
        self
    }

    /// Select the child's working directory.
    pub fn current_dir(mut self, directory: impl AsRef<Path>) -> Self {
        self.directory = Some(directory.as_ref().into());
        self
    }

    /// Execute even during a dry run. Use only for read-only planning commands.
    pub fn allow_in_dry_run(mut self) -> Self {
        self.allow_in_dry_run = true;
        self
    }

    /// Program to execute. Intended for runner implementations and assertions.
    pub fn program(&self) -> &OsStr {
        &self.program
    }

    /// Unredacted arguments. Do not log these; format the command instead.
    pub fn arguments(&self) -> impl Iterator<Item = &OsStr> {
        self.args.iter().map(|value| value.value.as_os_str())
    }

    /// Child working directory, if explicitly set.
    pub fn directory(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    /// Unredacted environment overrides; `None` removes a variable. Do not log values.
    pub fn environment(&self) -> impl Iterator<Item = (&OsStr, Option<&OsStr>)> {
        self.env
            .iter()
            .map(|(key, value)| (key.as_os_str(), value.as_ref().map(|v| v.value.as_os_str())))
    }

    /// Inherit stdio and fail on a nonzero or terminated child status.
    /// The child's inherited output is not filtered; use capture for secret-bearing tools.
    pub fn run(&self, runner: &dyn Runner) -> Result<(), Error> {
        self.ensure_success(self.execute(runner, false)?)
            .map(|_| ())
    }

    /// Inherit stdio, leaving exit-code policy to the caller.
    pub fn status(&self, runner: &dyn Runner) -> Result<Status, Error> {
        Ok(self.execute(runner, false)?.status)
    }

    /// Capture stdout/stderr as bytes and fail on a nonzero or terminated child status.
    pub fn output(&self, runner: &dyn Runner) -> Result<Output, Error> {
        self.ensure_success(self.execute(runner, true)?)
    }

    /// Capture UTF-8 stdout and trim leading/trailing whitespace.
    pub fn read(&self, runner: &dyn Runner) -> Result<String, Error> {
        let output = self.output(runner)?;
        let text = std::str::from_utf8(&output.stdout).map_err(|error| Error::Utf8 {
            command: self.to_string(),
            detail: error.to_string(),
            status: output.status,
            stderr: stderr_tail(&self.redact(&String::from_utf8_lossy(&output.stderr))),
        })?;
        Ok(text.trim().to_owned())
    }

    fn execute(&self, runner: &dyn Runner, capture: bool) -> Result<Output, Error> {
        runner.execute(self, capture).map_err(|error| Error::Spawn {
            command: self.to_string(),
            detail: self.redact(&error.to_string()),
        })
    }

    fn ensure_success(&self, output: Output) -> Result<Output, Error> {
        if output.status.success() {
            Ok(output)
        } else {
            // Redact before taking the tail so truncation cannot expose part of a secret.
            let stderr = self.redact(&String::from_utf8_lossy(&output.stderr));
            Err(Error::Failed {
                command: self.to_string(),
                status: output.status,
                stderr: stderr_tail(&stderr),
            })
        }
    }

    fn redact(&self, text: &str) -> String {
        let input = self
            .input
            .as_ref()
            .map(|bytes| String::from_utf8_lossy(bytes));
        let mut secrets: Vec<_> = self
            .args
            .iter()
            .chain(self.env.values().flatten())
            .filter(|v| v.secret && !v.value.is_empty())
            .map(|v| v.value.to_string_lossy())
            .collect();
        if let Some(input) = &input {
            for value in [input.as_ref(), input.trim()] {
                if !value.is_empty() {
                    secrets.push(value.into());
                }
            }
        }
        // Replace longer secrets first when one contains another.
        secrets.sort_by_key(|value| std::cmp::Reverse(value.len()));
        let mut text = text.to_owned();
        for secret in secrets {
            text = text.replace(secret.as_ref(), "[REDACTED]");
        }
        text
    }
}

fn quote(value: &OsStr) -> String {
    let value = value.to_string_lossy();
    if !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_=./:".contains(&b))
    {
        value.into_owned()
    } else {
        format!("{value:?}")
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact raw values first: diagnostic quoting escapes newlines, quotes
        // and backslashes, which would prevent matching the original secret.
        let quote = |value: &OsStr| quote(OsStr::new(&self.redact(&value.to_string_lossy())));
        let mut parts = Vec::new();
        if let Some(directory) = &self.directory {
            parts.push(format!("(in {})", quote(directory.as_os_str())));
        }
        for (key, value) in &self.env {
            parts.push(match value {
                Some(value) => format!(
                    "{}={}",
                    quote(key),
                    if value.secret {
                        "[REDACTED]".into()
                    } else {
                        quote(&value.value)
                    }
                ),
                None => format!("(unset {})", quote(key)),
            });
        }
        parts.push(quote(&self.program));
        parts.extend(self.args.iter().map(|value| {
            if value.secret {
                "[REDACTED]".into()
            } else {
                quote(&value.value)
            }
        }));
        if self.input.is_some() {
            parts.push("< [REDACTED stdin]".into());
        }
        f.write_str(&self.redact(&parts.join(" ")))
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// A child's exit status, including signal descriptions on platforms that provide them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {
    code: Option<i32>,
    description: String,
}

impl Status {
    /// Construct an ordinary exit status, useful for fake runners.
    pub fn from_code(code: i32) -> Self {
        Self {
            code: Some(code),
            description: format!("exit status: {code}"),
        }
    }

    /// The numeric exit code, or `None` when terminated by a signal.
    pub fn code(&self) -> Option<i32> {
        self.code
    }

    /// Whether the child exited with code zero.
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

impl From<process::ExitStatus> for Status {
    fn from(status: process::ExitStatus) -> Self {
        Self {
            code: status.code(),
            description: status.to_string(),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.description)
    }
}

/// Captured child output. In inherited-stdio mode the byte buffers are empty.
#[derive(Clone, Debug)]
pub struct Output {
    /// Exit status.
    pub status: Status,
    /// Unredacted stdout bytes.
    pub stdout: Vec<u8>,
    /// Unredacted stderr bytes.
    pub stderr: Vec<u8>,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            status: Status::from_code(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}

/// Injectable process execution. Fakes can inspect commands and return canned results.
pub trait Runner {
    /// Execute using inherited stdio, or capture stdout/stderr when `capture` is true.
    fn execute(&self, command: &Command, capture: bool) -> io::Result<Output>;

    /// Whether callers must also avoid filesystem mutations during planning.
    fn is_dry_run(&self) -> bool {
        false
    }

    /// Requested interruption exit code, or zero while work may continue.
    fn interruption_code(&self) -> usize {
        0
    }
}

/// Execute real processes, or print commands and return a configured dry-run result.
#[derive(Default)]
pub struct SystemRunner {
    dry_run_output: Option<Output>,
    interruption: Option<Arc<AtomicUsize>>,
}

impl SystemRunner {
    /// Dry-run commands return success with empty output unless overridden.
    pub fn dry_run() -> Self {
        Self {
            dry_run_output: Some(Output::default()),
            interruption: None,
        }
    }

    /// Select the fake result returned for commands suppressed during a dry run.
    pub fn with_dry_run_output(output: Output) -> Self {
        Self {
            dry_run_output: Some(output),
            interruption: None,
        }
    }

    /// Stop starting work, kill and reap the active child when this flag is set
    /// to an exit code (typically 130/143). Cleanup commands ignore the flag.
    pub fn with_interruption(mut self, flag: Arc<AtomicUsize>) -> Self {
        self.interruption = Some(flag);
        self
    }
}

impl Runner for SystemRunner {
    fn execute(&self, command: &Command, capture: bool) -> io::Result<Output> {
        if let Some(output) = &self.dry_run_output
            && !command.allow_in_dry_run
        {
            eprintln!("[dry-run] {command}");
            return Ok(output.clone());
        }
        let interrupted = || {
            if command.during_cleanup {
                0
            } else {
                self.interruption
                    .as_ref()
                    .map_or(0, |flag| flag.load(Ordering::SeqCst))
            }
        };
        if interrupted() != 0 {
            return Ok(Output {
                status: Status::from_code(interrupted() as i32),
                ..Output::default()
            });
        }
        let mut child = process::Command::new(command.program());
        child.args(command.arguments());
        if let Some(directory) = command.directory() {
            child.current_dir(directory);
        }
        for (key, value) in command.environment() {
            if let Some(value) = value {
                child.env(key, value);
            } else {
                child.env_remove(key);
            }
        }
        if capture {
            child
                .stdout(process::Stdio::piped())
                .stderr(process::Stdio::piped());
        }
        if command.input.is_some() {
            child.stdin(process::Stdio::piped());
        }
        // A tool can spawn helpers which inherit its output pipes. Cancellation
        // must stop the entire group before joining the pipe readers; killing
        // only the immediate child can otherwise block release cleanup forever.
        #[cfg(unix)]
        if self.interruption.is_some() && !command.during_cleanup {
            use std::os::unix::process::CommandExt;
            child.process_group(0);
        }
        let mut child = child.spawn()?;
        // Drain both pipes and feed stdin concurrently, avoiding pipe-capacity
        // deadlocks even when a tool writes before reading its input.
        thread::scope(|scope| {
            let stdout = child.stdout.take().map(|mut pipe| {
                scope.spawn(move || {
                    let mut bytes = Vec::new();
                    pipe.read_to_end(&mut bytes).map(|_| bytes)
                })
            });
            let stderr = child.stderr.take().map(|mut pipe| {
                scope.spawn(move || {
                    let mut bytes = Vec::new();
                    pipe.read_to_end(&mut bytes).map(|_| bytes)
                })
            });
            let stdin = child
                .stdin
                .take()
                .zip(command.input())
                .map(|(mut pipe, bytes)| scope.spawn(move || pipe.write_all(bytes)));
            let mut exited = None;
            let status = loop {
                if interrupted() != 0 {
                    #[cfg(unix)]
                    {
                        use rustix::process::{Pid, Signal, kill_process_group};
                        let _ = kill_process_group(Pid::from_child(&child), Signal::KILL);
                    }
                    let _ = child.kill();
                    child.wait()?;
                    break Status::from_code(interrupted() as i32);
                }
                if exited.is_none() {
                    exited = child.try_wait()?;
                }
                // Continue observing cancellation even after the direct child
                // exits if descendants still hold a captured pipe open.
                if let Some(status) = exited
                    && stdout.as_ref().is_none_or(|h| h.is_finished())
                    && stderr.as_ref().is_none_or(|h| h.is_finished())
                    && stdin.as_ref().is_none_or(|h| h.is_finished())
                {
                    break status.into();
                }
                thread::sleep(Duration::from_millis(20));
            };
            let join_error = || io::Error::other("command pipe worker panicked");
            let stdout = stdout
                .map(|h| h.join().map_err(|_| join_error())?)
                .transpose()?
                .unwrap_or_default();
            let stderr = stderr
                .map(|h| h.join().map_err(|_| join_error())?)
                .transpose()?
                .unwrap_or_default();
            if let Some(input) = stdin {
                let result = input.join().map_err(|_| join_error())?;
                if status.success() {
                    result?;
                }
            }
            Ok(Output {
                status,
                stdout,
                stderr,
            })
        })
    }

    fn is_dry_run(&self) -> bool {
        self.dry_run_output.is_some()
    }

    fn interruption_code(&self) -> usize {
        self.interruption
            .as_ref()
            .map_or(0, |flag| flag.load(Ordering::SeqCst))
    }
}

/// Sanitized command failures. Raw secret values are never retained in these errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The process could not be started or waited on.
    #[error("cannot run {command}: {detail}")]
    Spawn {
        /// Redacted command line.
        command: String,
        /// Redacted operating-system error.
        detail: String,
    },
    /// The child returned a failing status.
    #[error("{command}: {status}{stderr}")]
    Failed {
        /// Redacted command line.
        command: String,
        /// Child's exit status.
        status: Status,
        /// Redacted tail of captured stderr, prefixed with a newline when nonempty.
        stderr: String,
    },
    /// A successful command returned non-UTF-8 stdout to `read`.
    #[error("{command}: {status}; stdout is not UTF-8: {detail}{stderr}")]
    Utf8 {
        /// Redacted command line.
        command: String,
        /// Decoding error without raw output bytes.
        detail: String,
        /// Child's successful exit status.
        status: Status,
        /// Redacted tail of captured stderr.
        stderr: String,
    },
}

fn stderr_tail(stderr: &str) -> String {
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return String::new();
    }
    let lines: Vec<_> = stderr.lines().rev().take(20).collect();
    let tail = lines.into_iter().rev().collect::<Vec<_>>().join("\n");
    let start = tail.ceil_char_boundary(tail.len().saturating_sub(4096));
    format!("\nstderr (tail):\n{}", &tail[start..])
}

#[cfg(test)]
mod tests;
