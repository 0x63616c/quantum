use crate::{Command, TransformCx};

/// A read-only rewrite of the pending commands from a tool outcome.
///
/// The host runs enabled transforms in ascending declared order, breaking ties
/// by registration order. Results feed the next transform before the host commits
/// one atomic batch through the normal transaction path. Empty input is allowed;
/// implementations should preserve it so previews and no-op gestures stay inert.
pub trait CommandTransform {
    /// Return transformed commands without mutating the document or settings.
    fn transform(&self, cx: &dyn TransformCx, commands: Vec<Command>) -> Vec<Command>;
}
