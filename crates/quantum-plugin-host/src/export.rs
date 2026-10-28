use super::*;
use quantum_plugin_api::{ExportCallback, ExportCx, ExportImage, ExportResult, Snapshot};
use std::time::{Duration, Instant};

/// Read-only adapter for a captured snapshot and host-selected renderer.
pub struct SnapshotExport<'a> {
    /// Immutable document content; no engine or history is supplied.
    pub snapshot: &'a Snapshot,
    /// Render the only view the exporter may request, using the captured snapshot.
    pub render_view: &'a dyn Fn(&Snapshot) -> ExportResult<ExportImage>,
}

impl ExportCx for SnapshotExport<'_> {
    fn snapshot(&self) -> &Snapshot {
        self.snapshot
    }
    fn render_view(&self) -> ExportResult<ExportImage> {
        (self.render_view)(self.snapshot)
    }
}

/// An admitted callback that may be executed off-thread without moving the host.
/// Return its completion to the originating host before writing any bytes.
pub struct ExportJob {
    authority: std::sync::Arc<()>,
    owner: PluginId,
    format: String,
    callback: ExportCallback,
}

/// Isolated worker result. Only the host can admit its bytes for persistence.
pub struct ExportCompletion {
    authority: std::sync::Arc<()>,
    owner: PluginId,
    format: String,
    elapsed: Duration,
    result: Result<ExportResult<Vec<u8>>, String>,
}

impl ExportJob {
    /// Execute against immutable capabilities, recording time and catching panics.
    pub fn run(self, cx: &dyn ExportCx) -> ExportCompletion {
        let start = Instant::now();
        let result = isolate(|| (self.callback)(cx));
        ExportCompletion {
            authority: self.authority,
            owner: self.owner,
            format: self.format,
            elapsed: start.elapsed(),
            result,
        }
    }
}

impl Host {
    fn export_owner(&self, id: &str) -> Option<usize> {
        self.plugins.iter().position(|plugin| {
            plugin.state == PluginState::Enabled && plugin.contributions.iter().any(|c| {
                matches!(c, Contribution::Format(format) if format.id == id && format.export)
            })
        })
    }

    /// Resolve a declared, enabled export callback. Missing bindings are inert.
    pub fn prepare_export(&mut self, id: &str) -> Result<Option<ExportJob>, HostError> {
        let Some(index) = self.export_owner(id) else {
            return Ok(None);
        };
        let callback = self.invoke(index, Phase::Export, |plugin| plugin.exporter(id))?;
        Ok(callback.map(|callback| ExportJob {
            authority: self.export_authority.clone(),
            owner: self.plugins[index].manifest.id.clone(),
            format: id.into(),
            callback,
        }))
    }

    /// Whether a declared, enabled export format has a callback.
    pub fn has_exporter(&mut self, id: &str) -> Result<bool, HostError> {
        Ok(self.prepare_export(id)?.is_some())
    }

    /// Record worker diagnostics and admit output only while its owner is enabled.
    /// Panics quarantine the owner, discarding every contribution like other hooks.
    pub fn finish_export(
        &mut self,
        completion: ExportCompletion,
    ) -> Result<Option<Vec<u8>>, HostError> {
        let ExportCompletion {
            authority,
            owner,
            format,
            elapsed,
            result,
        } = completion;
        if !std::sync::Arc::ptr_eq(&authority, &self.export_authority) {
            return Ok(None);
        }
        self.timings
            .record(owner.as_str(), "export_encode", elapsed);
        self.collect_budget_notices();
        let Some(index) = self.plugins.iter().position(|p| p.manifest.id == owner) else {
            return Ok(None);
        };
        self.ensure_healthy(index)?;
        let result = result.map_err(|message| {
            let phase = Phase::Export;
            let plugin = &mut self.plugins[index];
            plugin.state = PluginState::Faulted {
                phase,
                message: message.clone(),
            };
            plugin.contributions = Contributions::default();
            self.events.push(HostEvent::Faulted {
                plugin_id: owner.clone(),
                phase,
                message: message.clone(),
            });
            HostError::Faulted {
                plugin_id: owner,
                phase,
                message,
            }
        })?;
        if self.export_owner(&format) != Some(index) {
            return Ok(None);
        }
        result.map(Some).map_err(|e| HostError::Export {
            message: e.to_string(),
        })
    }

    /// Invoke an enabled exporter synchronously through the same worker admission path.
    pub fn dispatch_export(
        &mut self,
        id: &str,
        cx: &dyn ExportCx,
    ) -> Result<Option<Vec<u8>>, HostError> {
        let Some(job) = self.prepare_export(id)? else {
            return Ok(None);
        };
        self.finish_export(job.run(cx))
    }
}
