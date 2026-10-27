struct Globals {
    view_projection: mat4x4<f32>,
    light_projection: mat4x4<f32>,
    light: vec4<f32>, eye: vec4<f32>, forward: vec4<f32>, settings: vec4<f32>,
    background: vec4<f32>,
    inverse_projection: mat4x4<f32>,
    grid: vec4<f32>,
    grid_color: vec4<f32>,
    shadow_settings: vec4<f32>,
};
@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var colors: texture_2d<f32>;
@group(1) @binding(1) var geometry: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}
fn srgb(c:vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(c,vec3(0.0)), vec3(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3(0.0031308));
}
fn backdrop(p: vec2<f32>) -> vec3<f32> {
    let uv = p / globals.settings.xy;
    let ndc = vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let near4 = globals.inverse_projection * vec4(ndc, 0.0, 1.0);
    let far4 = globals.inverse_projection * vec4(ndc, 1.0, 1.0);
    let origin = near4.xyz / near4.w;
    let ray = far4.xyz / far4.w - origin;
    let t = (globals.grid.x - origin.y) / select(0.000001, ray.y, abs(ray.y) > 0.000001);
    let world = (origin + ray * t).xz;
    let cell = (world - globals.grid.y) / max(globals.grid.w, 1.0);
    let edge = abs(cell - round(cell)) / max(fwidth(cell), vec2(0.000001));
    let coverage = 1.0 - smoothstep(0.0, 0.8, min(edge.x, edge.y));
    let visible = globals.grid.w > 0.5 && t >= 0.0 && abs(ray.y) > 0.000001
        && all(world >= vec2(globals.grid.y)) && all(world <= vec2(globals.grid.z));
    return mix(globals.background.rgb, globals.grid_color.rgb,
        select(0.0, coverage * globals.grid_color.a, visible));
}
@fragment fn fs_main(@builtin(position) p:vec4<f32>) -> @location(0) vec4<f32> {
    let background = backdrop(p.xy);
    let at=vec2<i32>(p.xy);
    let color=textureLoad(colors,at,0);
    if color.a == 0.0 { return vec4(background, 1.0); }
    var rgb=color.rgb / color.a; // linear premultiplied MSAA resolve -> straight alpha
    if globals.settings.w > 0.5 {
        let here=textureLoad(geometry,at,0);
        let offsets=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));
        for(var i=0;i<4;i++) {
            let q=clamp(at+offsets[i],vec2(0),vec2<i32>(globals.settings.xy)-vec2(1));
            let other=textureLoad(geometry,q,0);
            if other.a < 0.5 || (abs(other.x-here.x)>0.5 && abs(other.y-here.y)>0.05) {
                rgb*=0.55;break;
            }
        }
    }
    return vec4(mix(background, srgb(rgb), color.a), 1.0);
}
