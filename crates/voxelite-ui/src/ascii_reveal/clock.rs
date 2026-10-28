// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/voxelite-ui/NOTICE
//! Elapsed-time playback and one-shot repaint scheduling.

use super::skeleton::FPS_MS;
use gpui::{App, Bounds, Context, Entity, Pixels, Task, Window};
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
}
impl Clock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            loading: false,
            timer: None,
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
    }
}

/// Called during prepaint: a clipped element never arms another timer.
pub(crate) fn schedule(
    clock: &Entity<Clock>,
    bounds: Bounds<Pixels>,
    active: bool,
    window: &Window,
    cx: &mut App,
) {
    let visible = bounds.intersects(&window.content_mask().bounds)
        && window.is_visible()
        && window.is_window_active();
    if !visible {
        return;
    }
    clock.update(cx, |clock, cx: &mut Context<Clock>| {
        if !active {
            clock.stop();
            return;
        }
        if clock.timer.is_some() {
            return;
        }
        clock.timer = Some(cx.spawn_in(window, async move |state, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(FPS_MS))
                .await;
            let _ = state.update_in(cx, |clock, window, cx| {
                clock.timer = None;
                if window.is_visible() && window.is_window_active() {
                    cx.notify();
                }
            });
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

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
