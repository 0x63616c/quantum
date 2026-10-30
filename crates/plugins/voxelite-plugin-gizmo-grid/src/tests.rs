use super::*;
use voxelite_plugin_host::{Host, conformance};

fn host() -> Host {
    Host::new(vec![Box::new(GizmoGridPlugin)])
}
fn bounds() -> Region {
    Region::new([-13, -7, 5], [19, 25, 37]).unwrap()
}
fn basis() -> [[f64; 3]; 3] {
    [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]
}
fn input(gesture: OverlayGesture, position: [f32; 2], drag: Option<OverlayDrag>) -> OverlayInput {
    OverlayInput {
        gesture,
        position,
        pointer: position,
        drag,
    }
}
fn dispatch(host: &mut Host, input: OverlayInput) -> OverlayOutcome {
    host.dispatch_overlay(GIZMO_OVERLAY, bounds(), basis(), input)
        .unwrap()
        .unwrap()
}

#[test]
fn conforms_and_binds_both_passes_only_when_enabled() {
    conformance::check(|| Box::new(GizmoGridPlugin));
    let mut host = host();
    for id in [GRID_OVERLAY, GIZMO_OVERLAY] {
        assert!(host.has_overlay(id).unwrap());
    }
    assert_eq!(host.registry().render_passes().count(), 2);
    assert_eq!(
        host.registry().commands().next().unwrap().1.id,
        GRID_COMMAND
    );
    host.set_setting(GRID_SETTING, SettingValue::Bool(false))
        .unwrap();
    host.disable(PLUGIN_ID).unwrap();
    assert!(host.registry().iter().next().is_none());
    assert!(host.draw_overlays(bounds(), basis(), None).is_empty());
    for id in [GRID_OVERLAY, GIZMO_OVERLAY] {
        assert!(!host.has_overlay(id).unwrap());
    }
    assert!(
        host.dispatch_overlay(
            GIZMO_OVERLAY,
            bounds(),
            basis(),
            input(OverlayGesture::Press { double: false }, [56.; 2], None)
        )
        .unwrap()
        .is_none()
    );
    host.enable(PLUGIN_ID).unwrap();
    assert_eq!(host.setting(GRID_SETTING), Some(&SettingValue::Bool(false)));
    assert!(
        host.draw_overlays(bounds(), basis(), None)[0]
            .primitives
            .is_empty()
    );
    assert!(host.has_overlay(GIZMO_OVERLAY).unwrap());
}

#[test]
fn grid_geometry_style_and_setting_match_the_original_viewport() {
    let mut host = host();
    let frames = host.draw_overlays(bounds(), basis(), None);
    assert_eq!(frames[0].slot, RenderPassSlot::BeforeScene);
    assert_eq!(
        frames[0].primitives,
        vec![OverlayPrimitive::Grid(OverlayGrid {
            plane: -7,
            offset: -0.05,
            min: -29,
            max: 53,
            spacing: 4,
            width: 0.6,
            color: OverlayColor::Line,
            opacity: 0.24,
        })]
    );
    let original = frames[1].clone();
    host.set_setting(GRID_SETTING, SettingValue::Bool(false))
        .unwrap();
    let frames = host.draw_overlays(bounds(), basis(), None);
    assert!(frames[0].primitives.is_empty());
    assert_eq!(frames[1], original);
    host.set_setting(GRID_SETTING, SettingValue::Bool(true))
        .unwrap();
    assert!(
        !host.draw_overlays(bounds(), basis(), None)[0]
            .primitives
            .is_empty()
    );
}

#[test]
fn front_view_draw_order_labels_highlights_and_circle_hits_are_unchanged() {
    let mut host = host();
    let frame = host
        .draw_overlays(bounds(), basis(), Some((GIZMO_OVERLAY, 5)))
        .pop()
        .unwrap();
    let OverlayPrimitive::Widget(widget) = &frame.primitives[0] else {
        panic!()
    };
    assert_eq!((widget.size, widget.inset), (112., 12.));
    assert_eq!(widget.lines.len(), 6);
    assert_eq!(
        widget
            .circles
            .iter()
            .map(|c| (c.center, c.label.as_deref()))
            .collect::<Vec<_>>(),
        [
            ([56., 56.], None),
            ([18., 56.], None),
            ([56., 94.], None),
            ([94., 56.], Some("X")),
            ([56., 18.], Some("Y")),
            ([56., 56.], Some("Z")),
        ]
    );
    for circle in &widget.circles {
        assert_eq!(circle.radius, 11.);
    }
    assert!(widget.circles[5].highlighted);
    assert_eq!(widget.circles[5].border, OverlayColor::Text);
    assert_eq!(widget.lines[5].width, 2.);
    assert_eq!(
        dispatch(&mut host, input(OverlayGesture::Hover, [56.; 2], None)).hovered,
        Some(5)
    );
    assert_eq!(
        dispatch(&mut host, input(OverlayGesture::Hover, [103., 65.], None)).hovered,
        None
    );
    for position in [[0., 0.], [-1., 56.], [f32::NAN, 0.], [f32::INFINITY, 0.]] {
        assert_eq!(
            dispatch(&mut host, input(OverlayGesture::Hover, position, None)).hovered,
            None
        );
    }
}

#[test]
fn drag_click_cancel_and_double_click_return_only_navigation() {
    let mut host = host();
    let down = dispatch(
        &mut host,
        input(OverlayGesture::Press { double: false }, [94., 56.], None),
    );
    let release = dispatch(
        &mut host,
        input(OverlayGesture::Release, [94., 56.], down.drag),
    );
    assert_eq!(
        release.navigation,
        Some(OverlayNavigation::Orient {
            yaw: std::f64::consts::FRAC_PI_2,
            pitch: 0.,
            view: Some("dev.voxelite.core.right".into())
        })
    );
    assert!(
        dispatch(
            &mut host,
            input(OverlayGesture::Release, [56.; 2], down.drag)
        )
        .navigation
        .is_none()
    );
    let slight = dispatch(
        &mut host,
        input(
            OverlayGesture::Move { primary_down: true },
            [96., 56.],
            down.drag,
        ),
    );
    assert!(slight.navigation.is_none());
    let moved = dispatch(
        &mut host,
        input(
            OverlayGesture::Move { primary_down: true },
            [97., 56.],
            slight.drag,
        ),
    );
    assert_eq!(moved.navigation, Some(OverlayNavigation::Orbit([1., 0.])));
    assert!(
        dispatch(
            &mut host,
            input(OverlayGesture::Release, [94., 56.], moved.drag)
        )
        .navigation
        .is_none()
    );
    let cancel = dispatch(
        &mut host,
        input(
            OverlayGesture::Move {
                primary_down: false,
            },
            [94., 56.],
            moved.drag,
        ),
    );
    assert!(cancel.drag.is_none() && cancel.navigation.is_none());
    let double = dispatch(
        &mut host,
        input(OverlayGesture::Press { double: true }, [0.; 2], moved.drag),
    );
    assert_eq!(double.navigation, Some(OverlayNavigation::Frame));
    assert!(double.drag.is_none());
    for outcome in [down, release, slight, moved, cancel, double] {
        assert!(outcome.commands.is_empty());
    }
}
