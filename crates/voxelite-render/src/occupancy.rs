//! Shared exact shadow grid. Bricks contain 8³ bits (16 contiguous u32 words).
//! A second bitset marks nonempty bricks. The packed buffer is also the GPU ABI.
use crate::RenderResult;
use std::{collections::BTreeSet, ops::Range};
use voxelite_core::{Pos, Region, Snapshot};

/// Surface offset in voxel units, applied AFTER conversion to grid-local f32.
pub const SHADOW_EPSILON: f32 = 0.001;
const HEADER_WORDS: usize = 12;
const BRICK_WORDS: usize = 16;
const FAR: f32 = 1.0e30;

/// Visible occupancy, independent of palette and layer order. Coordinates passed
/// to editing methods are world cells; ray origins are relative to `bounds.min`.
#[derive(Clone, Debug)]
pub struct Occupancy {
    bounds: Region,
    bricks: [usize; 3],
    data: Vec<u32>,
    counts: Vec<u16>,
    dirty: BTreeSet<usize>,
    full_upload: bool,
}
impl Occupancy {
    /// Allocate an empty grid. Bounds must obey the document's 256³ limit.
    pub fn new(bounds: Region) -> RenderResult<Self> {
        bounds.validate()?;
        let size = std::array::from_fn::<_, 3, _>(|a| (bounds.max[a] - bounds.min[a]) as usize);
        let bricks = size.map(|n| n.div_ceil(8));
        let count = bricks.iter().product::<usize>();
        let mut data = vec![0; HEADER_WORDS + count * BRICK_WORDS + count.div_ceil(32)];
        for a in 0..3 {
            data[a] = bounds.min[a] as u32;
            data[4 + a] = size[a] as u32;
            data[8 + a] = bricks[a] as u32;
        }
        data[11] = (count * BRICK_WORDS) as u32; // mask offset, relative to words
        Ok(Self {
            bounds,
            bricks,
            data,
            counts: vec![0; count],
            dirty: BTreeSet::new(),
            full_upload: true,
        })
    }
    /// Build the occupancy of the composited visible layers. Overlaps are a
    /// boolean union: all nonzero palette slots are opaque in both renderers.
    pub fn from_snapshot(snapshot: &Snapshot) -> RenderResult<Self> {
        let mut grid = Self::new(snapshot.bounds)?;
        for layer in snapshot.layers.iter().filter(|l| l.visible) {
            for (p, _) in layer.voxels.iter() {
                grid.set(p, true);
            }
        }
        Ok(grid)
    }
    /// Half-open world bounds.
    pub fn bounds(&self) -> Region {
        self.bounds
    }
    /// GPU storage ABI: min:i32x4, size:u32x4, brick dimensions + mask offset:
    /// u32x4, then brick-local occupancy words and the coarse brick bitset.
    pub fn words(&self) -> &[u32] {
        &self.data
    }
    fn address(&self, p: [i32; 3]) -> (usize, usize, u32) {
        let p = p.map(|v| v as usize);
        let brick = p[0] / 8 + self.bricks[0] * (p[1] / 8 + self.bricks[1] * (p[2] / 8));
        let bit = (p[0] % 8 + 8 * (p[1] % 8) + 64 * (p[2] % 8)) as u32;
        (
            brick,
            HEADER_WORDS + brick * BRICK_WORDS + bit as usize / 32,
            1 << (bit % 32),
        )
    }
    /// Whether a world cell is occupied; outside cells are empty.
    pub fn contains(&self, p: Pos) -> bool {
        if !self.bounds.contains(p) {
            return false;
        }
        let (_, word, bit) = self.address(std::array::from_fn(|a| p[a] - self.bounds.min[a]));
        self.data[word] & bit != 0
    }
    /// Apply one composited cell change. Returns whether occupancy changed.
    /// Recoloring or replacing an overlapped cell does not dirty the GPU buffer.
    pub fn set(&mut self, p: Pos, occupied: bool) -> bool {
        if !self.bounds.contains(p) {
            return false;
        }
        let (brick, word, bit) = self.address(std::array::from_fn(|a| p[a] - self.bounds.min[a]));
        if (self.data[word] & bit != 0) == occupied {
            return false;
        }
        let mask_word = HEADER_WORDS + self.counts.len() * BRICK_WORDS + brick / 32;
        if occupied {
            self.data[word] |= bit;
            self.counts[brick] += 1;
            self.data[mask_word] |= 1 << (brick % 32);
        } else {
            self.data[word] &= !bit;
            self.counts[brick] -= 1;
            if self.counts[brick] == 0 {
                self.data[mask_word] &= !(1 << (brick % 32));
            }
        }
        // A pending full upload already includes every brick. Avoid building a
        // redundant BTreeSet for bulk construction or repeated pre-upload edits.
        if !self.full_upload {
            self.dirty.insert(brick);
        }
        true
    }
    /// Refresh only cells in a changed region, clipping to the document bounds.
    pub fn update_region(&mut self, snapshot: &Snapshot, region: Region) {
        let min = std::array::from_fn(|a| region.min[a].max(self.bounds.min[a]));
        let max = std::array::from_fn(|a| region.max[a].min(self.bounds.max[a]));
        for p in (Region { min, max }).cells() {
            self.set(p, snapshot.sample(p).is_some());
        }
    }
    /// Coalesced dirty u32 ranges, including their coarse mask words. The first
    /// upload (and any resize) covers the header and entire allocation.
    /// Acknowledge only after the upload succeeds using `clear_dirty`.
    pub fn dirty_ranges(&self) -> Vec<Range<usize>> {
        if self.full_upload {
            return std::iter::once(0..self.data.len()).collect();
        }
        let mask = HEADER_WORDS + self.counts.len() * BRICK_WORDS;
        let mut ranges: Vec<_> = self
            .dirty
            .iter()
            .map(|&b| HEADER_WORDS + b * BRICK_WORDS..HEADER_WORDS + (b + 1) * BRICK_WORDS)
            .collect();
        ranges.extend(self.dirty.iter().map(|&b| mask + b / 32..mask + b / 32 + 1));
        ranges.sort_unstable_by_key(|r| r.start);
        let mut merged: Vec<Range<usize>> = Vec::new();
        for r in ranges {
            if let Some(last) = merged.last_mut().filter(|last| last.end >= r.start) {
                last.end = last.end.max(r.end);
            } else {
                merged.push(r);
            }
        }
        merged
    }
    /// Acknowledge successful upload of all pending ranges.
    pub fn clear_dirty(&mut self) {
        self.full_upload = false;
        self.dirty.clear();
    }
}

/// Persistent snapshot cache. Unchanged snapshots reuse the grid; the GPU worker
/// can skip even this synchronization on camera-only frames.
#[derive(Default)]
pub struct OccupancyCache {
    source: Option<Snapshot>,
    grid: Option<Occupancy>,
}
impl OccupancyCache {
    /// Synchronize immutable chunks and layer membership. Only actual changed
    /// cells are sampled; shared chunks and hidden-layer edits are skipped.
    pub fn sync(&mut self, snapshot: &Snapshot) -> RenderResult<()> {
        if self
            .grid
            .as_ref()
            .is_none_or(|g| g.bounds != snapshot.bounds)
        {
            self.grid = Some(Occupancy::from_snapshot(snapshot)?);
        } else if let Some(old) = &self.source {
            let mut changed = BTreeSet::new();
            for layer in snapshot.layers.iter().filter(|l| l.visible) {
                if let Some(previous) = old.layers.iter().find(|l| l.id == layer.id && l.visible) {
                    changed.extend(
                        layer
                            .voxels
                            .changed_cells(&previous.voxels)
                            .into_iter()
                            .map(|(p, _, _)| p),
                    );
                } else {
                    changed.extend(layer.voxels.iter().map(|(p, _)| p));
                }
            }
            for previous in old.layers.iter().filter(|l| l.visible) {
                if !snapshot
                    .layers
                    .iter()
                    .any(|l| l.id == previous.id && l.visible)
                {
                    changed.extend(previous.voxels.iter().map(|(p, _)| p));
                }
            }
            let grid = self.grid.as_mut().unwrap();
            for p in changed {
                grid.set(p, snapshot.sample(p).is_some());
            }
        }
        self.source = Some(snapshot.clone());
        Ok(())
    }
    /// Current grid, absent until the first snapshot.
    pub fn grid(&self) -> Option<&Occupancy> {
        self.grid.as_ref()
    }
    /// Current grid, for acknowledging uploads.
    pub fn grid_mut(&mut self) -> Option<&mut Occupancy> {
        self.grid.as_mut()
    }
}

// The following traversal is mirrored in gpu/viewport.wgsl. Use f32 throughout;
// calculate crossing times from the ORIGINAL origin, never accumulated deltas.
// That keeps long rays and coarse skips on the very same floating-point lattice.
fn crossing(origin: f32, direction: f32, cell: i32) -> f32 {
    if direction == 0.0 {
        return FAR;
    }
    let boundary = (cell + i32::from(direction > 0.0)) as f32;
    let distance = if direction > 0.0 {
        boundary - origin
    } else {
        origin - boundary
    };
    // Avoid infinity/NaN even for a subnormal component (WGSL finite math).
    if distance > FAR * direction.abs() {
        return FAR;
    }
    distance / direction.abs()
}
fn first_axis(t: [f32; 3]) -> usize {
    if t[0] <= t[1] && t[0] <= t[2] {
        0
    } else if t[1] <= t[2] {
        1
    } else {
        2
    }
}
fn precedes(t: f32, axis: usize, end: f32, end_axis: usize) -> bool {
    t < end || (t == end && axis <= end_axis)
}
/// Exact hard-shadow reference, with a coarse empty-brick DDA. `origin` is
/// grid-local (`surface - bounds.min + axis_normal * SHADOW_EPSILON`); `dir` is
/// finite and nonzero. Outside starts return false, as do rays exiting bounds.
/// On an exact tie visit X, then Y, then Z, including zero-length touched cells.
/// No slab entry is needed for outward-facing surface-to-sun rays.
pub fn trace_shadow(occ: &Occupancy, origin: [f32; 3], dir: [f32; 3]) -> bool {
    trace(occ, origin, dir, true)
}
fn trace(occ: &Occupancy, origin: [f32; 3], dir: [f32; 3], skip: bool) -> bool {
    if origin.iter().chain(&dir).any(|v| !v.is_finite()) {
        return false;
    }
    let scale = dir[0].abs().max(dir[1].abs()).max(dir[2].abs());
    if scale == 0.0 {
        return false;
    }
    let dir = dir.map(|v| v / scale);
    let size = std::array::from_fn::<_, 3, _>(|a| occ.bounds.max[a] - occ.bounds.min[a]);
    let mut cell = origin.map(|v| v.floor() as i32);
    let step = dir.map(|v| {
        if v > 0.0 {
            1
        } else if v < 0.0 {
            -1
        } else {
            0
        }
    });
    // Each iteration advances at least one axis; a 256³ grid needs <=768.
    for _ in 0..768 {
        if (0..3).any(|a| cell[a] < 0 || cell[a] >= size[a]) {
            return false;
        }
        let (brick, word, bit) = occ.address(cell);
        let mask = HEADER_WORDS + occ.counts.len() * BRICK_WORDS + brick / 32;
        if skip && occ.data[mask] & (1 << (brick % 32)) == 0 {
            let lo = cell.map(|v| (v / 8) * 8);
            let hi = lo.map(|v| v + 7);
            let end = std::array::from_fn(|a| {
                crossing(origin[a], dir[a], if step[a] > 0 { hi[a] } else { lo[a] })
            });
            let axis = first_axis(end);
            let time = end[axis];
            for a in 0..3 {
                if step[a] == 0 {
                    continue;
                }
                // Jump to the exit event. Correct rounding with the SAME crossing
                // comparisons used by the fine DDA, including pending XYZ ties.
                let mut candidate =
                    ((origin[a] + dir[a] * time).floor() as i32).clamp(lo[a], hi[a]);
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
        } else {
            if occ.data[word] & bit != 0 {
                return true;
            }
            let next = std::array::from_fn(|a| crossing(origin[a], dir[a], cell[a]));
            let axis = first_axis(next);
            cell[axis] += step[axis];
        }
    }
    false
}

#[cfg(test)]
#[path = "gpu/trace_reference.rs"]
mod shader_reference;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn grid(size: i32, cells: &[[i32; 3]]) -> Occupancy {
        let mut grid = Occupancy::new(Region::new([0; 3], [size; 3]).unwrap()).unwrap();
        for &p in cells {
            grid.set(p, true);
        }
        grid
    }
    fn check(occ: &Occupancy, origin: [f32; 3], dir: [f32; 3], expected: bool) {
        if origin.iter().all(|v| v.is_finite()) {
            assert_eq!(
                shader_reference::trace(occ.words(), origin, dir),
                expected,
                "shader: {origin:?} {dir:?}"
            );
        }
        assert_eq!(
            trace_shadow(occ, origin, dir),
            expected,
            "{origin:?} {dir:?}"
        );
        assert_eq!(
            trace(occ, origin, dir, false),
            expected,
            "fine: {origin:?} {dir:?}"
        );
    }
    #[test]
    fn axes_adjacent_boundaries_and_exits() {
        for axis in 0..3 {
            for sign in [-1, 1] {
                let mut dir = [0.0; 3];
                dir[axis] = sign as f32;
                let mut blocker = [8; 3];
                blocker[axis] += sign;
                check(&grid(17, &[blocker]), [8.5; 3], dir, true);
                blocker[axis] = if sign > 0 { 16 } else { 0 };
                check(&grid(17, &[blocker]), [8.5; 3], dir, true);
                check(&grid(17, &[]), [8.5; 3], dir, false);
                let mut origin = [8.5; 3];
                origin[axis] = if sign > 0 { 17.0 } else { -SHADOW_EPSILON };
                check(&grid(17, &[blocker]), origin, dir, false);
                origin[axis] = 8.0;
                blocker[axis] = if sign > 0 { 8 } else { 7 };
                check(&grid(17, &[blocker]), origin, dir, true);
                // Exactly parallel and almost parallel components cannot cause NaN.
                for grazing in [
                    0.0,
                    -0.0,
                    1e-8,
                    -1e-8,
                    f32::MIN_POSITIVE,
                    -f32::MIN_POSITIVE,
                ] {
                    dir[(axis + 1) % 3] = grazing;
                    check(&grid(17, &[blocker]), origin, dir, true);
                }
            }
        }
        check(&grid(8, &[[0; 3]]), [0.0; 3], [0.0; 3], false);
        check(&grid(8, &[]), [f32::NAN; 3], [1.0; 3], false);
    }
    #[test]
    fn xyz_ties_include_touched_cells_and_survive_brick_skips() {
        // At the first corner crossing, X's cell is tested before Y or Z.
        for p in [[8, 7, 7], [8, 8, 7], [8, 8, 8]] {
            check(&grid(24, &[p]), [0.5; 3], [1.0; 3], true);
        }
        for p in [[7, 8, 7], [7, 7, 8], [7, 8, 8]] {
            check(&grid(24, &[p]), [0.5; 3], [1.0; 3], false);
        }
        for p in [[15, 16, 16], [15, 15, 16], [15, 15, 15]] {
            check(&grid(24, &[p]), [23.5; 3], [-1.0; 3], true);
        }
        // Opposite signs, exact zero-time crossings, and maximum-length rays.
        check(&grid(256, &[[255; 3]]), [0.0; 3], [1.0; 3], true);
        check(&grid(256, &[[0; 3]]), [255.999; 3], [-1.0; 3], true);
        check(&grid(24, &[[7, 8, 8]]), [8.0; 3], [-1.0, 1.0, 1.0], true);
    }
    #[test]
    fn epsilon_is_local_even_at_extreme_world_coordinates() {
        for min in [-32768, -9, 0, 32512] {
            let bounds = Region::new([min; 3], [min + 256; 3]).unwrap();
            let mut occ = Occupancy::new(bounds).unwrap();
            occ.set([min + 128; 3], true);
            for axis in 0..3 {
                for sign in [-1.0, 1.0] {
                    let mut surface = [128.5; 3];
                    surface[axis] = if sign > 0.0 { 129.0 } else { 128.0 };
                    let mut dir = [0.0; 3];
                    dir[axis] = sign;
                    surface[axis] += sign * SHADOW_EPSILON;
                    check(&occ, surface, dir, false);
                }
            }
        }
    }
    #[test]
    fn uploads_cover_changed_bricks_and_masks_only() {
        let mut occ = grid(256, &[]);
        assert_eq!(occ.words().len() * 4, 48 + 2_097_152 + 4096);
        occ.clear_dirty();
        assert!(occ.set([0; 3], true));
        assert!(!occ.set([0; 3], true));
        assert_eq!(occ.dirty_ranges(), vec![12..28, 524300..524301]);
        occ.set([8, 0, 0], true);
        assert_eq!(occ.dirty_ranges(), vec![12..44, 524300..524301]);
        occ.clear_dirty();
        occ.set([0; 3], false);
        assert_eq!(occ.data[524300] & 3, 2);
    }
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2048))]
        #[test]
        fn coarse_equals_fine(
            size in prop::array::uniform3(1i32..=64),
            cells in prop::collection::vec(prop::array::uniform3(0i32..64), 0..160),
            origin in prop::array::uniform3(-1.0f32..65.0),
            dir in prop::array::uniform3(-1.0f32..1.0),
        ) {
            let mut occ = Occupancy::new(Region::new([0; 3], size).unwrap()).unwrap();
            for &p in &cells { occ.set(p, true); }
            prop_assert_eq!(trace_shadow(&occ, origin, dir), trace(&occ, origin, dir, false));
            prop_assert_eq!(shader_reference::trace(occ.words(), origin, dir), trace_shadow(&occ, origin, dir));
            // Rays toward known occupied cells make hits common even on sparse grids.
            for &p in cells.iter().take(12) {
                let target = p.map(|v| v as f32 + 0.5);
                let dir = std::array::from_fn(|a| target[a] - origin[a]);
                prop_assert_eq!(trace_shadow(&occ, origin, dir), trace(&occ, origin, dir, false));
                prop_assert_eq!(shader_reference::trace(occ.words(), origin, dir), trace_shadow(&occ, origin, dir));
            }
        }
        #[test]
        fn rays_to_a_voxel_center_always_hit(
            size in prop::array::uniform3(1i32..=256),
            source in prop::array::uniform3(0i32..256),
            target in prop::array::uniform3(0i32..256),
        ) {
            let source: [i32; 3] = std::array::from_fn(|a| source[a] % size[a]);
            let target: [i32; 3] = std::array::from_fn(|a| target[a] % size[a]);
            let min = [-32768, -19, 32512];
            let bounds = Region::new(min, std::array::from_fn(|a| min[a] + size[a])).unwrap();
            let mut occ = Occupancy::new(bounds).unwrap();
            occ.set(std::array::from_fn(|a| min[a] + target[a]), true);
            let origin = source.map(|v| v as f32 + 0.25);
            let dir = std::array::from_fn(|a| target[a] as f32 + 0.5 - origin[a]);
            check(&occ, origin, dir, true);
        }
        #[test]
        fn lattice_ties_and_grazing(
            origin in prop::array::uniform3(0i32..32),
            dir in prop::array::uniform3(-2i32..=2),
            cells in prop::collection::vec(prop::array::uniform3(0i32..32), 0..100),
        ) {
            let occ = grid(32, &cells);
            for shift in [0.0, 0.001, 0.5, 0.999] {
                let origin = origin.map(|v| v as f32 + shift);
                let dir = dir.map(|v| v as f32);
                prop_assert_eq!(trace_shadow(&occ, origin, dir), trace(&occ, origin, dir, false));
                prop_assert_eq!(shader_reference::trace(occ.words(), origin, dir), trace_shadow(&occ, origin, dir));
            }
        }
    }
}
