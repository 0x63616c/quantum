struct Globals {
    view_projection: mat4x4<f32>,
    light_projection: mat4x4<f32>,
    light: vec4<f32>,
    eye: vec4<f32>,
    forward: vec4<f32>,
    settings: vec4<f32>, // width, height, shadows, outlines
    background: vec4<f32>,
    inverse_projection: mat4x4<f32>,
    grid: vec4<f32>,
    grid_color: vec4<f32>,
    shadow_settings: vec4<f32>, // inverse map size, world-space normal offset, reserved
};
@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var shadow_map: texture_depth_2d;
@group(1) @binding(1) var shadow_sampler: sampler_comparison;
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color_ao: vec4<f32>,
};
struct Varying {
    @builtin(position) position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color_ao: vec4<f32>,
};
@vertex fn vs_main(v: Vertex) -> Varying {
    var out: Varying;
    out.position = globals.view_projection * vec4(v.position, 1.0);
    out.world = v.position;
    out.normal = v.normal;
    out.color_ao = v.color_ao;
    return out;
}
@vertex fn vs_shadow(v: Vertex) -> @builtin(position) vec4<f32> {
    return globals.light_projection * vec4(v.position, 1.0);
}
struct Fragment {
    @location(0) color: vec4<f32>,
    @location(1) geometry: vec4<f32>,
};
// Orthographic light projection rows are orthogonal but have different scales.
// Transform the receiver plane's normal by the inverse transpose, then solve
// its depth slope in texture UV (including the flipped Y axis).
fn shadow_depth_gradient(normal: vec3<f32>, m: mat4x4<f32>) -> vec2<f32> {
    let x = vec3(m[0].x, m[1].x, m[2].x);
    let y = vec3(m[0].y, m[1].y, m[2].y);
    let z = vec3(m[0].z, m[1].z, m[2].z);
    let plane = vec3(dot(normal, x) / dot(x, x),
        dot(normal, y) / dot(y, y), dot(normal, z) / dot(z, z));
    return -plane.xy / plane.z * vec2(2.0, -2.0);
}
fn shadow_filter_depth(center: f32, gradient: vec2<f32>, offset_uv: vec2<f32>, texel: f32) -> f32 {
    // Bilinear comparison uses four texel centers, each up to one texel from
    // the tap. Cover their receiver-depth variation independently of the caster
    // slope bias (the footprint can cross onto a differently oriented face).
    let footprint_bias = dot(abs(gradient), vec2(texel));
    return center + dot(gradient, offset_uv) - footprint_bias;
}
fn shadow_visibility(world: vec3<f32>, normal: vec3<f32>, diffuse: f32) -> f32 {
    if globals.settings.z < 0.5 || diffuse <= 0.0 { return 1.0; }
    // Bias follows the fitted texel size and surface slope, so it shrinks when
    // zooming in instead of detaching close-up contact shadows from the voxels.
    let offset = globals.shadow_settings.y * sqrt(max(0.0, 1.0 - diffuse * diffuse));
    let position = globals.light_projection * vec4(world + normal * offset, 1.0);
    let uv = position.xy * vec2(0.5, -0.5) + vec2(0.5);
    if any(uv < vec2(0.0)) || any(uv > vec2(1.0)) || position.z < 0.0 || position.z > 1.0 {
        return 1.0;
    }
    // Nine bilinear hardware comparisons. Explicit LOD permits the early-outs
    // above without derivative-uniformity issues on Metal or in WGSL validation.
    let gradient = shadow_depth_gradient(normal, globals.light_projection);
    var visibility = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset_uv = vec2(f32(x), f32(y)) * globals.shadow_settings.x;
            // Each tap lies at a different depth on a sloping receiver. Using
            // the center depth for every tap causes periodic self-shadow bands;
            // raster bias alone does not cover the diagonal PCF footprint.
            let depth = shadow_filter_depth(position.z, gradient, offset_uv, globals.shadow_settings.x);
            visibility += textureSampleCompareLevel(shadow_map, shadow_sampler,
                uv + offset_uv, depth);
        }
    }
    return visibility / 9.0;
}
fn surface_illumination(v: Varying) -> f32 {
    // Same ambient-only plan as the exact-shadow shader; no shadow sampling.
    if globals.light.w != 0.0 { return 0.44; }
    let diffuse = max(0.0, dot(v.normal, globals.light.xyz));
    let shadow = 0.2 + 0.8 * shadow_visibility(v.world, v.normal, diffuse);
    return (0.44 + 0.56 * diffuse * shadow) * v.color_ao.a;
}
@fragment fn fs_main(v: Varying) -> Fragment {
    let illumination = surface_illumination(v);
    var out: Fragment;
    out.color = vec4(v.color_ao.rgb * illumination, 1.0);
    let face = dot(abs(v.normal), vec3(1.0, 3.0, 5.0)) + max(0.0, dot(v.normal, vec3(1.0)));
    out.geometry = vec4(face, dot(v.world - globals.eye.xyz, globals.forward.xyz), 0.0, 1.0);
    return out;
}
