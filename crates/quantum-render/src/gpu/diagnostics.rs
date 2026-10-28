//! Opt-in measurement shared by the worker and headless renderer.
use super::*;

impl Viewport {
    pub(super) fn set_diagnostics(&mut self, enabled: bool) {
        if enabled {
            self.stats.get_or_insert_with(FrameStats::default);
            if self.timer.is_none()
                && self
                    .device
                    .features()
                    .contains(wgpu::Features::TIMESTAMP_QUERY)
            {
                self.timer = Some(GpuTimer::new(&self.device, &self.queue));
            }
        } else {
            self.stats = None;
            self.timer = None;
        }
    }
}

/// No clocks, queries, readbacks or heap allocations when disabled.
pub(super) struct FrameDiagnostics {
    started: Option<Instant>,
    pub(super) sync: Option<FrameStats>,
    #[cfg(target_os = "macos")]
    presentation: Option<Instant>,
    submitted: Option<Instant>,
}
impl FrameDiagnostics {
    pub(super) fn new(enabled: bool) -> Self {
        Self {
            started: enabled.then(Instant::now),
            sync: enabled.then(FrameStats::default),
            #[cfg(target_os = "macos")]
            presentation: None,
            submitted: None,
        }
    }

    #[cfg(target_os = "macos")]
    pub(super) fn presenting(&mut self) {
        self.presentation = self.started.map(|_| Instant::now());
    }

    pub(super) fn submitting(&mut self) {
        self.submitted = self.started.map(|_| Instant::now());
        #[cfg(target_os = "macos")]
        if let Some(sync) = &mut self.sync {
            // IOSurface acquisition and NV12 encoding belong to CPU encode.
            sync.encode_ms = elapsed_ms(self.presentation);
        }
    }

    pub(super) fn finish(self, viewport: &mut Viewport) -> RenderResult<Option<FrameStats>> {
        let Some(mut stats) = viewport.stats else {
            return Ok(None);
        };
        stats.submit_complete_ms = elapsed_ms(self.submitted);
        stats.wall_ms = elapsed_ms(self.started);
        if let Some(sync) = self.sync {
            stats.occupancy_sync_ms = sync.occupancy_sync_ms;
            stats.mesh_sync_ms = sync.mesh_sync_ms;
            stats.encode_ms += sync.encode_ms;
        }
        if let Some(timer) = &viewport.timer {
            // Missing/invalid timestamps must not invalidate an otherwise good frame.
            stats.gpu = timer.read(&viewport.device).ok().flatten();
        }
        #[cfg(target_os = "macos")]
        {
            stats.device_allocated_bytes = mac::device_allocated_bytes(&viewport.device);
        }
        viewport.check_error()?;
        viewport.stats = Some(stats);
        Ok(Some(stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_frames_do_not_sample_clocks_or_collect_stats() {
        let mut frame = FrameDiagnostics::new(false);
        frame.submitting();
        assert!(frame.started.is_none());
        assert!(frame.submitted.is_none());
        assert!(frame.sync.is_none());
    }
}
