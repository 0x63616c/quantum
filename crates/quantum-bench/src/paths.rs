//! Shared scripted cameras; no live UI state is consulted.
use crate::Result;
use quantum_core::Snapshot;
use quantum_render::{Camera, PresetView, Projection};
pub fn fit(snapshot: &Snapshot, display: [u32; 2]) -> Result<Camera> {
    Ok(Camera::fit(
        snapshot,
        PresetView::Iso,
        f64::from(display[0]) / f64::from(display[1]),
        false,
    )?)
}
pub fn orbit(camera: Camera, degrees: f64) -> Camera {
    let (s, c) = degrees.to_radians().sin_cos();
    let d = [
        camera.eye[0] - camera.target[0],
        camera.eye[2] - camera.target[2],
    ];
    Camera {
        eye: [
            camera.target[0] + d[0] * c - d[1] * s,
            camera.eye[1],
            camera.target[2] + d[0] * s + d[1] * c,
        ],
        ..camera
    }
}
pub fn close(camera: Camera) -> Camera {
    match camera.projection {
        Projection::Orthographic { height } => Camera {
            projection: Projection::Orthographic {
                height: height / 4.0,
            },
            ..camera
        },
        Projection::Perspective { .. } => Camera {
            eye: std::array::from_fn(|a| {
                camera.target[a] + (camera.eye[a] - camera.target[a]) / 4.0
            }),
            ..camera
        },
    }
}
pub fn sequence(camera: Camera, name: &str, count: usize) -> Vec<Camera> {
    (0..count)
        .map(|i| match name {
            "fit" => camera,
            "orbit_fit" => orbit(camera, i as f64 * 3.0),
            "orbit_close" => orbit(close(camera), i as f64 * 3.0),
            _ => unreachable!("internal camera path"),
        })
        .collect()
}
pub fn stills(scene: &str, camera: Camera) -> Vec<(&'static str, Camera)> {
    match scene {
        "autumn" => vec![
            ("fit", camera),
            ("close", close(camera)),
            ("orbit90", orbit(camera, 90.0)),
            ("orbit180", orbit(camera, 180.0)),
        ],
        "terrain10m" => vec![("fit", camera)],
        _ => vec![("fit", camera), ("close", close(camera))],
    }
}
