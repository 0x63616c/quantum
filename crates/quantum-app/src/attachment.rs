//! Owner-thread boundary between queued agent work and editor reconciliation.
use crate::editor::Editor;
use quantum_mcp::attached::{AttachedSession, SessionEvent};
#[cfg(not(test))]
use std::env::home_dir;
use std::{
    io,
    path::{Path, PathBuf},
};
#[cfg(test)]
use tests::home_dir;

fn file_root(project: Option<&Path>, home: Option<&Path>) -> io::Result<PathBuf> {
    let root = match project {
        Some(path) => path.parent().map(Path::to_path_buf),
        None => home.map(|home| home.join("Documents/Quantum")),
    };
    root.filter(|root| root.is_absolute()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "MCP requires an absolute project path or home directory",
        )
    })
}

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
    /// File sandbox for the open project, or ~/Documents/Quantum when untitled.
    /// Only resolves the path; the attachment creates the directory if missing.
    pub fn attachment_root(&self) -> io::Result<PathBuf> {
        file_root(self.path.as_deref(), home_dir().as_deref())
    }

    /// Drain one bounded agent tick and reconcile selection and playback together.
    /// The shell calls this only when no dialog or pending gesture blocks polling.
    pub fn poll_attachment(
        &mut self,
        attachment: &mut AttachedSession,
    ) -> io::Result<AttachmentUpdate> {
        // Update before draining requests, including those queued during Save As/Open.
        attachment.set_root(&self.attachment_root()?)?;
        let revision = self.session.engine.revision();
        let clients = attachment.clients();
        let events = attachment.poll_with_host(&mut self.session, self.plugins.get_mut());
        let changed = self.session.engine.revision() != revision;
        if changed {
            self.agent_updated();
        }
        Ok(AttachmentUpdate {
            clients,
            events,
            changed,
        })
    }
}

#[cfg(test)]
mod tests;
