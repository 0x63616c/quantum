// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/voxelite-ui/NOTICE
//! Elapsed-time playback and one-shot repaint scheduling.

use super::skeleton::FPS_MS;
use gpui::{App, Bounds, Context, Entity, Pixels, Subscription, Task, Window};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Default)]
pub(crate) struct Playback {
    pub start: Option<Instant>,
    pub elapsed: Option<Duration>,
}

pub(crate) struct Clock {
    start: Instant,
    loading: bool,
    timer: Option<Task<()>>,
    visibility: Option<Subscription>,
}
impl Clock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            loading: false,
            timer: None,
            visibility: None,
        }
    }
    pub fn elapsed(&mut self, playback: Playback, loading: bool, restart: bool) -> f64 {
        self.elapsed_at(playback, loading, restart, Instant::now())
    }
    fn elapsed_at(
        &mut self,
        playback: Playback,
        loading: bool,
        restart: bool,
        now: Instant,
    ) -> f64 {
        if loading != self.loading || restart {
            self.start = now;
        }
        self.loading = loading;
        playback
            .elapsed
            .unwrap_or_else(|| now.saturating_duration_since(playback.start.unwrap_or(self.start)))
            .as_secs_f64()
            * 1000.
    }
    pub fn stop(&mut self) {
        self.timer = None;
        self.visibility = None;
    }

    fn visibility_changed(&mut self, visible: bool) -> bool {
        if !visible {
            self.timer = None;
        }
        visible
    }
}

/// Called during prepaint: a clipped element never arms another timer.
pub(crate) fn schedule(
    clock: &Entity<Clock>,
    bounds: Bounds<Pixels>,
    active: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let visible = bounds.intersects(&window.content_mask().bounds) && window.is_visible();
    clock.update(cx, |clock, cx: &mut Context<Clock>| {
        if !active {
            clock.stop();
            return;
        }
        if clock.visibility.is_none() {
            clock.visibility = Some(cx.observe_window_visibility(
                window,
                |clock, visibility, _, cx| {
                    if clock.visibility_changed(visibility.is_visible()) {
                        // No polling while hidden: one repaint re-evaluates elapsed time
                        // and prepaint rearms only if this component is still animating.
                        cx.notify();
                    }
                },
            ));
        }
        if !should_arm(active, visible, clock.timer.is_some()) {
            return;
        }
        clock.timer = Some(cx.spawn_in(window, async move |state, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(FPS_MS))
                .await;
            let _ = state.update_in(cx, |clock, window, cx| {
                clock.timer = None;
                if window.is_visible() {
                    cx.notify();
                }
            });
        }));
    });
}

fn should_arm(animating: bool, visible: bool, pending: bool) -> bool {
    animating && visible && !pending
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timers_follow_visibility_and_animation_without_focus_or_idle_polling() {
        assert!(should_arm(true, true, false)); // Visible, including inactive windows.
        assert!(!should_arm(true, true, true)); // At most one queued timer.
        assert!(!should_arm(true, false, false)); // Hidden: wait for visibility event.
        assert!(should_arm(true, true, false)); // Visible again: rearm.
        assert!(!should_arm(false, true, false)); // Finished/static: no idle work.
        assert!(!should_arm(false, false, false));
        assert!(!should_arm(false, true, true));
    }

    #[test]
    fn hiding_cancels_the_timer_but_completion_also_removes_the_resume_observer() {
        let unsubscribed = std::rc::Rc::new(std::cell::Cell::new(0));
        let count = unsubscribed.clone();
        let mut clock = Clock::new();
        clock.visibility = Some(Subscription::new(move || count.set(count.get() + 1)));
        clock.timer = Some(Task::ready(()));
        assert!(!clock.visibility_changed(false));
        assert!(clock.timer.is_none());
        assert_eq!(unsubscribed.get(), 0);
        assert!(clock.visibility_changed(true));
        assert!(should_arm(true, true, clock.timer.is_some()));
        clock.timer = Some(Task::ready(()));
        clock.stop();
        assert!(clock.timer.is_none());
        assert!(clock.visibility.is_none());
        assert_eq!(unsubscribed.get(), 1);
        clock.stop();
        assert_eq!(unsubscribed.get(), 1);
    }

    #[test]
    fn automatic_clock_restarts_when_data_arrives_and_preserves_elapsed_between_renders() {
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        let mut clock = Clock::new();
        let automatic = Playback::default();
        assert_eq!(clock.elapsed_at(automatic, true, true, start), 0.);
        assert_eq!(clock.elapsed_at(automatic, true, false, at(3000)), 3000.);
        assert_eq!(clock.elapsed_at(automatic, false, false, at(3010)), 0.);
        assert_eq!(clock.elapsed_at(automatic, false, false, at(3110)), 100.);
        assert_eq!(clock.elapsed_at(automatic, false, true, at(3200)), 0.);
    }

    #[test]
    fn caller_clocks_override_automatic_restarts_and_snapshots_override_start() {
        let start = Instant::now();
        let mut clock = Clock::new();
        let playback = Playback {
            start: Some(start),
            elapsed: None,
        };
        assert_eq!(
            clock.elapsed_at(playback, false, true, start + Duration::from_millis(100)),
            100.
        );
        assert_eq!(
            clock.elapsed_at(playback, true, true, start + Duration::from_millis(150)),
            150.
        );
        let snapshot = Playback {
            elapsed: Some(Duration::from_millis(50)),
            ..playback
        };
        assert_eq!(
            clock.elapsed_at(snapshot, false, true, start + Duration::from_secs(5)),
            50.
        );
        let future = Playback {
            start: Some(start + Duration::from_secs(1)),
            elapsed: None,
        };
        assert_eq!(clock.elapsed_at(future, false, false, start), 0.);
    }
}
