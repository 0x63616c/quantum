use crate::{Command, CoreError, Pos, Region, Swatch, ToolCx, Uuid};

/// The nearest visible cell and face, computed by the host renderer's picking path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolHit {
    /// Occupied cell.
    pub at: Pos,
    /// Outward face normal, exactly one signed axis (zero when starting inside).
    pub normal: Pos,
    /// Topmost visible layer at this cell.
    pub layer: Uuid,
    /// Original stable palette slot.
    pub slot: u16,
}

/// Layer properties relevant to editing, without access to document internals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolLayer {
    /// Whether the layer participates in the visible composite.
    pub visible: bool,
    /// Whether edits are blocked.
    pub locked: bool,
}

/// Pointer and keyboard inputs independent of any UI framework.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolGesture {
    /// Resolve a pointer target and its preview without editing.
    Hover,
    /// Outline a previously resolved pointer or keyboard target, without editing.
    Preview(Option<Pos>),
    /// Apply one pointer click using the host's hit and work plane queries.
    Click,
    /// Apply at an explicit coordinate, such as the keyboard cursor.
    ApplyAt(Pos),
}

/// One gesture plus transient state retained by the host between invocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolInput {
    /// Input being handled.
    pub gesture: ToolGesture,
    /// First point of a pending multi-point gesture, cleared by host cancellation.
    pub anchor: Option<Pos>,
}

/// Proposed edit and feedback. Only the host may commit these commands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolOutcome {
    /// Ordered commands to apply in one atomic engine batch; empty means no edit.
    pub commands: Vec<Command>,
    /// Human-facing transaction label when commands are present.
    pub label: String,
    /// Optional half-open viewport outline for the returned gesture state.
    pub preview: Option<Region>,
    /// Original palette color to select without changing the document.
    pub sampled_color: Option<Swatch>,
    /// Resolved pointer or keyboard target.
    pub target: Option<Pos>,
    /// Next transient first point; never persisted or placed in undo history.
    pub anchor: Option<Pos>,
}

/// A gesture failed before producing an edit.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ToolError {
    /// A host document query failed.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// User-facing explanation of an invalid gesture.
    #[error("{0}")]
    Gesture(String),
}

/// Editing behavior available only through the host's enabled tool registry.
///
/// Implementations only query capabilities and return proposed commands. Undo,
/// transaction admission, mirroring and preview rendering remain host concerns.
/// Proposed voxel commands must leave their mirror settings disabled.
pub trait Tool {
    /// Resolve one input without mutating the document or UI.
    fn interact(&self, cx: &dyn ToolCx, input: ToolInput) -> Result<ToolOutcome, ToolError>;
}
