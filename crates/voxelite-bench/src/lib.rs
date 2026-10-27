//! Independent performance and visual tools using the production GPU viewport.
// Public modules form the harness API; report field definitions are documented in
// docs/perf/rendering-harness.md, rather than repeating their JSON names here.
#![allow(missing_docs)]
pub mod bench;
pub mod cli;
pub mod compare;
pub mod metrics;
pub mod paths;
pub mod report;
pub mod scenes;
pub mod visual;

use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use voxelite_core::Region;
use voxelite_render::gpu::Style;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const STYLE_ID: &str = "app-dark-grid-v1";
pub fn style(bounds: Region) -> Style {
    // gpu_viewport.rs + theme::Theme::Dark + gizmo-grid's default Grid overlay.
    Style {
        background: [0.0, 0.0, 0.0, 1.0],
        outline: false,
        lighting: voxelite_render::Lighting::Exact,
        grid: Some([
            bounds.min[1] as f32 - 0.05,
            (bounds.min[0].min(bounds.min[2]) - 16) as f32,
            (bounds.max[0].max(bounds.max[2]) + 16) as f32,
            4.0,
        ]),
        grid_color: [35.0 / 255.0, 35.0 / 255.0, 35.0 / 255.0, 0.24],
    }
}
/// Small safe executor; the renderer explicitly polls wgpu for submitted work.
pub fn block_on<F: Future>(future: F) -> F::Output {
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
