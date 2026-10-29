//! Explicit, single-attempt relay transport. No credentials or default destination.
use quantum_crash::{RelayRequest, RelayTransport};
use std::io::{Read, Write};

/// Read the endpoint once when the confirmation window opens. An empty setting
/// keeps the browser fallback. Invalid nonempty settings fail during preparation.
pub fn endpoint() -> Option<String> {
    std::env::var("QUANTUM_REPORT_ENDPOINT")
        .ok()
        .or_else(|| option_env!("QUANTUM_REPORT_ENDPOINT").map(str::to_owned))
        .filter(|endpoint| !endpoint.is_empty())
}

/// System curl transport (bundled with macOS). Ignores curlrc, disables redirects
/// and retries, and bounds connection/total time, upload and response sizes.
pub struct Transport;

impl RelayTransport for Transport {
    type Error = std::io::Error;

    fn post(&self, request: &RelayRequest) -> Result<u16, Self::Error> {
        if request.json_body.len() > 96 * 1024 {
            return Err(std::io::Error::other("report exceeds relay limit"));
        }
        let mut input = tempfile::NamedTempFile::new()?;
        input.write_all(request.json_body.as_bytes())?;
        let mut command = std::process::Command::new("curl");
        command.args([
            "--disable",
            "--silent",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-redirs",
            "0",
            "--retry",
            "0",
            "--connect-timeout",
            "5",
            "--max-time",
            "15",
            "--max-filesize",
            "4096",
            "--request",
            "POST",
            "--write-out",
            "\n%{http_code}",
        ]);
        command
            .arg("--data-binary")
            .arg(format!("@{}", input.path().display()));
        for (key, value) in &request.headers {
            command.arg("--header").arg(format!("{key}: {value}"));
        }
        let child = command
            .arg("--url")
            .arg(&request.url)
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        read_status(child)
    }
}

fn read_status(mut child: std::process::Child) -> Result<u16, std::io::Error> {
    let mut bytes = Vec::new();
    let read = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("missing relay response"))
        .and_then(|stdout| stdout.take(4101).read_to_end(&mut bytes));
    // Enforce the cap ourselves, including on older system curl versions that
    // only apply --max-filesize when the response advertises Content-Length.
    if read.is_err() || bytes.len() > 4100 {
        let _ = child.kill();
        let _ = child.wait();
        return Err(std::io::Error::other("relay response exceeds limit"));
    }
    let result = child.wait()?;
    let suffix = bytes
        .len()
        .checked_sub(4)
        .and_then(|start| bytes.get(start..));
    let Some([b'\n', a, b, c]) = suffix else {
        return Err(std::io::Error::other("invalid relay response"));
    };
    if !result.success() {
        return Err(std::io::Error::other("relay acceptance unconfirmed"));
    }
    let status = std::str::from_utf8(&[*a, *b, *c])
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| std::io::Error::other("invalid relay status"))?;
    // This service confirms creation with exactly 201, never queued/202.
    Ok(if status == 201 {
        201
    } else if (200..300).contains(&status) {
        502
    } else {
        status
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn response(script: &str) -> Result<u16, std::io::Error> {
        read_status(
            std::process::Command::new("sh")
                .args(["-c", script])
                .stdout(std::process::Stdio::piped())
                .spawn()?,
        )
    }

    #[test]
    fn accepts_only_confirmed_creation_and_bounds_process_output() {
        assert_eq!(
            response("printf '{\"code\":\"created\"}\\n201'").unwrap(),
            201
        );
        assert_eq!(response("printf '\\n202'").unwrap(), 502);
        assert_eq!(response("printf '\\n302'").unwrap(), 302);
        assert!(response("printf '\\n201'; exit 1").is_err());
        assert!(response("printf 'invalid'").is_err());
        assert!(response("printf '%8192s' x").is_err());
    }

    #[test]
    fn oversized_report_fails_before_starting_transport() {
        let request = RelayRequest {
            url: "https://unused.invalid".into(),
            headers: vec![],
            json_body: "x".repeat(96 * 1024 + 1),
        };
        assert!(Transport.post(&request).is_err());
    }
}
