//! Wire adapters for the enabled portable discussion panel.
use crate::error::{Failure, Result};
use quantum_plugin_api::COMMENTS_PANEL_ID as PANEL_ID;
use quantum_plugin_api::{
    Command, CommentsAction, CommentsCx, CommentsQuery, CommentsState, Snapshot, Value,
};
use quantum_plugin_host::{Host, HostError};

pub(crate) fn enabled(host: &Host) -> bool {
    host.registry().panels().any(|(_, p)| p.id == PANEL_ID)
}
fn unavailable() -> Failure {
    Failure::new("comments_unavailable", "Comments plugin is unavailable")
}
fn failure(error: HostError) -> Failure {
    match error {
        HostError::Tool(quantum_plugin_api::ToolError::Core(error)) => error.into(),
        error => Failure::new("comments_failed", error.to_string()),
    }
}
pub(crate) fn command(
    host: &mut Host,
    snapshot: &Snapshot,
    action: CommentsAction,
) -> Result<Command> {
    let out = host
        .dispatch_comments(
            PANEL_ID,
            &DocumentComments {
                snapshot,
                state: CommentsState::default(),
            },
            action,
        )
        .map_err(failure)?
        .ok_or_else(unavailable)?;
    // Keep the existing single-command history shape, including explicit batch callers.
    match out.commands.as_slice() {
        [command] => Ok(command.clone()),
        _ => Ok(Command::Batch(out.commands)),
    }
}
pub(crate) fn list(host: &mut Host, snapshot: &Snapshot, query: CommentsQuery) -> Result<Value> {
    host.list_comments(
        PANEL_ID,
        &DocumentComments {
            snapshot,
            state: CommentsState::default(),
        },
        query,
    )
    .map_err(failure)?
    .ok_or_else(unavailable)
}
struct DocumentComments<'a> {
    snapshot: &'a Snapshot,
    state: CommentsState,
}
impl CommentsCx for DocumentComments<'_> {
    fn snapshot(&self) -> &Snapshot {
        self.snapshot
    }
    fn state(&self) -> &CommentsState {
        &self.state
    }
}
