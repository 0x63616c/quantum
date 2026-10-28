//! Rolling timing of newly painted viewport content, independent of render cost.
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(1);

/// Backend that produced the viewport content (not the UI compositor).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererKind {
    /// Software voxel renderer.
    Cpu,
    /// GPU voxel renderer.
    Gpu,
}

impl RendererKind {
    /// Compact status-bar label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Gpu => "GPU",
        }
    }
}

/// Averages over the same set of complete presentation intervals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameAverage {
    /// Newly painted viewport frames per second.
    pub fps: f64,
    /// Mean time between newly painted viewport frames, in milliseconds.
    pub frame_ms: f64,
}

/// Measures roughly the last second of newly painted viewport frames.
///
/// Call only after a successful paint of visible viewport content, with a stable
/// ID for each image (or a new serial for each GPU surface update). Repainting
/// the same content does not advance the clock. Keep the interval crossing the
/// window boundary so low or uneven frame rates do not bias the average upward.
/// Call [`Self::begin_frame`] when starting work from idle to exclude long idle
/// pauses without excluding slow renders. The last average is held until two
/// new frames arrive. Reading the meter never decays it or requires a timer.
#[derive(Debug)]
pub struct FrameMeter {
    renderer: RendererKind,
    last_frame: Option<usize>,
    timestamps: VecDeque<Instant>,
    average: Option<FrameAverage>,
}

impl FrameMeter {
    /// Start an empty meter for the selected backend.
    pub fn new(renderer: RendererKind) -> Self {
        Self {
            renderer,
            last_frame: None,
            timestamps: VecDeque::new(),
            average: None,
        }
    }

    /// Backend of the last newly painted frame, or the initial backend.
    pub fn renderer(&self) -> RendererKind {
        self.renderer
    }

    /// Latest measured average; `None` until two distinct frames are timed.
    pub fn average(&self) -> Option<FrameAverage> {
        self.average
    }

    /// Start viewport work after being idle (not when continuing queued work).
    ///
    /// Forget the sampling window if the last presentation was over a second
    /// ago, holding the previous average through warmup. This does not count a
    /// frame. Check idle time here, before rendering, so genuinely slow frames
    /// still contribute their full presentation interval.
    pub fn begin_frame(&mut self, at: Instant) {
        if self
            .timestamps
            .back()
            .is_some_and(|last| at.saturating_duration_since(*last) > WINDOW)
        {
            self.timestamps.clear();
        }
    }

    /// Record a successful viewport paint at an injected monotonic timestamp.
    ///
    /// Returns whether new content was accepted. Duplicate content and backward
    /// timestamps are ignored. Coincident timestamps are coalesced to avoid
    /// zero-duration intervals. Switching backends resets the sample window.
    pub fn record_presented(
        &mut self,
        renderer: RendererKind,
        frame_id: usize,
        at: Instant,
    ) -> bool {
        if self.timestamps.back().is_some_and(|last| at < *last) {
            return false;
        }
        if self.renderer != renderer {
            *self = Self::new(renderer);
        }
        if self.last_frame == Some(frame_id) {
            return false;
        }
        self.last_frame = Some(frame_id);
        if self.timestamps.back() == Some(&at) {
            return true;
        }
        self.timestamps.push_back(at);
        while self.timestamps.len() > 2 && at - self.timestamps[1] >= WINDOW {
            self.timestamps.pop_front();
        }
        if self.timestamps.len() >= 2 {
            let seconds = (at - self.timestamps[0]).as_secs_f64();
            let intervals = (self.timestamps.len() - 1) as f64;
            self.average = Some(FrameAverage {
                fps: intervals / seconds,
                frame_ms: seconds * 1000.0 / intervals,
            });
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_average(meter: &FrameMeter, fps: f64, ms: f64) {
        let average = meter.average().expect("two timed frames");
        assert!((average.fps - fps).abs() < 1e-9, "{average:?}");
        assert!((average.frame_ms - ms).abs() < 1e-9, "{average:?}");
    }

    #[test]
    fn warms_up_then_measures_regular_presentation_intervals() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(RendererKind::Cpu);
        assert_eq!(meter.average(), None);
        assert!(meter.record_presented(RendererKind::Cpu, 0, start));
        assert_eq!(meter.average(), None);
        for frame in 1..=200 {
            meter.record_presented(
                RendererKind::Cpu,
                frame,
                start + Duration::from_millis(frame as u64 * 20),
            );
        }
        assert_average(&meter, 50.0, 20.0);
        assert_eq!(meter.timestamps.len(), 51);
    }

    #[test]
    fn averages_elapsed_intervals_instead_of_instantaneous_fps() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(RendererKind::Cpu);
        for (frame, ms) in [0, 10, 40, 100].into_iter().enumerate() {
            meter.record_presented(RendererKind::Cpu, frame, start + Duration::from_millis(ms));
        }
        assert_average(&meter, 30.0, 100.0 / 3.0);
    }

    #[test]
    fn rolls_the_window_and_keeps_the_interval_crossing_its_boundary() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(RendererKind::Cpu);
        for (frame, ms) in [0, 200, 500, 900, 1_100].into_iter().enumerate() {
            meter.record_presented(RendererKind::Cpu, frame, start + Duration::from_millis(ms));
        }
        assert_average(&meter, 4.0 / 1.1, 275.0);
        meter.record_presented(RendererKind::Cpu, 5, start + Duration::from_millis(1_200));
        assert_average(&meter, 4.0, 250.0);
        meter.record_presented(RendererKind::Cpu, 6, start + Duration::from_millis(1_800));
        assert_average(&meter, 4.0 / 1.3, 325.0);
    }

    #[test]
    fn unchanged_repaints_neither_count_nor_shift_the_last_presentation() {
        for renderer in [RendererKind::Cpu, RendererKind::Gpu] {
            let start = Instant::now();
            let mut meter = FrameMeter::new(renderer);
            meter.record_presented(renderer, 1, start);
            for ms in [5, 10, 15] {
                assert!(!meter.record_presented(renderer, 1, start + Duration::from_millis(ms)));
            }
            assert_eq!(meter.average(), None);
            // Skipped IDs represent completed renders that were never painted.
            meter.record_presented(renderer, 4, start + Duration::from_millis(20));
            assert_average(&meter, 50.0, 20.0);
            for ms in [40, 1_000, 10_000] {
                assert!(!meter.record_presented(renderer, 4, start + Duration::from_millis(ms)));
                assert_average(&meter, 50.0, 20.0);
            }
            // A long idle gap is excluded; the held value lasts through warmup.
            meter.begin_frame(start + Duration::from_secs(19));
            assert_average(&meter, 50.0, 20.0);
            // The old image may still be repainted while new work is in flight.
            assert!(!meter.record_presented(renderer, 4, start + Duration::from_secs(19)));
            meter.record_presented(renderer, 5, start + Duration::from_secs(20));
            assert_average(&meter, 50.0, 20.0);
            meter.record_presented(renderer, 6, start + Duration::from_millis(20_040));
            assert_average(&meter, 25.0, 40.0);
        }
    }

    #[test]
    fn handles_equal_and_backward_timestamps_without_invalid_averages() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(RendererKind::Cpu);
        meter.record_presented(RendererKind::Cpu, 0, start);
        meter.record_presented(RendererKind::Cpu, 1, start);
        assert_eq!(meter.average(), None);
        meter.record_presented(RendererKind::Cpu, 2, start + Duration::from_millis(20));
        assert!(!meter.record_presented(RendererKind::Cpu, 3, start + Duration::from_millis(10)));
        assert_average(&meter, 50.0, 20.0);
        meter.record_presented(RendererKind::Cpu, 3, start + Duration::from_millis(40));
        assert_average(&meter, 50.0, 20.0);
    }

    #[test]
    fn one_second_intervals_are_still_measured() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(RendererKind::Cpu);
        for frame in 0..4 {
            meter.record_presented(
                RendererKind::Cpu,
                frame,
                start + Duration::from_secs(frame as u64),
            );
        }
        assert_average(&meter, 1.0, 1_000.0);
    }

    #[test]
    fn slow_rendering_is_not_mistaken_for_idle_time() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(RendererKind::Cpu);
        meter.begin_frame(start);
        assert_eq!(meter.average(), None);
        meter.record_presented(RendererKind::Cpu, 0, start + Duration::from_secs(2));
        meter.begin_frame(start + Duration::from_millis(2_010));
        meter.record_presented(RendererKind::Cpu, 1, start + Duration::from_secs(4));
        assert_average(&meter, 0.5, 2_000.0);
        // Queued work continues without an idle reset, even if its previous
        // image has not been painted yet when the next render starts.
        meter.record_presented(RendererKind::Cpu, 2, start + Duration::from_secs(6));
        assert_average(&meter, 0.5, 2_000.0);
    }

    #[test]
    fn renderer_switch_resets_samples_even_if_frame_ids_match() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(RendererKind::Cpu);
        assert_eq!(meter.renderer().label(), "CPU");
        meter.record_presented(RendererKind::Cpu, 0, start);
        meter.record_presented(RendererKind::Cpu, 1, start + Duration::from_millis(20));
        assert!(meter.record_presented(RendererKind::Gpu, 1, start + Duration::from_secs(1)));
        assert_eq!(meter.renderer().label(), "GPU");
        assert_eq!(meter.average(), None);
        meter.record_presented(RendererKind::Gpu, 2, start + Duration::from_millis(1_010));
        assert_average(&meter, 100.0, 10.0);
        // A fallback starts a new CPU window even if its image ID overlaps a
        // surface serial. Subsequent CPU paints must not inherit GPU timing.
        assert!(meter.record_presented(RendererKind::Cpu, 2, start + Duration::from_secs(2)));
        assert_eq!(meter.renderer().label(), "CPU");
        assert_eq!(meter.average(), None);
        meter.record_presented(RendererKind::Cpu, 3, start + Duration::from_millis(2_100));
        assert_average(&meter, 10.0, 100.0);
    }
}
