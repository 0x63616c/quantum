//! Deterministic orbit-camera transitions driven by a caller-supplied clock.
use std::{
    f64::consts::{PI, TAU},
    time::Duration,
};

/// An orbit rig's complete pose, independent of any UI or vector library.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitPose {
    /// World-space look-at point, including pan.
    pub target: [f64; 3],
    /// Azimuth in radians.
    pub yaw: f64,
    /// Elevation in radians, including exact pole views.
    pub pitch: f64,
    /// Vertical world-space viewport extent.
    pub span: f64,
    /// Distance from the look-at point to the eye.
    pub distance: f64,
}

impl OrbitPose {
    /// Interpolate with shortest-path yaw and geometric positive zoom extents.
    ///
    /// Progress is clamped to `[0, 1]`; non-finite progress selects the destination.
    /// Endpoints are returned unchanged, including their original yaw representation.
    /// Interior yaw is normalized to `[0, TAU)`; a half-turn tie takes positive yaw.
    pub fn lerp(self, other: Self, t: f64) -> Self {
        let t = progress(t);
        if t == 0.0 {
            return self;
        }
        if t == 1.0 {
            return other;
        }
        Self {
            target: std::array::from_fn(|i| linear(self.target[i], other.target[i], t)),
            yaw: normalize_yaw(self.yaw.rem_euclid(TAU) + yaw_delta(self.yaw, other.yaw) * t),
            pitch: linear(self.pitch, other.pitch, t),
            span: geometric(self.span, other.span, t),
            distance: geometric(self.distance, other.distance, t),
        }
    }

    fn is_finite(self) -> bool {
        self.target.into_iter().all(f64::is_finite)
            && [self.yaw, self.pitch, self.span, self.distance]
                .into_iter()
                .all(f64::is_finite)
    }

    fn near(self, other: Self) -> bool {
        const EPSILON: f64 = 1e-10;
        self.target
            .into_iter()
            .zip(other.target)
            .all(|(a, b)| (a - b).abs() <= EPSILON)
            && yaw_delta(self.yaw, other.yaw).abs() <= EPSILON
            && (self.pitch - other.pitch).abs() <= EPSILON
            && (self.span - other.span).abs() <= EPSILON
            && (self.distance - other.distance).abs() <= EPSILON
    }
}

fn progress(t: f64) -> f64 {
    if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn linear(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

fn geometric(a: f64, b: f64, t: f64) -> f64 {
    if a > 0.0 && b > 0.0 && a.is_finite() && b.is_finite() {
        linear(a.ln(), b.ln(), t).exp()
    } else {
        linear(a, b, t)
    }
}

fn yaw_delta(from: f64, to: f64) -> f64 {
    let delta = (to.rem_euclid(TAU) - from.rem_euclid(TAU)).rem_euclid(TAU);
    if delta > PI { delta - TAU } else { delta }
}

fn normalize_yaw(yaw: f64) -> f64 {
    let yaw = yaw.rem_euclid(TAU);
    // rem_euclid can round a tiny negative input up to the divisor itself.
    if yaw == TAU { 0.0 } else { yaw }
}

/// Symmetric cubic easing with slow starts and finishes and clamped progress.
/// Non-finite progress selects the finished endpoint.
pub fn ease_in_out_cubic(t: f64) -> f64 {
    let t = progress(t);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Default duration for discrete camera navigation.
pub const DEFAULT_TRANSITION: Duration = Duration::from_millis(250);

/// A sampled transition pose and whether it is the exact destination.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// Interpolated pose, or the unchanged destination on completion.
    pub pose: OrbitPose,
    /// True only for the final sample; subsequent samples return `None`.
    pub finished: bool,
}

#[derive(Clone, Copy, Debug)]
struct Transition {
    from: OrbitPose,
    to: OrbitPose,
    start: Duration,
    duration: Duration,
}

/// Interruptible camera interpolation with no timers, UI dependencies or idle work.
#[derive(Debug, Default)]
pub struct CameraAnimator {
    transition: Option<Transition>,
}

impl CameraAnimator {
    /// Construct an idle animator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a transition still needs sampling.
    pub fn is_animating(&self) -> bool {
        self.transition.is_some()
    }

    /// Destination of the active transition, or `None` when idle.
    pub fn target(&self) -> Option<OrbitPose> {
        self.transition.map(|transition| transition.to)
    }

    /// Start or retarget from the caller's current pose using time since its epoch.
    ///
    /// Returns false and cancels existing motion for zero duration, non-finite poses,
    /// or poses within `1e-10` on every field (yaw compared modulo a full turn).
    /// The caller handles snapping to the destination in those cases.
    pub fn start(
        &mut self,
        from: OrbitPose,
        to: OrbitPose,
        now: Duration,
        duration: Duration,
    ) -> bool {
        self.cancel();
        if duration.is_zero() || !from.is_finite() || !to.is_finite() || from.near(to) {
            return false;
        }
        self.transition = Some(Transition {
            from,
            to,
            start: now,
            duration,
        });
        true
    }

    /// Stop motion without producing or changing a pose. Safe to call while idle.
    pub fn cancel(&mut self) {
        self.transition = None;
    }

    /// Sample eased progress using the same epoch as `start`.
    ///
    /// Earlier times produce the exact starting pose. Completion returns the exact
    /// destination once and makes the animator idle; idle sampling returns `None`.
    pub fn sample(&mut self, now: Duration) -> Option<Sample> {
        let transition = self.transition?;
        let elapsed = now.saturating_sub(transition.start);
        let finished = elapsed >= transition.duration;
        let pose = if finished {
            self.cancel();
            transition.to
        } else {
            let t = elapsed.as_secs_f64() / transition.duration.as_secs_f64();
            transition.from.lerp(transition.to, ease_in_out_cubic(t))
        };
        Some(Sample { pose, finished })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose() -> OrbitPose {
        OrbitPose {
            target: [-0.0, 2.0, -3.0],
            yaw: -PI / 2.0,
            pitch: 0.1,
            span: 4.0,
            distance: 9.0,
        }
    }

    fn destination() -> OrbitPose {
        OrbitPose {
            target: [10.0, 12.0, 7.0],
            yaw: PI / 2.0,
            pitch: PI / 2.0,
            span: 16.0,
            distance: 36.0,
        }
    }

    fn bits(pose: OrbitPose) -> [u64; 7] {
        [
            pose.target[0],
            pose.target[1],
            pose.target[2],
            pose.yaw,
            pose.pitch,
            pose.span,
            pose.distance,
        ]
        .map(f64::to_bits)
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-12, "{a} != {b}");
    }

    #[test]
    fn lerp_endpoints_are_bit_exact_and_progress_is_clamped() {
        let (a, b) = (pose(), destination());
        for t in [0.0, -0.0, -1.0] {
            assert_eq!(bits(a.lerp(b, t)), bits(a));
        }
        for t in [1.0, 2.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(bits(a.lerp(b, t)), bits(b));
        }
    }

    #[test]
    fn yaw_crosses_the_seam_in_both_directions() {
        let a = OrbitPose { yaw: 0.1, ..pose() };
        let b = OrbitPose {
            yaw: TAU - 0.1,
            ..a
        };
        for (from, to) in [(a, b), (b, a)] {
            close(yaw_delta(0.0, from.lerp(to, 0.5).yaw), 0.0);
            for step in 1..100 {
                let yaw = from.lerp(to, f64::from(step) / 100.0).yaw;
                assert!((0.0..TAU).contains(&yaw));
                assert!(yaw_delta(0.0, yaw).abs() <= 0.1);
            }
        }
        close(a.lerp(b, 0.25).yaw, 0.05);
        close(b.lerp(a, 0.25).yaw, TAU - 0.05);
        let tiny = OrbitPose { yaw: -1e-15, ..a };
        let zero = OrbitPose { yaw: 0.0, ..a };
        assert!((0.0..TAU).contains(&zero.lerp(tiny, 0.1).yaw));
    }

    #[test]
    fn left_right_half_turn_ties_always_take_positive_yaw() {
        let (left, right) = (pose(), destination());
        close(left.lerp(right, 0.5).yaw, 0.0);
        close(right.lerp(left, 0.5).yaw, PI);
        for yaw in [PI, -PI, 3.0 * PI] {
            let from = OrbitPose { yaw: 0.0, ..pose() };
            let to = OrbitPose { yaw, ..from };
            close(from.lerp(to, 0.5).yaw, PI / 2.0);
        }
        close(yaw_delta(0.0, PI - 0.01), PI - 0.01);
        close(yaw_delta(0.0, PI + 0.01), -PI + 0.01);
    }

    #[test]
    fn target_and_pitch_are_linear_and_zoom_is_geometric() {
        let (a, b) = (pose(), destination());
        let middle = a.lerp(b, 0.5);
        assert_eq!(middle.target, [5.0, 7.0, 2.0]);
        close(middle.pitch, (a.pitch + b.pitch) / 2.0);
        close(middle.span, (a.span * b.span).sqrt());
        close(middle.distance, (a.distance * b.distance).sqrt());
        close(a.lerp(b, 0.25).pitch, a.pitch * 0.75 + b.pitch * 0.25);
        let a = OrbitPose {
            span: -4.0,
            distance: 0.0,
            ..a
        };
        assert_eq!(a.lerp(b, 0.5).span, 6.0);
        assert_eq!(a.lerp(b, 0.5).distance, 18.0);
        assert!(geometric(f64::INFINITY, 4.0, 0.5).is_infinite());
        assert!(geometric(4.0, f64::NAN, 0.5).is_nan());
        // Logs avoid overflow from forming the ratio or product of extreme extents.
        close(geometric(1e-300, 1e300, 0.5), 1.0);
    }

    #[test]
    fn cubic_easing_is_symmetric_monotonic_and_slow_at_ends() {
        assert_eq!(ease_in_out_cubic(-1.0), 0.0);
        assert_eq!(ease_in_out_cubic(0.0), 0.0);
        assert_eq!(ease_in_out_cubic(0.5), 0.5);
        assert_eq!(ease_in_out_cubic(1.0), 1.0);
        assert_eq!(ease_in_out_cubic(2.0), 1.0);
        assert_eq!(ease_in_out_cubic(f64::NAN), 1.0);
        let mut previous = 0.0;
        for step in 0..=1000 {
            let t = f64::from(step) / 1000.0;
            let value = ease_in_out_cubic(t);
            assert!(value >= previous);
            close(ease_in_out_cubic(1.0 - t), 1.0 - value);
            previous = value;
        }
        assert!(ease_in_out_cubic(0.01) < 0.001);
        assert!(1.0 - ease_in_out_cubic(0.99) < 0.001);
    }

    #[test]
    fn animator_samples_then_finishes_exactly_and_stays_idle() {
        let mut animator = CameraAnimator::new();
        assert!(!animator.is_animating());
        assert_eq!(animator.target(), None);
        assert_eq!(animator.sample(Duration::ZERO), None);
        let start = Duration::from_secs(1);
        assert!(animator.start(pose(), destination(), start, DEFAULT_TRANSITION));
        assert!(animator.is_animating());
        assert_eq!(animator.target(), Some(destination()));
        assert_eq!(bits(animator.sample(start).unwrap().pose), bits(pose()));
        let middle = animator.sample(start + DEFAULT_TRANSITION / 2).unwrap();
        assert!(!middle.finished);
        assert_eq!(middle.pose, pose().lerp(destination(), 0.5));
        let end = animator.sample(start + DEFAULT_TRANSITION).unwrap();
        assert!(end.finished);
        assert_eq!(bits(end.pose), bits(destination()));
        assert!(!animator.is_animating());
        assert_eq!(animator.target(), None);
        assert_eq!(animator.sample(start + DEFAULT_TRANSITION), None);
        assert_eq!(animator.sample(Duration::MAX), None);
    }

    #[test]
    fn retarget_is_continuous_and_cancel_is_idempotent() {
        let mut animator = CameraAnimator::new();
        animator.start(pose(), destination(), Duration::ZERO, DEFAULT_TRANSITION);
        let now = Duration::from_millis(100);
        let current = animator.sample(now).unwrap().pose;
        assert!(animator.start(current, pose(), now, DEFAULT_TRANSITION));
        assert_eq!(bits(animator.sample(now).unwrap().pose), bits(current));
        assert_eq!(animator.target(), Some(pose()));
        animator.cancel();
        animator.cancel();
        assert!(!animator.is_animating());
        assert_eq!(animator.target(), None);
        assert_eq!(animator.sample(now), None);
    }

    #[test]
    fn noop_and_zero_duration_cancel_existing_motion() {
        let a = pose();
        for (to, duration) in [
            (a, DEFAULT_TRANSITION),
            (
                OrbitPose {
                    yaw: a.yaw + TAU,
                    span: a.span + 1e-12,
                    ..a
                },
                DEFAULT_TRANSITION,
            ),
            (destination(), Duration::ZERO),
        ] {
            let mut animator = CameraAnimator::new();
            animator.start(a, destination(), Duration::ZERO, DEFAULT_TRANSITION);
            assert!(!animator.start(a, to, Duration::ZERO, duration));
            assert!(!animator.is_animating());
            assert_eq!(animator.sample(Duration::ZERO), None);
        }
    }

    #[test]
    fn every_non_finite_pose_field_is_rejected_in_either_endpoint() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for field in 0..7 {
                let mut invalid = pose();
                match field {
                    0..=2 => invalid.target[field] = value,
                    3 => invalid.yaw = value,
                    4 => invalid.pitch = value,
                    5 => invalid.span = value,
                    _ => invalid.distance = value,
                }
                for (from, to) in [(invalid, destination()), (pose(), invalid)] {
                    let mut animator = CameraAnimator::new();
                    animator.start(pose(), destination(), Duration::ZERO, DEFAULT_TRANSITION);
                    assert!(!animator.start(from, to, Duration::ZERO, DEFAULT_TRANSITION));
                    assert!(!animator.is_animating());
                    assert_eq!(animator.target(), None);
                }
            }
        }
    }

    #[test]
    fn time_before_start_and_past_finish_are_safe() {
        let mut animator = CameraAnimator::new();
        animator.start(
            pose(),
            destination(),
            Duration::from_secs(1),
            DEFAULT_TRANSITION,
        );
        let early = animator.sample(Duration::ZERO).unwrap();
        assert!(!early.finished);
        assert_eq!(bits(early.pose), bits(pose()));
        assert!(animator.sample(Duration::MAX).unwrap().finished);
    }

    #[test]
    fn many_step_animation_progress_is_monotonic() {
        let (a, b) = (pose(), destination());
        let mut animator = CameraAnimator::new();
        animator.start(a, b, Duration::ZERO, DEFAULT_TRANSITION);
        let mut previous = a;
        for step in 0..=1000 {
            let sample = animator.sample(DEFAULT_TRANSITION * step / 1000).unwrap();
            for axis in 0..3 {
                assert!(sample.pose.target[axis] >= previous.target[axis]);
                assert!(sample.pose.target[axis] <= b.target[axis]);
            }
            assert!(sample.pose.pitch >= previous.pitch);
            assert!(sample.pose.span >= previous.span);
            assert!(sample.pose.distance >= previous.distance);
            assert!(yaw_delta(previous.yaw, sample.pose.yaw) >= -1e-12);
            assert_eq!(sample.finished, step == 1000);
            previous = sample.pose;
        }
        assert_eq!(animator.sample(DEFAULT_TRANSITION), None);
    }
}
