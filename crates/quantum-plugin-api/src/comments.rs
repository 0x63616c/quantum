//! Portable discussion behavior. Core retains comment storage and transaction history.
use crate::{Anchor, Command, Comment, Pos, Region, Snapshot, ToolError, Uuid};
use std::collections::BTreeMap;

/// Registered discussion panel shared by shell and MCP adapters.
pub const COMMENTS_PANEL_ID: &str = "dev.quantum.comments.panel";

/// Transient discussion state retained by the shell, never saved in a project.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommentsState {
    /// Selected thread.
    pub selected: Option<Uuid>,
    /// Stable presentation numbers, including threads seen in history previews.
    pub numbers: BTreeMap<Uuid, usize>,
    /// Whether the next viewport click places an anchor.
    pub placing: bool,
    /// Whether placement uses two corners.
    pub region: bool,
    /// First region corner.
    pub first: Option<Pos>,
    /// Unposted anchor.
    pub draft: Option<Anchor>,
    /// Open (0), resolved (1), or all (2) threads.
    pub filter: usize,
}

/// Read-only capabilities for the discussion panel and agent requests.
pub trait CommentsCx {
    /// Displayed content, including history previews.
    fn snapshot(&self) -> &Snapshot;
    /// Live content used to preserve numbering across history previews.
    fn live_snapshot(&self) -> &Snapshot {
        self.snapshot()
    }
    /// Last successfully admitted presentation state.
    fn state(&self) -> &CommentsState;
    /// Whether the displayed content is read-only history.
    fn preview(&self) -> bool {
        false
    }
}

/// Presentation data supplied by the enabled panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommentsView {
    /// State with newly encountered threads numbered.
    pub state: CommentsState,
    /// Threads in presentation order, filtered for the list.
    pub comments: Vec<Comment>,
    /// Whether the document contains any threads, regardless of filter.
    pub empty: bool,
    /// Selected thread and its original document index.
    pub thread: Option<(usize, Comment)>,
    /// Selected or draft anchor outline.
    pub region: Option<Region>,
}

/// Layer association for a newly created discussion.
#[derive(Clone, Copy, Debug)]
pub enum CommentLayer {
    /// Human voxel anchors use the visible voxel's layer; regions have no layer.
    Automatic,
    /// Agent requests retain their explicitly supplied layer, including absence.
    Explicit(Option<Uuid>),
}

/// Typed requests from the shell and MCP adapter.
#[derive(Clone, Debug)]
pub enum CommentsAction {
    /// Execute a declared shell command.
    Command(String),
    /// Resolve a coordinate supplied by the registered placement tool.
    Anchor(Pos),
    /// Select a thread or return to the list.
    Select(Option<Uuid>),
    /// Cancel placement and selection.
    Cancel,
    /// Toggle voxel/region placement.
    ToggleRegion,
    /// Change the list and pin filter.
    Filter(usize),
    /// Post the trimmed shell text to the draft or selected thread.
    Post(String),
    /// Create an attributed thread through a typed core command.
    Create {
        /// Thread identity.
        id: Uuid,
        /// Spatial anchor.
        anchor: Anchor,
        /// Layer association policy.
        layer: CommentLayer,
        /// Initial message identity.
        message: Uuid,
        /// Plain text, validated by core on admission.
        text: String,
    },
    /// Append an attributed message.
    Reply {
        /// Thread identity.
        id: Uuid,
        /// Message identity.
        message: Uuid,
        /// Plain text.
        text: String,
    },
    /// Move an existing anchor.
    Reanchor {
        /// Thread identity.
        id: Uuid,
        /// Replacement anchor.
        anchor: Anchor,
        /// Replacement layer association.
        layer: Option<Uuid>,
    },
    /// Resolve or reopen a thread.
    Resolve {
        /// Thread identity.
        id: Uuid,
        /// Desired state.
        resolved: bool,
    },
}

/// Proposed state and edits, adopted only after core admits the complete transaction.
#[derive(Clone, Debug, Default)]
pub struct CommentsOutcome {
    /// Next presentation state.
    pub state: CommentsState,
    /// Typed core commands; no engine or mutation capability crosses the boundary.
    pub commands: Vec<Command>,
    /// Existing human history label.
    pub label: String,
    /// Optional shell status.
    pub status: Option<String>,
    /// Focus the registered panel.
    pub open_panel: bool,
    /// Clear the native text field after successful admission.
    pub clear_text: bool,
    /// Scroll the discussion to its newest message.
    pub scroll_bottom: bool,
}

/// Portable behavior bound to a declared discussion panel.
pub trait CommentsPanel {
    /// Produce list, thread and outline data without GUI access.
    fn view(&self, cx: &dyn CommentsCx) -> CommentsView;
    /// Propose presentation changes and typed core edits.
    fn interact(
        &self,
        cx: &dyn CommentsCx,
        action: CommentsAction,
    ) -> Result<CommentsOutcome, ToolError>;
    /// List agent-visible threads with the existing pagination wire shape.
    fn list(&self, cx: &dyn CommentsCx, query: CommentsQuery) -> crate::Value;
}

/// Existing MCP pagination and filtering, validated by the transport schema.
#[derive(Clone, Debug)]
pub struct CommentsQuery {
    /// All, open or resolved threads.
    pub state: String,
    /// Thread offset.
    pub offset: usize,
    /// Maximum threads.
    pub limit: usize,
    /// Per-thread message offset.
    pub message_offset: usize,
    /// Maximum messages per thread.
    pub message_limit: usize,
}
