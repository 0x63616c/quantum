//! Cheap spans used by scripts/bench-camera.mjs. No collection without `perf`.
#[cfg(all(feature = "perf", target_family = "wasm"))]
mod browser {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(inline_js = "export function voxelite_perf_now() { return globalThis.__voxelitePerf ? performance.now() : 0; } export function voxelite_perf_event(name, a, b) { globalThis.__voxelitePerf?.event(name, a, b); }")]
    extern "C" {
        pub fn voxelite_perf_now() -> f64;
        pub fn voxelite_perf_event(name: &str, a: f64, b: f64);
    }
}
/// Record a timestamped event when the benchmark collector is installed.
#[inline]
pub fn event(_name: &str, _a: f64, _b: f64) {
    #[cfg(all(feature = "perf", target_family = "wasm"))]
    browser::voxelite_perf_event(_name, _a, _b);
}
/// A CPU interval. Nested intervals are inclusive and must not be summed.
pub struct Span {
    #[cfg(all(feature = "perf", target_family = "wasm"))]
    name: &'static str,
    #[cfg(all(feature = "perf", target_family = "wasm"))]
    start: f64,
}
impl Span {
    /// Begin an interval; a zero-sized no-op in ordinary builds.
    #[inline]
    pub fn new(_name: &'static str) -> Self {
        Self {
            #[cfg(all(feature = "perf", target_family = "wasm"))]
            name: _name,
            #[cfg(all(feature = "perf", target_family = "wasm"))]
            start: browser::voxelite_perf_now(),
        }
    }
}
impl Drop for Span {
    #[inline]
    fn drop(&mut self) {
        #[cfg(all(feature = "perf", target_family = "wasm"))]
        if self.start != 0.0 {
            event(self.name, browser::voxelite_perf_now() - self.start, -1.0);
        }
    }
}
