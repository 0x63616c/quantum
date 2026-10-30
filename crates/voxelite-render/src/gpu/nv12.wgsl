// Exact inverse of GPUI 1.25.3 shaders.metal's full-range BT.601 matrix.
// GPUI: R=Y+1.402*Cr-.701, G=Y-.3441*Cb-.7141*Cr+.5291,
//       B=Y+1.772*Cb-.886. Chroma is centered at .5 (no video-range scaling).
@group(0) @binding(0) var source: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    return vec4(f32((i << 1u) & 2u) * 2.0 - 1.0, f32(i & 2u) * 2.0 - 1.0, 0.0, 1.0);
}
fn ycbcr(rgb: vec3<f32>) -> vec3<f32> {
    let cr_weight = 0.7141 / 1.402;
    let cb_weight = 0.3441 / 1.772;
    let y = (rgb.g + cr_weight * rgb.r + cb_weight * rgb.b) / (1.0 + cr_weight + cb_weight);
    return vec3(y, (rgb.b - y) / 1.772 + 0.5, (rgb.r - y) / 1.402 + 0.5);
}
@fragment fn fs_y(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(ycbcr(textureLoad(source, vec2<i32>(p.xy), 0).rgb).x, 0.0, 0.0, 1.0);
}
@fragment fn fs_chroma(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    // Center chroma over the corresponding 2x2 luma block. The source scene
    // normally has twice the physical display dimensions; budget-limited
    // surfaces use fewer samples. Average nonlinear RGB before the affine
    // conversion (equivalent to averaging CbCr), matching GPUI's sampler.
    let at = vec2<i32>(p.xy) * 2;
    let rgb = (textureLoad(source, at, 0).rgb
        + textureLoad(source, at + vec2(1, 0), 0).rgb
        + textureLoad(source, at + vec2(0, 1), 0).rgb
        + textureLoad(source, at + vec2(1, 1), 0).rgb) * 0.25;
    return vec4(ycbcr(rgb).yz, 0.0, 1.0);
}
