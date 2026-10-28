use regex::Regex;

use crate::{Error, Report};

/// Reusable redaction policy, compiled before installing a panic hook.
/// Only the dedicated contact field preserves emails, when explicitly supplied.
#[derive(Clone, Debug)]
pub struct Redactor {
    home: Option<String>,
    paths: Regex,
    emails: Regex,
}

impl Redactor {
    /// Configure the actual home directory if known. Empty/root values are ignored.
    /// No environment variables are read. Standard macOS/Linux home paths are
    /// always covered, even when the actual home directory is unknown.
    pub fn new(home: Option<&str>) -> Result<Self, regex::Error> {
        Ok(Self {
            home: home
                .map(|s| s.trim_end_matches('/'))
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
            paths: Regex::new(r"/(?:Users|home)/[^/\s\\:<>\x22'()][^/\r\n\\:<>\x22'()]*")?,
            emails: Regex::new(
                r#"(?:"[^"\r\n]+"|[\p{L}\p{N}.!#$%&'*+/=?^_`{|}~\-]+)@(?:\[[^\]\r\n]+\]|[\p{L}\p{N}](?:[\p{L}\p{N}.\-]*[\p{L}\p{N}])?)"#,
            )?,
        })
    }

    /// Redact home prefixes and email-like text conservatively, including Unicode.
    pub fn redact(&self, text: &str) -> String {
        self.emails
            .replace_all(&self.redact_paths(text), "[redacted email]")
            .into_owned()
    }

    fn redact_paths(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        if let Some(home) = &self.home {
            let mut rest = text;
            while let Some(index) = rest.find(home) {
                let (prefix, tail) = rest.split_at(index);
                result.push_str(prefix);
                let suffix = &tail[home.len()..];
                if suffix
                    .chars()
                    .next()
                    .is_none_or(|c| c == '/' || !c.is_alphanumeric() && c != '_' && c != '-')
                {
                    result.push('~');
                } else {
                    result.push_str(home);
                }
                rest = suffix;
            }
            result.push_str(rest);
        } else {
            result.push_str(text);
        }
        self.paths.replace_all(&result, "~").into_owned()
    }

    /// Sanitize every string in a report, preserving only explicit contact consent.
    pub fn report(&self, report: &Report) -> Result<Report, Error> {
        let contact = match report {
            Report::Crash(r) => r.contact_email.clone(),
            Report::Feedback(r) => r.contact_email.clone(),
        };
        let mut value = serde_json::to_value(report)?;
        self.value(&mut value);
        let mut report: Report = serde_json::from_value(value)?;
        // A contact field is not an escape hatch for arbitrary unredacted text.
        let contact = contact.map(|s| {
            if self
                .emails
                .find(&s)
                .is_some_and(|m| m.start() == 0 && m.end() == s.len())
            {
                self.redact_paths(&s)
            } else {
                self.redact(&s)
            }
        });
        match &mut report {
            Report::Crash(r) => r.contact_email = contact,
            Report::Feedback(r) => r.contact_email = contact,
        }
        Ok(report)
    }

    fn value(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(s) => *s = self.redact(s),
            serde_json::Value::Array(values) => values.iter_mut().for_each(|v| self.value(v)),
            serde_json::Value::Object(values) => values.values_mut().for_each(|v| self.value(v)),
            _ => {}
        }
    }
}
