//! Capability-scoped descriptions of renderer-owned passes; no GPU resources.
use crate::{RenderPassSlot, SettingValue};

pub use quantum_render_plan::RenderKernel;

/// Read-only owner settings and a staged, slot-scoped kernel selection sink.
/// No document mutations, renderer handles or unrestricted services are exposed.
pub trait PassCx {
    /// Slot from this pass's admitted declaration.
    fn slot(&self) -> RenderPassSlot;
    /// Only settings declared by this pass's owner are visible.
    fn setting(&self, key: &str) -> Option<&SettingValue>;
    /// Select an implementation by its typed ID. Unsupported slots return false.
    /// Selection is staged until the callback succeeds; panics discard it.
    fn select(&mut self, kernel: RenderKernel) -> bool;
}

/// Portable behavior bound to a declared [`crate::RenderPassContribution`].
pub trait RenderPass {
    /// Describe a pass for this frame using only the capabilities granted here.
    fn describe(&self, cx: &mut dyn PassCx);
}
