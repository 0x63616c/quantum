struct Globals {
    view_projection: mat4x4<f32>,
    light_projection: mat4x4<f32>,
    light: vec4<f32>, eye: vec4<f32>, forward: vec4<f32>, settings: vec4<f32>,
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
@fragment fn fs_main(@builtin(position) p:vec4<f32>) -> @location(0) vec4<f32> {
    let at=vec2<i32>(p.xy);
    let color=textureLoad(colors,at,0);
    if color.a == 0.0 { return vec4(0.0); }
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
    return vec4(srgb(rgb),color.a);
}
