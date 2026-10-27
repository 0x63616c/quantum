//! UI-independent worker diagnostics, applied at the next serialized request.
use super::*;

impl Worker {
    /// Enable measurements for subsequent requests. Disabled by default; GPU
    /// query resources are created/released on the worker, never on the UI thread.
    pub fn set_diagnostics(&mut self, enabled: bool) {
        if self.diagnostics != enabled {
            self.diagnostics = enabled;
            self.last_stats.set(None);
        }
    }

    /// Measurements from the last completed render, or None while disabled or
    /// awaiting the first diagnostic frame. Reading never polls or reads the GPU.
    pub fn last_stats(&self) -> Option<FrameStats> {
        self.last_stats.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a real macOS Metal device"]
    fn worker_diagnostics_toggle_without_recreating_the_scene() {
        objc2::rc::autoreleasepool(|_| {
            pollster::block_on(async {
                let id = objc2_metal::MTLCreateSystemDefaultDevice()
                    .unwrap()
                    .registryID();
                let mut worker = Worker::new(id).unwrap();
                let snapshot = voxelite_core::Document::new(
                    voxelite_core::Uuid::from_u128(1),
                    voxelite_core::Uuid::from_u128(2),
                )
                .content;
                let camera = Camera::fit(&snapshot, crate::PresetView::Iso, 1.0, false).unwrap();
                let style = Style {
                    background: [0., 0., 0., 1.],
                    outline: false,
                    grid: None,
                    grid_color: [0.; 4],
                };
                assert!(worker.last_stats().is_none());
                for (index, enabled) in [false, true, false, true].into_iter().enumerate() {
                    worker.set_diagnostics(enabled);
                    assert!(worker.last_stats().is_none());
                    let frame = worker
                        .render(
                            (index == 0).then(|| snapshot.clone()),
                            snapshot.bounds,
                            camera,
                            [17, 13],
                            style,
                        )
                        .await
                        .unwrap();
                    assert_eq!(CVPixelBufferGetWidth(&frame.0), 34);
                    assert_eq!(worker.last_stats().is_some(), enabled);
                    if let Some(stats) = worker.last_stats() {
                        assert_eq!(stats.target_size, [34, 26]);
                        assert_eq!(stats.occupancy_sync_ms, 0.0);
                        assert!(stats.device_allocated_bytes.is_some());
                    }
                }
            });
        });
    }
}
