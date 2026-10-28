# GPUI 1.25.3 macOS surface integration patch

`gpui-apple` is copied from crates.io `gpui-apple-gpui-unofficial` 1.25.3,
including its Apache-2.0 license. Cargo patches this crate locally; the other
GPUI packages stay pinned to their registry versions. The `gpui-apple/gpui`
files are upstream's packaged cbindgen inputs, required by the Metal build script.

Only `gpui-apple/src/metal_renderer.rs` changes upstream behavior:

- Retain painted CVPixelBuffers and both CVMetalTexture plane wrappers until the
  consumer command buffer completes. Aborted encodings release their leases.
- Flush cache references after encoding and after completion releases the
  wrappers. A mutex serializes cache creation/flush calls across those threads.
- Cache GPUI's selected macOS Metal device for the process and expose its registry
  ID. The producer selects this actual physical device, including on multi-GPU
  Macs, rather than independently guessing the current power-preference policy.
- Report surface import errors through `take_surface_error()` instead of
  asserting. The single Quantum editor monitors this and switches to CPU.

The producer creates its CVMetalTextureCache with the exact MTLDevice borrowed
from wgpu's HAL. See `crates/quantum-render/src/gpu/mac.rs` for the three-buffer
CoreVideo pool, producer completion fence and imported-plane retirement leases.
Do not remove these consumer leases independently of the producer pool.

Manifests, build scripts, shaders and packaged cbindgen inputs are unchanged.
The matrix regression evaluates the actual WGSL converter against the actual
vendored `shaders.metal` matrix. Re-audit both lifetime and color contracts when
upgrading GPUI. No web or wgpu GPUI fork is required for the Mac-only viewport.

Apple documents the required [texture-wrapper lifetime](https://developer.apple.com/documentation/corevideo/cvmetaltexturecachecreatetexturefromimage(_:_:_:_:_:_:_:_:_:))
and [cache flushing](https://developer.apple.com/documentation/corevideo/cvmetaltexturecacheflush(_:_:)).
