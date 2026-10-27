//! Test-only f32 transliteration of viewport.wgsl, using its packed storage ABI.
//! Intentionally does not call the host traversal/indexing helpers.
use glam::{IVec3, UVec3, Vec3};
const FAR: f32 = 1.0e30;
fn crossing(origin: f32, direction: f32, cell: i32) -> f32 {
    if direction == 0.0 {
        return FAR;
    }
    let boundary = (cell + if direction > 0.0 { 1 } else { 0 }) as f32;
    let mut distance = origin - boundary;
    if direction > 0.0 {
        distance = boundary - origin;
    }
    if distance > FAR * direction.abs() {
        return FAR;
    }
    distance / direction.abs()
}
fn first_axis(t: Vec3) -> usize {
    if t.x <= t.y && t.x <= t.z {
        return 0;
    }
    if t.y <= t.z {
        return 1;
    }
    2
}
fn precedes(t: f32, axis: usize, end: f32, end_axis: usize) -> bool {
    t < end || (t == end && axis <= end_axis)
}
pub(super) fn trace(data: &[u32], origin: [f32; 3], direction: [f32; 3]) -> bool {
    let origin = Vec3::from_array(origin);
    let direction = Vec3::from_array(direction);
    let scale = direction
        .x
        .abs()
        .max(direction.y.abs())
        .max(direction.z.abs());
    if scale == 0.0 {
        return false;
    }
    let dir = direction / scale;
    let size = UVec3::new(data[4], data[5], data[6]).as_ivec3();
    let bricks = UVec3::new(data[8], data[9], data[10]);
    let words = &data[20..];
    let amin = IVec3::new(data[12] as i32, data[13] as i32, data[14] as i32);
    let amax = IVec3::new(data[16] as i32, data[17] as i32, data[18] as i32);
    if amin.cmpgt(amax).any() {
        return false;
    }
    let mut cell = origin.floor().as_ivec3();
    // WGSL sign(0) == 0 (unlike Rust f32::signum).
    let step = IVec3::from_array(dir.to_array().map(|d| {
        if d > 0.0 {
            1
        } else if d < 0.0 {
            -1
        } else {
            0
        }
    }));
    let supers = (bricks + UVec3::splat(3)) / 4;
    for _iteration in 0..768 {
        if cell.cmplt(IVec3::ZERO).any() || cell.cmpge(size).any() {
            return false;
        }
        let mut past = false;
        for a in 0..3 {
            past |= (step[a] >= 0 && cell[a] > amax[a]) || (step[a] <= 0 && cell[a] < amin[a]);
        }
        if past {
            return false;
        }
        let p = cell.as_uvec3();
        let b = p / 8;
        let brick = b.x + bricks.x * (b.y + bricks.y * b.z);
        let bit = p.x % 8 + 8 * (p.y % 8) + 64 * (p.z % 8);
        let word = brick * 16 + bit / 32;
        let mask = data[11] + brick / 32;
        if words[mask as usize] & (1 << (brick % 32)) == 0 {
            let s = b / 4;
            let sup = s.x + supers.x * (s.y + supers.y * s.z);
            let super_mask = data[7] + sup / 32;
            if words[super_mask as usize] & (1 << (sup % 32)) == 0 {
                cell = skip_box(origin, dir, step, cell, 32);
            } else {
                cell = skip_box(origin, dir, step, cell, 8);
            }
        } else {
            if words[word as usize] & (1 << (bit % 32)) != 0 {
                return true;
            }
            let mut next = Vec3::ZERO;
            for a in 0..3 {
                next[a] = crossing(origin[a], dir[a], cell[a]);
            }
            let axis = first_axis(next);
            cell[axis] += step[axis];
        }
    }
    false
}
fn skip_box(origin: Vec3, dir: Vec3, step: IVec3, mut cell: IVec3, edge: i32) -> IVec3 {
    let lo = (cell / edge) * edge;
    let hi = lo + IVec3::splat(edge - 1);
    let mut end = Vec3::ZERO;
    for a in 0..3 {
        end[a] = crossing(origin[a], dir[a], if step[a] > 0 { hi[a] } else { lo[a] });
    }
    let axis = first_axis(end);
    let time = end[axis];
    for a in 0..3 {
        if step[a] == 0 {
            continue;
        }
        let mut candidate = ((origin[a] + dir[a] * time).floor() as i32).clamp(lo[a], hi[a]);
        while (candidate - cell[a]) * step[a] > 0
            && !precedes(
                crossing(origin[a], dir[a], candidate - step[a]),
                a,
                time,
                axis,
            )
        {
            candidate -= step[a];
        }
        while candidate >= lo[a]
            && candidate <= hi[a]
            && precedes(crossing(origin[a], dir[a], candidate), a, time, axis)
        {
            candidate += step[a];
        }
        cell[a] = candidate;
    }
    cell
}
