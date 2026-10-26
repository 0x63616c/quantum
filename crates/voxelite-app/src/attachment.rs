//! Owner-thread boundary between queued agent work and editor reconciliation.
use crate::editor::Editor;
use voxelite_mcp::attached::{AttachedSession, SessionEvent};

/// Completed agent activity after reconciling the live editor state.
pub struct AttachmentUpdate {
    /// Connected transports sampled before processing this tick's requests.
    pub clients: usize,
    /// Completed events in attachment order, including finished background renders.
    pub events: Vec<SessionEvent>,
    /// Whether queued commands advanced the live authority.
    pub changed: bool,
}
impl Editor {
    /// Drain one bounded agent tick and reconcile selection and playback together.
    /// The shell calls this only when no dialog or pending gesture blocks polling.
    pub fn poll_attachment(&mut self, attachment: &mut AttachedSession) -> AttachmentUpdate {
        let revision = self.session.engine.revision();
        let clients = attachment.clients();
        let events = attachment.poll(&mut self.session);
        let changed = self.session.engine.revision() != revision;
        if changed {
            self.agent_updated();
        }
        AttachmentUpdate {
            clients,
            events,
            changed,
        }
    }
}
