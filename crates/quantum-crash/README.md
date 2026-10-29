# Quantum crash and feedback reporting core

`quantum-crash` is independent of all other workspace crates, GPUI and HTTP
clients. It has no analytics, telemetry, crash SaaS, GitHub credentials or default
relay endpoint. Nothing is sent automatically. The app must show the complete
preview and obtain explicit confirmation before calling `RelayRequest::send`.
Merely creating a report, installing a hook, rendering, listing pending reports,
or preparing a URL/request does not contact any service or open a browser.

## Data and privacy

A crash report contains caller-supplied app version, OS name/version and CPU
architecture; panic message and source location; `std::backtrace` text; UTC Unix
epoch seconds; a caller-supplied live log tail; and an optional user note. The hook
records no email, user note or scene consent. It keeps the last configured N log
lines (at most 100), bounds each line and panic/backtrace text to 16,384 characters,
and captures a backtrace even when `RUST_BACKTRACE` is unset. It does not read log
files or collect environment variables, hostnames, device identifiers or documents.
The app retains a clone of `LogTail` and pushes lines during normal operation;
the hook snapshots the latest lines and redacts them at capture time.

Feedback contains caller-supplied version/OS/architecture, a bug/feature kind and
what happened. Either report may carry an optional contact email: setting that
field is explicit consent to send it privately to maintainers through the relay.
It is never included in the public issue title/body or browser fallback URL.
The relay must never publish `contact_email`; it may discard it under its documented
policy. The prepared Cloudflare relay discards contact immediately and the app
offers no contact field. See [the deployment runbook](../../packaging/feedback-relay/README.md).
Emails in all other fields remain redacted regardless of contact consent. Malformed contact
text is redacted normally. A crash report's scene flag records consent only; this
crate never reads, embeds or uploads scene contents, even when the flag is true.

Construct a `Redactor` with the actual home directory, if known. Every string in
reports is redacted on local write/load and Markdown/payload preparation. Home
prefixes become `~`; standard `/Users/…` and `/home/…` prefixes are covered even
without a configured home. Email-like strings (including Unicode) become
`[redacted email]`. Test identifiers and addresses are synthetic privacy markers.
The policy is conservative, not a general secret detector: project names, arbitrary
paths outside home directories, tokens and other sensitive prose may remain.
The app must let users inspect/edit the preview and omit sensitive logs/notes.
Do not directly serialize raw input models for transmission; use `Report::payload`.
Raw models remain editable so the app can implement review and consent.

## Storage and panic capture

`install_panic_hook(HookConfig)` is explicitly installed once by the shell during
normal startup. It precompiles no regexes in the hook, runs no caller callbacks,
uses `try_lock` on the log tail (skipping contended or poisoned buffers), and
chains the previous hook even after capture I/O failure.
It writes redacted JSON using exclusive temporary creation, sync, then atomic
rename; temporary files are ignored by `list_pending`. Unix report files use mode
0600. The app must supply a private crash directory, manage retention and surface
setup errors. `load` and `write` enforce a 4 MiB serialized report limit.
`list_pending` returns stable filename order; `dismiss` and `mark_sent` explicitly
delete reports and tolerate files already removed. Sending never deletes reports.
The crash directory is trusted app-owned storage; these APIs are not a filesystem
sandbox. The rename prevents partial JSON visibility, not a guarantee of survival
across sudden power loss.

Capture is best effort: allocations and backtrace symbolization are unavoidable.
Out-of-memory aborts, stack overflow, aborting panics and a panicking previous hook
cannot be recovered. Fatal signal support is follow-up work, not implemented here.
No panic hook is installed simply by linking this library.

## Preview, relay and browser fallback

`Report::markdown` and `Report::payload` share the exact sectioned Markdown body,
including a “What is sent” section. User text is blockquoted and HTML escaped.
`Report::payload(title, redactor)` also redacts the title. The serializable
`RelayPayload` protocol has `schema_version: 1`, `title`, `body`, `labels`, `kind`
and an optional private `contact_email` field (omitted without consent).
Labels are `type:fix` or `type:feat`; kind is `bug` or `feature`.

After preview and confirmation, the app can prepare a request with
`payload.request(configured_endpoint, app_version)` and explicitly call
`request.send(&transport)`. The endpoint must be HTTPS without credentials, query
or fragment. Headers are `Content-Type: application/json` and the generic
`User-Agent: Quantum/<version>`. There is no authorization header or token.
`RelayTransport` is supplied by the app; it must disable redirects, apply timeouts
and bound response sizes. The relay holds GitHub credentials, validates schema and
labels, handles abuse controls, and files the public issue. Only 2xx responses
count as acceptance; transport failures and other statuses return typed errors.
There are no automatic retries or deletion. Relay deployment is outside this crate.

`payload.github_url(max_bytes)` builds the repository's prefilled issue URL with
`bug.yml` or `feature.yml`, title, body and labels using URL query encoding. The
cap applies to the full encoded URL in bytes. Oversized reports return
`Error::UrlTooLong` rather than silently removing previewed content; the app can
offer relay submission or copying/saving the report. This function does not open
the browser. Browser submission also requires confirmation and a GitHub account.

Startup installation, next-launch preview/confirmation dialogs, Help menu feedback
and native metadata/log capture live in `quantum-app`. Scene attachments remain
unavailable. Relay deployment and release activation require the owner steps in
[the runbook](../../packaging/feedback-relay/README.md).
