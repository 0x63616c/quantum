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
@fragment fn fs_main(v: Varying) -> Fragment {
    let diffuse = max(0.0, dot(v.normal, globals.light.xyz));
    let light_position = globals.light_projection * vec4(v.world + v.normal * 0.025, 1.0);
    let uv = light_position.xy * vec2(0.5, -0.5) + vec2(0.5);
    // One hardware comparison, with a constant world-space normal bias. The CPU
    // reference also uses hard directional shadows, not a moving quality level.
    let visibility = textureSampleCompare(shadow_map, shadow_sampler, uv, light_position.z);
    let shadow = select(1.0, 0.2 + 0.8 * visibility, globals.settings.z > 0.5);
    let illumination = (0.44 + 0.56 * diffuse * shadow) * v.color_ao.a;
    var out: Fragment;
    out.color = vec4(v.color_ao.rgb * illumination, 1.0);
    let face = dot(abs(v.normal), vec3(1.0, 3.0, 5.0)) + max(0.0, dot(v.normal, vec3(1.0)));
    out.geometry = vec4(face, dot(v.world - globals.eye.xyz, globals.forward.xyz), 0.0, 1.0);
    return out;
}
