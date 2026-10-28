//! Signing and notarization helpers. Secret-bearing tools always use captured,
//! suppressed diagnostics, including failures from certificate discovery.
use super::{Context, files, invalid};
use crate::{Error, Result, cmd::Command};
use std::path::{Path, PathBuf};

/// Temporary keychain ownership. Drop deletes it and restores the original user
/// search list on errors and panics. Call `finish` to report cleanup failures.
pub struct Keychain<'a, 'b> {
    context: &'a Context<'b>,
    path: PathBuf,
    original: Vec<String>,
    active: bool,
}
impl<'a, 'b> Keychain<'a, 'b> {
    /// Snapshot the search list before any keychain mutation. An empty list stays empty.
    pub fn prepare(context: &'a Context<'b>, directory: &Path) -> Result<Self> {
        let output = context.command("security").args(["list-keychains", "-d", "user"])
            .read(context.runner).map_err(|_| invalid("Cannot read the user keychain search list; raw signing diagnostics suppressed."))?;
        let original = keychain_paths(&output)?;
        Ok(Self {
            context,
            path: directory.join("signing.keychain-db"),
            original,
            active: true,
        })
    }
    /// Create and import a certificate, then select exactly one Developer ID identity.
    pub fn import(&self, certificate: &Path) -> Result<String> {
        let context = self.context;
        let password = context.command("openssl").args(["rand", "-hex", "32"])
            .read(context.runner).map_err(|_| invalid("Cannot generate temporary keychain password; raw signing diagnostics suppressed."))?;
        let password = if context.runner.is_dry_run() {
            "DRY-RUN-PASSWORD"
        } else {
            password.as_str()
        };
        context.private(
            "Keychain creation",
            context
                .command("security")
                .args(["create-keychain", "-p"])
                .secret_arg(password)
                .arg(&self.path),
        )?;
        context.private(
            "Keychain settings",
            context
                .command("security")
                .args(["set-keychain-settings", "-lut", "21600"])
                .arg(&self.path),
        )?;
        context.private(
            "Keychain unlock",
            context
                .command("security")
                .args(["unlock-keychain", "-p"])
                .secret_arg(password)
                .arg(&self.path),
        )?;
        context.private(
            "Certificate import",
            context
                .command("security")
                .arg("import")
                .arg(certificate)
                .arg("-k")
                .arg(&self.path)
                .arg("-P")
                .secret_arg(context.environment.get("MACOS_CERTIFICATE_PASSWORD"))
                .args(["-T", "/usr/bin/codesign", "-T", "/usr/bin/security"]),
        )?;
        context.private(
            "Keychain access configuration",
            context
                .command("security")
                .args([
                    "set-key-partition-list",
                    "-S",
                    "apple-tool:,apple:,codesign:",
                    "-s",
                    "-k",
                ])
                .secret_arg(password)
                .arg(&self.path),
        )?;
        context.private(
            "Keychain search list update",
            context
                .command("security")
                .args(["list-keychains", "-d", "user", "-s"])
                .args(&self.original)
                .arg(&self.path),
        )?;
        if !context.runner.is_dry_run() {
            files::remove(certificate)?;
        }
        context.plan(format_args!("remove {}", certificate.display()));
        let identities = context
            .command("security")
            .args(["find-identity", "-v", "-p", "codesigning"])
            .arg(&self.path)
            .read(context.runner)
            .map_err(|_| {
                invalid("Identity discovery failed; raw signing diagnostics suppressed.")
            })?;
        if context.runner.is_dry_run() {
            return Ok("DRY-RUN-IDENTITY".into());
        }
        select_identity(
            &identities,
            context.environment.get("MACOS_SIGNING_IDENTITY"),
        )
    }
    /// Path passed explicitly to codesign, never the login keychain.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Attempt both cleanup operations even when the first fails.
    pub fn finish(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        self.active = false;
        let delete = self
            .context
            .command("security")
            .arg("delete-keychain")
            .arg(&self.path)
            .during_cleanup()
            .output(self.context.runner);
        let restore = self
            .context
            .command("security")
            .args(["list-keychains", "-d", "user", "-s"])
            .args(&self.original)
            .during_cleanup()
            .output(self.context.runner);
        if delete.is_err() {
            return Err(Error::Cleanup(
                "Failed to delete the temporary signing keychain.".into(),
            ));
        }
        if restore.is_err() {
            return Err(Error::Cleanup(
                "Failed to restore the user keychain search list.".into(),
            ));
        }
        Ok(())
    }
}
impl Drop for Keychain<'_, '_> {
    fn drop(&mut self) {
        if let Err(error) = self.finish() {
            eprintln!("{error}");
        }
    }
}

/// Parse security's shell-quoted search list before mutating it. Like POSIX
/// shlex.split with comments disabled, paths may contain spaces, quotes and '#'.
pub fn keychain_paths(text: &str) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some('\''), '\'') | (Some('"'), '"') => quote = None,
            (Some('\''), c) => word.push(c),
            (Some('"'), '\\') => {
                let next = chars
                    .next()
                    .ok_or_else(|| invalid("Malformed user keychain search list"))?;
                if !matches!(next, '"' | '\\') {
                    word.push('\\');
                }
                word.push(next);
            }
            (None, '\'' | '"') => {
                quote = Some(c);
                started = true;
            }
            (None, '\\') => {
                word.push(
                    chars
                        .next()
                        .ok_or_else(|| invalid("Malformed user keychain search list"))?,
                );
                started = true;
            }
            (None, c) if c.is_ascii_whitespace() => {
                if started {
                    paths.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            (_, c) => {
                word.push(c);
                started = true;
            }
        }
    }
    if quote.is_some() {
        return Err(invalid("Malformed user keychain search list"));
    }
    if started {
        paths.push(word);
    }
    Ok(paths)
}

/// Select exactly one valid Developer ID Application identity by name or digest.
/// Diagnostics never contain certificate names or requested identities.
pub fn select_identity(output: &str, requested: &str) -> Result<String> {
    let identities: Vec<_> = output
        .lines()
        .filter_map(|line| {
            let (before, quoted) = line.split_once('"')?;
            let name = quoted.split_once('"')?.0;
            let digest = before.split_whitespace().last()?;
            (digest.len() == 40
                && digest.bytes().all(|b| b.is_ascii_hexdigit())
                && name.starts_with("Developer ID Application: ")
                && name.len() > "Developer ID Application: ".len()
                && (requested.is_empty()
                    || requested == name
                    || requested.eq_ignore_ascii_case(digest)))
            .then_some(digest)
        })
        .collect();
    if identities.len() != 1 {
        return Err(invalid(
            "Expected one valid Developer ID Application identity in the imported p12; set MACOS_SIGNING_IDENTITY to select one.",
        ));
    }
    Ok(identities[0].into())
}

/// Codesign policy shared by every nested executable and the outer bundle.
pub struct Signing<'a> {
    /// Fingerprint selected from the imported certificate, or '-' for ad-hoc signing.
    pub identity: &'a str,
    /// Temporary keychain for Developer ID signing.
    pub keychain: Option<&'a Path>,
}
impl Signing<'_> {
    /// Inside-out signing arguments; never use --deep when signing.
    pub fn command(&self, context: &Context<'_>) -> Command {
        let mut command = context.command("codesign").args(["--force", "--sign"]);
        if let Some(keychain) = self.keychain {
            command = command
                .secret_arg(self.identity)
                .arg("--keychain")
                .arg(keychain)
                .args(["--options", "runtime", "--timestamp"]);
        } else {
            command = command.arg("-").arg("--timestamp=none");
        }
        command
    }
}

/// Submit an archive, accept only Apple's Accepted result, and suppress raw
/// authentication diagnostics. The private key stays in a 0600 temporary file.
pub fn notarize(context: &Context<'_>, archive: &Path, key: &Path) -> Result<()> {
    let output = context
        .command("xcrun")
        .args(["notarytool", "submit"])
        .arg(archive)
        .arg("--key")
        .arg(key)
        .arg("--key-id")
        .secret_arg(context.environment.get("APPLE_API_KEY_ID"))
        .arg("--issuer")
        .secret_arg(context.environment.get("APPLE_API_ISSUER_ID"))
        .args(["--wait", "--timeout", "30m", "--output-format", "json"])
        .output(context.runner)
        .map_err(|_| {
            invalid("Notarization submission failed; raw authentication diagnostics suppressed.")
        })?;
    if context.runner.is_dry_run() {
        return Ok(());
    }
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|_| {
        invalid("Invalid notarization result; raw authentication diagnostics suppressed.")
    })?;
    if result.get("status").and_then(|s| s.as_str()) != Some("Accepted") {
        // Even an alphanumeric or UUID-shaped value can be an echoed credential.
        // Keep all returned authentication metadata out of release logs.
        return Err(invalid(
            "Notarization was not accepted. Inspect the submission locally with notarytool log.",
        ));
    }
    Ok(())
}
/// Staple and validate the ticket after successful submission.
pub fn staple(context: &Context<'_>, path: &Path) -> Result<()> {
    for operation in ["staple", "validate"] {
        context
            .command("xcrun")
            .args(["stapler", operation])
            .arg(path)
            .run(context.runner)?;
    }
    Ok(())
}
