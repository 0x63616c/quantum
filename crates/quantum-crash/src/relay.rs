use serde::{Deserialize, Serialize};
use url::Url;

use crate::{Error, FeedbackKind};

/// Versioned relay protocol. Contains the public issue and optional private contact, never scene bytes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayPayload {
    /// Currently 1. Relays must reject unsupported versions.
    pub schema_version: u32,
    /// Redacted issue title.
    pub title: String,
    /// Redacted Markdown preview.
    pub body: String,
    /// Suggested GitHub labels; the relay must enforce its own allowlist.
    pub labels: Vec<String>,
    /// Private opt-in contact. The relay must store/forward this privately and
    /// never publish it in the GitHub issue. Omitted without contact consent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,
    /// Feedback classification.
    pub kind: FeedbackKind,
}

impl RelayPayload {
    /// Build a prefilled GitHub URL. Returns an error instead of silently truncating
    /// the preview. The app can offer copy/save or relay when the URL is too long.
    /// The private contact field is always excluded from this browser fallback.
    pub fn github_url(&self, max_bytes: usize) -> Result<String, Error> {
        let mut url = Url::parse("https://github.com/0x63616c/quantum/issues/new")
            .map_err(|_| Error::Configuration)?;
        url.query_pairs_mut()
            .append_pair(
                "template",
                match self.kind {
                    FeedbackKind::Bug => "bug.yml",
                    FeedbackKind::Feature => "feature.yml",
                },
            )
            .append_pair("title", &self.title)
            .append_pair("body", &self.body)
            .append_pair("labels", &self.labels.join(","));
        let url: String = url.into();
        if url.len() > max_bytes {
            return Err(Error::UrlTooLong);
        }
        Ok(url)
    }

    /// Prepare an HTTPS POST to an app-configured relay. No production endpoint,
    /// authorization token, environment data or automatic transport is provided.
    pub fn request(&self, endpoint: &str, version: &str) -> Result<RelayRequest, Error> {
        let url = Url::parse(endpoint).map_err(|_| Error::Configuration)?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.query().is_some()
            || version.is_empty()
            || !version
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b".-+".contains(&c))
        {
            return Err(Error::Configuration);
        }
        Ok(RelayRequest {
            url: url.into(),
            headers: vec![
                ("Content-Type".into(), "application/json".into()),
                ("User-Agent".into(), format!("Quantum/{version}")),
            ],
            json_body: serde_json::to_string(self)?,
        })
    }
}

/// Prepared POST request. Show the payload preview before explicitly calling send.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayRequest {
    /// Validated HTTPS URL; configured by the app.
    pub url: String,
    /// Generic content type and product/version User-Agent. No credentials.
    pub headers: Vec<(String, String)>,
    /// Serialized versioned payload.
    pub json_body: String,
}

/// App-supplied transport. Implementations must POST once, disable redirects
/// (to avoid forwarding diagnostic data), and bound timeouts/response sizes.
/// The relay owns GitHub authentication, abuse controls and issue creation.
pub trait RelayTransport {
    /// Transport-specific failure.
    type Error;
    /// POST the prepared request, returning an HTTP status. A 2xx status must
    /// mean the relay accepted the report; failures must never delete local data.
    fn post(&self, request: &RelayRequest) -> Result<u16, Self::Error>;
}

/// Sending failure; deliberately excludes response bodies that might contain PII.
#[derive(Debug, thiserror::Error)]
pub enum SendError<E> {
    /// Transport could not complete the request.
    #[error("relay transport failed")]
    Transport(E),
    /// Relay did not accept the request.
    #[error("relay returned HTTP {0}")]
    Status(u16),
}

impl RelayRequest {
    /// Explicitly send this exact request after the app has obtained confirmation.
    /// Never retries and never marks any report sent or removes local files.
    pub fn send<T: RelayTransport>(&self, transport: &T) -> Result<(), SendError<T::Error>> {
        let status = transport.post(self).map_err(SendError::Transport)?;
        if !(200..300).contains(&status) {
            return Err(SendError::Status(status));
        }
        Ok(())
    }
}
