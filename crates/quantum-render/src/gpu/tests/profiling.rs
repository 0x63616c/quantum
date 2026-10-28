//! Exercise the real GPU path and tracing's poll/return boundaries together.
use super::*;
use std::task::{Context, Poll, Wake, Waker};
use tracing::{Subscriber, span::Id};
use tracing_subscriber::{Layer, layer, prelude::*, registry::LookupSpan};

#[derive(Default)]
struct Zones {
    entered: Vec<&'static str>,
    stack: Vec<Id>,
}

struct Recorder(Arc<Mutex<Zones>>);

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Recorder {
    fn on_new_span(&self, attrs: &tracing::span::Attributes<'_>, _: &Id, _: layer::Context<'_, S>) {
        assert!(
            attrs.metadata().fields().is_empty(),
            "arguments must be skipped"
        );
    }

    fn on_enter(&self, id: &Id, ctx: layer::Context<'_, S>) {
        let mut zones = self.0.lock().unwrap();
        zones.entered.push(ctx.span(id).unwrap().name());
        zones.stack.push(id.clone());
    }

    fn on_exit(&self, id: &Id, _: layer::Context<'_, S>) {
        assert_eq!(self.0.lock().unwrap().stack.pop().as_ref(), Some(id));
    }
}

fn block_on<F: std::future::Future>(future: F, zones: &Mutex<Zones>) -> F::Output {
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
        let result = future.as_mut().poll(&mut context);
        assert!(
            zones.lock().unwrap().stack.is_empty(),
            "zone escaped a poll"
        );
        if let Poll::Ready(result) = result {
            return result;
        }
        std::thread::park();
    }
}

#[test]
#[ignore = "requires a Metal or Vulkan adapter (lavapipe is supported)"]
fn profiling_gpu_zones_preserve_pixels_and_exit_on_errors() {
    let mut snapshot = quantum_core::Document::default().content;
    snapshot.layers[0].voxels = serde_json::from_value(serde_json::json!([[0, 0, 0, 1]])).unwrap();
    let camera = Camera::fit(&snapshot, crate::PresetView::Iso, 1.0, false).unwrap();
    let style = Style {
        lighting: crate::Lighting::Exact,
        background: [0., 0., 0., 1.],
        outline: true,
        ambient_occlusion_strength: Default::default(),
        grid: None,
        grid_color: [0.; 4],
    };
    let zones = Arc::new(Mutex::new(Zones::default()));
    let mut gpu = block_on(Headless::new(HeadlessOptions::default()), &zones).unwrap();
    let mut edited = snapshot.clone();
    edited.layers[0].voxels =
        serde_json::from_value(serde_json::json!([[0, 0, 0, 1], [1, 0, 0, 1]])).unwrap();
    let [reference, edited_reference] = [&snapshot, &edited].map(|source| {
        gpu.reset_scene().unwrap();
        block_on(
            gpu.render(
                Some(source.clone()),
                source.bounds,
                camera,
                [32; 2],
                style,
                true,
            ),
            &zones,
        )
        .unwrap()
    });
    assert_eq!(reference.rgba.as_ref().unwrap().len(), 64 * 64 * 4);
    assert_ne!(edited_reference.rgba, reference.rgba);
    gpu.reset_scene().unwrap();

    let subscriber = tracing_subscriber::registry().with(
        Recorder(Arc::clone(&zones)).with_filter(
            tracing_subscriber::filter::Targets::new()
                .with_target("quantum_render", tracing::Level::INFO),
        ),
    );
    tracing::subscriber::with_default(subscriber, || {
        // Early return before encoding must leave no entered span behind.
        assert!(
            block_on(
                gpu.render(None, snapshot.bounds, camera, [32; 2], style, true),
                &zones,
            )
            .is_err()
        );
        // Cold, camera-only and incremental edit frames must match untraced rebuilds.
        for (source, expected) in [
            (Some(snapshot.clone()), &reference),
            (None, &reference),
            (Some(edited), &edited_reference),
        ] {
            let actual = block_on(
                gpu.render(source, snapshot.bounds, camera, [32; 2], style, true),
                &zones,
            )
            .unwrap();
            assert_eq!(actual.size, expected.size);
            assert_eq!(actual.rgba, expected.rgba);
        }
        // A nested encoding error must unwind both async spans in order.
        let invalid = Camera {
            eye: camera.target,
            ..camera
        };
        assert!(
            block_on(
                gpu.render(None, snapshot.bounds, invalid, [32; 2], style, true),
                &zones,
            )
            .is_err()
        );
    });
    let zones = zones.lock().unwrap();
    assert!(zones.stack.is_empty());
    for name in [
        "render.gpu.headless",
        "render.gpu.encode",
        "render.gpu.occupancy",
        "mesh.sync",
        "mesh.chunk",
    ] {
        assert!(
            zones.entered.contains(&name),
            "missing entered zone: {name}"
        );
    }
}
