//! Convert errors only at presentation boundaries; domain errors remain typed.
use std::error::Error;

/// Build a terminal diagnostic retaining every message in the source chain.
/// This accepts borrowed errors so recoverable UI failures remain available.
pub fn report(error: &dyn Error) -> miette::Report {
    let mut messages = vec![error.to_string()];
    let mut source = error.source();
    while let Some(error) = source {
        messages.push(error.to_string());
        source = error.source();
    }
    let mut messages = messages.into_iter().rev();
    let mut report = miette::Report::msg(messages.next().unwrap_or_default());
    for message in messages {
        report = report.wrap_err(message);
    }
    report
}
