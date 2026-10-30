//! Camera gizmo geometry and pointer behavior, independent of camera ownership or GUI.
use voxelite_plugin_api::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AxisEnd {
    axis: usize,
    positive: bool,
}
impl AxisEnd {
    const ALL: [Self; 6] = [
        Self {
            axis: 0,
            positive: false,
        },
        Self {
            axis: 1,
            positive: false,
        },
        Self {
            axis: 2,
            positive: false,
        },
        Self {
            axis: 0,
            positive: true,
        },
        Self {
            axis: 1,
            positive: true,
        },
        Self {
            axis: 2,
            positive: true,
        },
    ];
    fn direction(self) -> [f64; 3] {
        let direction = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]][self.axis];
        direction.map(|v| v * if self.positive { 1.0 } else { -1.0 })
    }
    fn id(self) -> u32 {
        self.axis as u32 + if self.positive { 3 } else { 0 }
    }
    fn navigation(self) -> OverlayNavigation {
        use std::f64::consts::{FRAC_PI_2, PI};
        let (yaw, pitch) = match (self.axis, self.positive) {
            (0, true) => (FRAC_PI_2, 0.0),
            (0, false) => (-FRAC_PI_2, 0.0),
            (1, true) => (0.0, FRAC_PI_2),
            (1, false) => (0.0, -FRAC_PI_2),
            (2, true) => (0.0, 0.0),
            (2, false) => (PI, 0.0),
            _ => unreachable!("axis is X, Y or Z"),
        };
        let view = match (self.axis, self.positive) {
            (0, true) => Some("dev.voxelite.core.right"),
            (1, true) => Some("dev.voxelite.core.top"),
            (2, true) => Some("dev.voxelite.core.front"),
            _ => None,
        }
        .map(str::to_owned);
        OverlayNavigation::Orient { yaw, pitch, view }
    }
}

#[derive(Clone, Copy, Debug)]
struct ProjectedEnd {
    end: AxisEnd,
    position: [f32; 2],
    depth: f64,
}
const SIZE: f32 = 112.0;
const HIT_RADIUS: f32 = 11.0;
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn project([right, up, back]: [[f64; 3]; 3]) -> [ProjectedEnd; 6] {
    let mut ends = AxisEnd::ALL.map(|end| {
        let direction = end.direction();
        ProjectedEnd {
            end,
            position: [
                SIZE / 2.0 + dot(direction, right) as f32 * 38.0,
                SIZE / 2.0 - dot(direction, up) as f32 * 38.0,
            ],
            depth: dot(direction, back),
        }
    });
    ends.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    ends
}
fn hit_test(basis: [[f64; 3]; 3], point: [f32; 2]) -> Option<AxisEnd> {
    // The old viewport first checked the widget's square bounds.
    if !point.into_iter().all(|p| (0.0..=SIZE).contains(&p)) {
        return None;
    }
    project(basis)
        .into_iter()
        .rev()
        .find(|end| {
            let dx = point[0] - end.position[0];
            let dy = point[1] - end.position[1];
            dx * dx + dy * dy <= HIT_RADIUS * HIT_RADIUS
        })
        .map(|end| end.end)
}

pub(super) struct Gizmo;
impl Overlay for Gizmo {
    fn draw(&self, cx: &mut dyn OverlayCx, hovered: Option<u32>) {
        let ends = project(cx.camera_basis());
        let lines = ends
            .iter()
            .map(|p| OverlayLine {
                from: [SIZE / 2.0; 2],
                to: p.position,
                width: if hovered == Some(p.end.id()) {
                    2.0
                } else {
                    1.0
                },
                color: OverlayColor::Axis(p.end.axis),
            })
            .collect();
        let circles = ends
            .into_iter()
            .map(|p| {
                let color = OverlayColor::Axis(p.end.axis);
                let highlighted = hovered == Some(p.end.id());
                OverlayCircle {
                    center: p.position,
                    radius: HIT_RADIUS,
                    fill: if p.end.positive {
                        color
                    } else {
                        OverlayColor::Background
                    },
                    border: if highlighted {
                        OverlayColor::Text
                    } else {
                        color
                    },
                    label: p.end.positive.then(|| ["X", "Y", "Z"][p.end.axis].into()),
                    text: OverlayColor::Background,
                    highlighted,
                }
            })
            .collect();
        cx.emit(OverlayPrimitive::Widget(OverlayWidget {
            size: SIZE, inset: 12.0,
            label: "Camera orientation: drag to orbit, click an axis for its view, double-click for isometric".into(),
            lines, circles,
        }));
    }
    fn interact(
        &self,
        cx: &dyn OverlayCx,
        input: OverlayInput,
    ) -> Result<OverlayOutcome, ToolError> {
        let hit = hit_test(cx.camera_basis(), input.position);
        let mut outcome = OverlayOutcome::default();
        match input.gesture {
            OverlayGesture::Hover => outcome.hovered = hit.map(AxisEnd::id),
            OverlayGesture::Press { double: true } => {
                outcome.navigation = Some(OverlayNavigation::Frame)
            }
            OverlayGesture::Press { double: false } => {
                outcome.drag = Some(OverlayDrag {
                    origin: input.pointer,
                    last: input.pointer,
                    target: hit.map(AxisEnd::id),
                    moved: false,
                })
            }
            OverlayGesture::Move { primary_down: true } => {
                if let Some(mut drag) = input.drag {
                    let travel = [
                        input.pointer[0] - drag.origin[0],
                        input.pointer[1] - drag.origin[1],
                    ];
                    drag.moved |= travel[0].hypot(travel[1]) >= 3.0;
                    if drag.moved {
                        let delta = [
                            input.pointer[0] - drag.last[0],
                            input.pointer[1] - drag.last[1],
                        ];
                        outcome.navigation = Some(OverlayNavigation::Orbit(delta.map(f64::from)));
                    }
                    drag.last = input.pointer;
                    outcome.drag = Some(drag);
                }
            }
            OverlayGesture::Move {
                primary_down: false,
            } => {}
            OverlayGesture::Release => {
                if let Some(drag) = input.drag
                    && !drag.moved
                    && let Some(end) = hit
                    && drag.target == Some(end.id())
                {
                    outcome.navigation = Some(end.navigation());
                }
            }
        }
        Ok(outcome)
    }
}
