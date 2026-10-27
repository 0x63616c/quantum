//! Portable registration, lifecycle and diagnostics for statically linked plugins.
//!
//! The host publishes only enabled plugins' validated declarations. Conflicting
//! plugins stay disabled until explicitly enabled after the conflict is resolved.
//! Panicking hooks quarantine a plugin for the lifetime of the host; they are not
//! retried. Isolation requires unwinding panics (not `panic = "abort"`).
//! No app, GUI, dynamic loading or implementation binding is provided here.

pub mod conformance;
mod host;
mod registry;
mod timings;

pub use host::{
    EnabledSet, Host, HostError, HostEvent, ManifestError, Phase, PluginState, RegistrationError,
    api_compatible, validate_manifest,
};
pub use registry::{Conflict, ContributionOwner, Registry};
pub use timings::{BudgetExceeded, Timing, Timings};

#[cfg(test)]
mod tests;
