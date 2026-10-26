use crate::Pos;
use std::{collections::HashMap, sync::Arc};

/// Edge length of a runtime chunk.
pub const CHUNK_SIZE: i32 = 16;
const CELLS: usize = 4096;

/// Sparse cells promote at 256 occupied cells and demote below 128.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Cells {
    Sparse(Vec<(u16, u16)>),
    Dense(Box<[u16; CELLS]>),
}

/// Copy-on-write 16³ storage; empty cells have palette slot zero.
#[derive(Clone, Debug)]
pub struct Chunk {
    cells: Cells,
    len: usize,
}
impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.entries().eq(other.entries())
    }
}
impl Eq for Chunk {}
impl Default for Chunk {
    fn default() -> Self {
        Self {
            cells: Cells::Sparse(Vec::new()),
            len: 0,
        }
    }
}
impl Chunk {
    /// Number of occupied cells.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Whether no cells are occupied.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Whether this chunk currently uses a dense representation.
    pub fn is_dense(&self) -> bool {
        matches!(self.cells, Cells::Dense(_))
    }
    pub(crate) fn get(&self, index: u16) -> u16 {
        match &self.cells {
            Cells::Sparse(v) => v
                .binary_search_by_key(&index, |c| c.0)
                .map_or(0, |i| v[i].1),
            Cells::Dense(v) => v[usize::from(index)],
        }
    }
    pub(crate) fn set(&mut self, index: u16, slot: u16) {
        let old = self.get(index);
        if old == slot {
            return;
        }
        self.len = self.len + usize::from(slot != 0) - usize::from(old != 0);
        match &mut self.cells {
            Cells::Sparse(v) => match v.binary_search_by_key(&index, |c| c.0) {
                Ok(i) if slot == 0 => {
                    v.remove(i);
                }
                Ok(i) => v[i].1 = slot,
                Err(i) if slot != 0 => v.insert(i, (index, slot)),
                Err(_) => {}
            },
            Cells::Dense(v) => v[usize::from(index)] = slot,
        }
        match &self.cells {
            Cells::Sparse(v) if self.len >= 256 => {
                let mut dense = Box::new([0; CELLS]);
                for &(i, s) in v {
                    dense[usize::from(i)] = s;
                }
                self.cells = Cells::Dense(dense);
            }
            Cells::Dense(v) if self.len < 128 => {
                self.cells = Cells::Sparse(
                    v.iter()
                        .enumerate()
                        .filter(|(_, s)| **s != 0)
                        .map(|(i, s)| (i as u16, *s))
                        .collect(),
                );
            }
            _ => {}
        }
    }
    fn entries(&self) -> Box<dyn Iterator<Item = (u16, u16)> + '_> {
        match &self.cells {
            Cells::Sparse(v) => Box::new(v.iter().copied()),
            Cells::Dense(v) => Box::new(
                v.iter()
                    .enumerate()
                    .filter(|(_, s)| **s != 0)
                    .map(|(i, s)| (i as u16, *s)),
            ),
        }
    }
    pub(crate) fn bytes(&self) -> usize {
        32 + match &self.cells {
            Cells::Sparse(v) => v.len() * 4,
            Cells::Dense(_) => CELLS * 2,
        }
    }
}

/// Euclidean chunk address and `x + 16*y + 256*z` local index.
pub fn chunk_address(p: Pos) -> (Pos, u16) {
    let c = p.map(|v| v.div_euclid(CHUNK_SIZE));
    let r = p.map(|v| v.rem_euclid(CHUNK_SIZE));
    (c, (r[0] + 16 * r[1] + 256 * r[2]) as u16)
}

/// A sparse volume with immutable chunks shared between snapshots.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Volume {
    chunks: HashMap<Pos, Arc<Chunk>>,
    len: usize,
}
impl Volume {
    /// Read a palette slot, returning zero for air.
    pub fn get(&self, p: Pos) -> u16 {
        let (c, i) = chunk_address(p);
        self.chunks.get(&c).map_or(0, |chunk| chunk.get(i))
    }
    pub(crate) fn set(&mut self, p: Pos, slot: u16) {
        let (c, i) = chunk_address(p);
        let old = self.get(p);
        if old == slot {
            return;
        }
        self.len = self.len + usize::from(slot != 0) - usize::from(old != 0);
        let chunk = self.chunks.entry(c).or_default();
        Arc::make_mut(chunk).set(i, slot);
        if chunk.is_empty() {
            self.chunks.remove(&c);
        }
    }
    /// Number of occupied cells, independent of representation.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Whether this volume contains no occupied voxels.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Number of allocated nonempty chunks.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
    /// Inspect an immutable runtime chunk.
    pub fn chunk(&self, at: Pos) -> Option<&Chunk> {
        self.chunks.get(&at).map(AsRef::as_ref)
    }
    /// Allocated chunk coordinates, in unspecified order.
    pub fn chunk_positions(&self) -> impl Iterator<Item = Pos> + '_ {
        self.chunks.keys().copied()
    }
    /// Chunks whose content differs, including removed chunks. Shared immutable
    /// chunks are skipped in O(1); independently loaded equal chunks are compared.
    pub fn changed_chunks(&self, previous: &Self) -> Vec<Pos> {
        let mut changed: Vec<_> = self
            .chunks
            .iter()
            .filter_map(|(at, chunk)| {
                (!previous
                    .chunks
                    .get(at)
                    .is_some_and(|old| Arc::ptr_eq(old, chunk) || old == chunk))
                .then_some(*at)
            })
            .collect();
        changed.extend(
            previous
                .chunks
                .keys()
                .filter(|at| !self.chunks.contains_key(*at))
                .copied(),
        );
        changed.sort_unstable();
        changed
    }
    /// Exact cell differences, skipping immutable chunks shared with `previous`.
    pub fn changed_cells(&self, previous: &Self) -> Vec<(Pos, u16, u16)> {
        let mut out = Vec::new();
        for at in self.changed_chunks(previous) {
            let coords: std::collections::BTreeSet<_> = self
                .chunk_entries(at)
                .chain(previous.chunk_entries(at))
                .map(|(p, _)| p)
                .collect();
            for p in coords {
                let old = previous.get(p);
                let new = self.get(p);
                if old != new {
                    out.push((p, old, new));
                }
            }
        }
        out
    }
    /// Occupied world cells belonging to one immutable chunk.
    pub fn chunk_entries(&self, at: Pos) -> impl Iterator<Item = (Pos, u16)> + '_ {
        self.chunks.get(&at).into_iter().flat_map(move |chunk| {
            chunk.entries().map(move |(i, slot)| {
                let i = i32::from(i);
                (
                    [
                        at[0] * 16 + i % 16,
                        at[1] * 16 + (i / 16) % 16,
                        at[2] * 16 + i / 256,
                    ],
                    slot,
                )
            })
        })
    }
    /// Occupied cells in unspecified order; sort for portable output.
    pub fn iter(&self) -> impl Iterator<Item = (Pos, u16)> + '_ {
        self.chunks.iter().flat_map(|(c, chunk)| {
            chunk.entries().map(move |(i, s)| {
                let i = i32::from(i);
                (
                    [
                        c[0] * 16 + i % 16,
                        c[1] * 16 + (i / 16) % 16,
                        c[2] * 16 + i / 256,
                    ],
                    s,
                )
            })
        })
    }
    /// Occupied cells in numeric x/y/z order.
    pub fn sorted(&self) -> Vec<(Pos, u16)> {
        let mut v: Vec<_> = self.iter().collect();
        v.sort_unstable();
        v
    }
    pub(crate) fn bytes(&self) -> usize {
        self.chunks.values().map(|c| c.bytes() + 64).sum()
    }
}
impl serde::Serialize for Volume {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(self.len))?;
        for ([x, y, z], slot) in self.sorted() {
            seq.serialize_element(&[x, y, z, i32::from(slot)])?;
        }
        seq.end()
    }
}
impl<'de> serde::Deserialize<'de> for Volume {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Volume;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("unique bounded voxel tuples")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Volume, A::Error> {
                use serde::de::Error;
                let mut v = Volume::default();
                while let Some([x, y, z, s]) = seq.next_element::<[i32; 4]>()? {
                    if v.len >= crate::MAX_VOXELS || v.chunk_count() > crate::MAX_CHUNKS {
                        return Err(A::Error::custom("voxel/chunk limit"));
                    }
                    if !(1..=256).contains(&s)
                        || [x, y, z].iter().any(|v| !(-32768..32768).contains(v))
                    {
                        return Err(A::Error::custom("invalid voxel tuple"));
                    }
                    if v.get([x, y, z]) != 0 {
                        return Err(A::Error::custom("duplicate coordinate"));
                    }
                    v.set([x, y, z], s as u16);
                }
                Ok(v)
            }
        }
        d.deserialize_seq(Visitor)
    }
}
