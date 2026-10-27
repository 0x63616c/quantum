//! Adapter tests are opt-in; use headless-vulkan on Linux.
use super::*;
use voxelite_core::*;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::task::{Context, Poll, Wake, Waker};
    struct Parker(std::thread::Thread);
    impl Wake for Parker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let mut future = std::pin::pin!(future);
    let waker = Waker::from(Arc::new(Parker(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
            return result;
        }
        std::thread::park();
    }
}

#[test]
#[ignore = "requires a Metal or Vulkan adapter (lavapipe is supported)"]
fn runtime_diagnostics_release_resources_and_preserve_pixels() {
    block_on(async {
        let mut engine =
            Engine::new(Document::new(Uuid::from_u128(1), Uuid::from_u128(2))).unwrap();
        engine
            .execute(Transaction {
                expected_revision: 0,
                id: Uuid::from_u128(3),
                actor: Actor::new(Origin::Human),
                label: "Cube".into(),
                command: Command::Primitive {
                    layer: Uuid::from_u128(2),
                    shape: Shape::Box {
                        region: Region::new([0; 3], [3; 3]).unwrap(),
                        hollow: false,
                    },
                    mode: Mode::Place,
                    slot: Some(1),
                    mirror: Mirror::default(),
                },
            })
            .unwrap();
        let snapshot = engine.document().content.clone();
        let camera = Camera::fit(&snapshot, crate::PresetView::Iso, 1.0, false).unwrap();
        let style = Style {
            background: [0., 0., 0., 1.],
            outline: true,
            ambient_occlusion_strength: Default::default(),
            grid: None,
            grid_color: [0.; 4],
        };
        let mut gpu = Headless::new(HeadlessOptions::default()).await.unwrap();
        assert!(gpu.last_stats().is_none());
        assert!(gpu.viewport.timer.is_none());
        gpu.set_diagnostics(false);
        let reference = gpu
            .render(
                Some(snapshot.clone()),
                snapshot.bounds,
                camera,
                [64; 2],
                style,
                true,
            )
            .await
            .unwrap();
        assert!(gpu.viewport.stats.is_none());
        assert!(gpu.viewport.timer.is_none());
        assert!(gpu.last_stats().is_none());
        let vertices = gpu
            .viewport
            .buffers
            .values()
            .map(|b| u64::from(b.vertices))
            .sum::<u64>();
        assert!(vertices > 0);
        let vertex_bytes = gpu
            .viewport
            .buffers
            .values()
            .map(|b| b.buffer.size())
            .sum::<u64>();
        // Warm frames must reuse scene buffers across repeated toggles.
        for enabled in [true, false, true, false] {
            gpu.set_diagnostics(enabled);
            let frame = gpu
                .render(None, snapshot.bounds, camera, [64; 2], style, true)
                .await
                .unwrap();
            assert_eq!(
                frame.rgba, reference.rgba,
                "diagnostics {enabled}: max delta must be 0"
            );
            assert_eq!(gpu.viewport.stats.is_some(), enabled);
            assert_eq!(gpu.last_stats().is_some(), enabled);
            assert_eq!(
                gpu.viewport.timer.is_some(),
                enabled
                    && gpu
                        .viewport
                        .device
                        .features()
                        .contains(wgpu::Features::TIMESTAMP_QUERY)
            );
            if enabled {
                assert!(frame.stats.wall_ms > 0.0);
                assert_eq!(frame.stats.occupancy_sync_ms, 0.0);
                assert_eq!(frame.stats.mesh_sync_ms, 0.0);
                assert_eq!(frame.stats.chunks_updated, 0);
                assert_eq!(frame.stats.resident_chunks, 1);
                assert_eq!(frame.stats.occupied_bricks, 1);
                assert!(frame.stats.total_bricks >= 1);
                assert_eq!(frame.stats.draw_calls, 1);
                assert_eq!(frame.stats.vertices, vertices);
                assert_eq!(frame.stats.triangles, vertices / 3);
                assert_eq!(frame.stats.vertex_bytes, vertex_bytes);
                assert!(frame.stats.occupancy_bytes > 0);
                assert_eq!(frame.stats.target_size, [128; 2]);
                assert_eq!(frame.stats.target_bytes, 128 * 128 * 80);
                assert_eq!(frame.stats.shadow_path, ShadowPath::Exact);
            } else {
                assert_eq!(frame.stats.wall_ms, 0.0);
                assert!(frame.stats.gpu.is_none());
            }
        }
        // Removing scene content while enabled measures deletions, then clears
        // per-frame counters on the next camera-only frame.
        gpu.set_diagnostics(true);
        let empty = Document::new(Uuid::from_u128(1), Uuid::from_u128(2)).content;
        let frame = gpu
            .render(Some(empty), snapshot.bounds, camera, [64; 2], style, false)
            .await
            .unwrap();
        assert!(frame.stats.chunks_updated > 0);
        assert_eq!(frame.stats.resident_chunks, 0);
        assert_eq!(frame.stats.occupied_bricks, 0);
        assert_eq!(frame.stats.vertices, 0);
        assert_eq!(frame.stats.vertex_bytes, 0);
        assert!(frame.stats.occupancy_sync_ms > 0.0);
        let frame = gpu
            .render(None, snapshot.bounds, camera, [64; 2], style, false)
            .await
            .unwrap();
        assert_eq!(frame.stats.chunks_updated, 0);
        assert_eq!(frame.stats.occupancy_sync_ms, 0.0);
        gpu.reset_scene().unwrap();
        assert!(gpu.last_stats().is_none());
    });
}
