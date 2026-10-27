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
// Packed ABI from occupancy.rs: 80-byte header, 16 words per 8³ brick,
// one bit per nonempty brick, then one bit per nonempty 4³-brick super-brick.
// World origin is used in VS only.
struct Occupancy {
    min: vec4<i32>,
    size: vec4<u32>, // xyz extent, w = super-mask word offset
    bricks: vec4<u32>, // xyz dimensions, w = coarse-mask word offset
    box_min: vec4<u32>, // inclusive grow-only occupied cell box (min > max: empty)
    box_max: vec4<u32>,
    words: array<u32>,
};
@group(1) @binding(0) var<storage, read> occupancy: Occupancy;
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color_ao: vec4<f32>,
};
struct Varying {
    @builtin(position) position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) @interpolate(flat) normal: vec3<f32>,
    @location(2) color_ao: vec4<f32>,
    @location(3) local: vec3<f32>,
};
@vertex fn vs_main(v: Vertex) -> Varying {
    var out: Varying;
    out.position = globals.view_projection * vec4(v.position, 1.0);
    out.world = v.position;
    out.local = v.position - vec3<f32>(occupancy.min.xyz);
    out.normal = v.normal;
    out.color_ao = v.color_ao;
    return out;
}
struct Fragment {
    @location(0) color: vec4<f32>,
    @location(1) geometry: vec4<f32>,
};
// Mirror of occupancy.rs. Keep the operation order, f32 precision and XYZ ties.
const SHADOW_EPSILON: f32 = 0.001;
const FAR: f32 = 1.0e30;
fn crossing(origin: f32, direction: f32, cell: i32) -> f32 {
    if direction == 0.0 { return FAR; }
    let boundary = f32(cell + select(0, 1, direction > 0.0));
    var distance = origin - boundary;
    if direction > 0.0 { distance = boundary - origin; }
    if distance > FAR * abs(direction) { return FAR; }
    return distance / abs(direction);
}
fn first_axis(t: vec3<f32>) -> u32 {
    if t.x <= t.y && t.x <= t.z { return 0u; }
    if t.y <= t.z { return 1u; }
    return 2u;
}
fn precedes(t: f32, axis: u32, end: f32, end_axis: u32) -> bool {
    return t < end || (t == end && axis <= end_axis);
}
fn skip_box(origin: vec3<f32>, dir: vec3<f32>, step: vec3<i32>, start: vec3<i32>, edge: i32) -> vec3<i32> {
    var cell = start;
    let lo = (cell / edge) * edge;
    let hi = lo + vec3(edge - 1);
    var end: vec3<f32>;
    for (var a = 0u; a < 3u; a++) {
        end[a] = crossing(origin[a], dir[a], select(lo[a], hi[a], step[a] > 0));
    }
    let axis = first_axis(end);
    let time = end[axis];
    for (var a = 0u; a < 3u; a++) {
        if step[a] == 0 { continue; }
        var candidate = clamp(i32(floor(origin[a] + dir[a] * time)), lo[a], hi[a]);
        while (candidate - cell[a]) * step[a] > 0 && !precedes(crossing(origin[a], dir[a], candidate - step[a]), a, time, axis) {
            candidate -= step[a];
        }
        while candidate >= lo[a] && candidate <= hi[a] && precedes(crossing(origin[a], dir[a], candidate), a, time, axis) {
            candidate += step[a];
        }
        cell[a] = candidate;
    }
    return cell;
}
fn trace_shadow(origin: vec3<f32>, direction: vec3<f32>) -> bool {
    let scale = max(max(abs(direction.x), abs(direction.y)), abs(direction.z));
    if scale == 0.0 { return false; }
    let dir = direction / scale;
    let size = vec3<i32>(occupancy.size.xyz);
    let amin = vec3<i32>(occupancy.box_min.xyz);
    let amax = vec3<i32>(occupancy.box_max.xyz);
    if any(amin > amax) { return false; }
    let supers = (occupancy.bricks.xyz + vec3(3u)) / 4u;
    var cell = vec3<i32>(floor(origin));
    let step = vec3<i32>(sign(dir));
    for (var iteration = 0u; iteration < 768u; iteration++) {
        if any(cell < vec3(0)) || any(cell >= size) { return false; }
        let p = vec3<u32>(cell);
        let b = p / 8u;
        let brick = b.x + occupancy.bricks.x * (b.y + occupancy.bricks.y * b.z);
        let bit = p.x % 8u + 8u * (p.y % 8u) + 64u * (p.z % 8u);
        let word = brick * 16u + bit / 32u;
        let mask = occupancy.bricks.w + brick / 32u;
        // Test the brick first: dense regions pay only the original brick-mask read.
        // The coarse level is consulted only once the brick is known to be empty.
        if (occupancy.words[mask] & (1u << (brick % 32u))) == 0u {
            // Monotonic per axis: past the occupied box in the travel direction means no hit
            // ahead. Checked only in empty bricks so dense regions do not pay for it.
            if any((step >= vec3(0) & cell > amax) | (step <= vec3(0) & cell < amin)) { return false; }
            let s = b / 4u;
            let sup = s.x + supers.x * (s.y + supers.y * s.z);
            let super_mask = occupancy.size.w + sup / 32u;
            if (occupancy.words[super_mask] & (1u << (sup % 32u))) == 0u {
                cell = skip_box(origin, dir, step, cell, 32);
            } else {
                cell = skip_box(origin, dir, step, cell, 8);
            }
        } else {
            if (occupancy.words[word] & (1u << (bit % 32u))) != 0u { return true; }
            var next: vec3<f32>;
            for (var a = 0u; a < 3u; a++) { next[a] = crossing(origin[a], dir[a], cell[a]); }
            let axis = first_axis(next);
            cell[axis] += step[axis];
        }
    }
    return false;
}
@fragment fn fs_main(v: Varying) -> Fragment {
    let diffuse = max(0.0, dot(v.normal, globals.light.xyz));
    var shadow = 1.0;
    if diffuse > 0.0 {
        if trace_shadow(v.local + v.normal * SHADOW_EPSILON, globals.light.xyz) { shadow = 0.2; }
    }
    let illumination = (0.44 + 0.56 * diffuse * shadow) * v.color_ao.a;
    var out: Fragment;
    out.color = vec4(v.color_ao.rgb * illumination, 1.0);
    let face = dot(abs(v.normal), vec3(1.0, 3.0, 5.0)) + max(0.0, dot(v.normal, vec3(1.0)));
    out.geometry = vec4(face, dot(v.world - globals.eye.xyz, globals.forward.xyz), 0.0, 1.0);
    return out;
}
