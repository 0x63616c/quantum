//! Portable command data and its stable serialization.
use crate::{Anchor, Pos, Region, Swatch, Uuid};
use serde::{Deserialize, Serialize};

/// Voxel editing semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Write, overwriting existing cells.
    Place,
    /// Recolor existing cells only.
    Paint,
    /// Remove cells.
    Erase,
}
/// World axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
}
impl Axis {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
}
/// Explicit independent mirror planes. `None` disables an axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mirror {
    /// Doubled plane positions; reflection is `plane2 - cell - 1`.
    pub plane2: [Option<i32>; 3],
}
impl Mirror {
    /// Enable selected axes at the document midplanes.
    pub fn centered(bounds: Region, axes: [bool; 3]) -> Self {
        Self {
            plane2: std::array::from_fn(|a| axes[a].then_some(bounds.min[a] + bounds.max[a])),
        }
    }
}
/// An explicit cell write. Slot zero is accepted by SetVoxels to erase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voxel {
    /// World coordinate.
    pub at: Pos,
    /// Stable palette slot, or zero for air.
    pub slot: u16,
}
/// Integer shape occupancy; no platform-dependent floating point boundaries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Shape {
    /// Half-open box, optionally a one-cell shell.
    Box {
        /// Candidate region.
        region: Region,
        /// Emit only six-neighbor shell cells.
        hollow: bool,
    },
    /// Ellipsoid evaluated at doubled cell centers.
    Ellipsoid {
        /// Doubled center coordinates.
        center2: Pos,
        /// Positive doubled radii, at most 512.
        radii2: Pos,
        /// Emit only the six-neighbor shell.
        hollow: bool,
    },
    /// Axis-aligned elliptic cylinder, with caps when hollow.
    Cylinder {
        /// Cylinder axis.
        axis: Axis,
        /// Doubled center; axial component is ignored.
        center2: Pos,
        /// Doubled radii; axial component is ignored.
        radii2: Pos,
        /// Half-open axial interval.
        range: [i32; 2],
        /// Emit only the six-neighbor shell.
        hollow: bool,
    },
    /// Center-to-center supercover, including all cells touched at ties.
    Line {
        /// First cell.
        from: Pos,
        /// Last cell.
        to: Pos,
    },
}
/// Collision behavior for a region transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Collision {
    /// Reject any occupied destination.
    Reject,
    /// Overwrite destinations.
    Overwrite,
    /// Leave occupied destinations untouched (copy only).
    Skip,
}
/// Integer copy/move transform, in reflection → rotation → translation order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transform {
    /// Source layer identity.
    pub source: Uuid,
    /// Destination layer identity.
    pub destination: Uuid,
    /// Source region.
    pub region: Region,
    /// Remove source after all validation; false copies.
    pub move_cells: bool,
    /// Integer translation.
    pub translate: Pos,
    /// Doubled rotation/reflection pivot.
    pub pivot2: Pos,
    /// Axes to reflect.
    pub reflect: [bool; 3],
    /// Optional right-handed axis rotation and quarter-turn count (0..=3).
    pub rotate: Option<(Axis, u8)>,
    /// Explicit destination collision policy.
    pub collision: Collision,
}
/// Literal palette edits, suitable for ordered atomic batches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaletteEdit {
    /// Add a stable slot; None chooses the lowest unused slot.
    Add {
        /// Optional requested identity.
        slot: Option<u16>,
        /// Opaque color.
        color: Swatch,
    },
    /// Recolor an existing stable slot, including appearance on locked layers.
    Set {
        /// Stable slot.
        slot: u16,
        /// Replacement color.
        color: Swatch,
    },
    /// Delete a slot, explicitly remapping any references.
    Remove {
        /// Slot to delete.
        slot: u16,
        /// Replacement for referenced cells.
        replacement: Option<u16>,
    },
    /// Reorder display swatches without recoloring references.
    Reorder(Vec<u16>),
}
/// Layer metadata and structural edits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerEdit {
    /// Insert an empty layer at a bottom-to-top index.
    Create {
        /// Caller-supplied identity.
        id: Uuid,
        /// Display name.
        name: String,
        /// Insertion index.
        index: usize,
    },
    /// Change metadata. Unlocking must precede voxel edits in another command.
    Update {
        /// Target identity.
        id: Uuid,
        /// Optional replacement name.
        name: Option<String>,
        /// Optional visibility.
        visible: Option<bool>,
        /// Optional lock flag.
        locked: Option<bool>,
    },
    /// Duplicate cells and metadata into an unlocked layer just above the source.
    Duplicate {
        /// Source identity.
        source: Uuid,
        /// New identity.
        id: Uuid,
        /// New name.
        name: String,
    },
    /// Full bottom-to-top permutation of layer IDs.
    Reorder(Vec<Uuid>),
    /// Delete an unlocked layer, retaining any discussion anchors.
    Delete(Uuid),
}
/// Typed command shared by human tools and agent adapters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    /// Last explicit duplicate wins before mirror expansion.
    SetVoxels {
        /// Target layer.
        layer: Uuid,
        /// Explicit writes (at most 65,536).
        #[serde(with = "voxel_tuples")]
        voxels: Vec<Voxel>,
        /// Editing semantics.
        mode: Mode,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Erase occupied cells in a half-open region.
    EraseRegion {
        /// Target layer.
        layer: Uuid,
        /// Finite erase extent.
        region: Region,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Six-connected bounded fill of the seed's slot, including air.
    FloodFill {
        /// Target layer.
        layer: Uuid,
        /// Seed in region.
        seed: Pos,
        /// Nonzero replacement slot.
        slot: u16,
        /// Finite search extent; None uses document bounds.
        region: Option<Region>,
        /// Maximum distinct visited cells.
        cap: usize,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Apply an exact integer primitive.
    Primitive {
        /// Target layer.
        layer: Uuid,
        /// Shape occupancy rule.
        shape: Shape,
        /// Editing semantics.
        mode: Mode,
        /// Nonzero for place/paint, None for erase.
        slot: Option<u16>,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Copy or move occupied source cells atomically.
    TransformRegion(Transform),
    /// Ordered atomic commands; nesting and version/comment operations are forbidden.
    Batch(Vec<Command>),
    /// Modify literal palette data.
    Palette(PaletteEdit),
    /// Modify layers.
    Layer(LayerEdit),
    /// Rename or resize the document without clipping hidden cells.
    EditDocument {
        /// Optional display name.
        name: Option<String>,
        /// Optional finite grid.
        bounds: Option<Region>,
    },
    /// Create an anchored thread. Message authorship is assigned from the transaction.
    CreateComment {
        /// Thread identity.
        id: Uuid,
        /// Spatial anchor.
        anchor: Anchor,
        /// Optional layer anchor.
        layer: Option<Uuid>,
        /// First message identity.
        message: Uuid,
        /// Plain text.
        text: String,
    },
    /// Explicitly recover or move an anchor.
    Reanchor {
        /// Thread identity.
        id: Uuid,
        /// New spatial anchor.
        anchor: Anchor,
        /// Optional layer identity.
        layer: Option<Uuid>,
    },
    /// Append to a thread, preserving message order.
    Reply {
        /// Thread identity.
        id: Uuid,
        /// Message identity.
        message: Uuid,
        /// Plain text.
        text: String,
    },
    /// Resolve or reopen a discussion.
    Resolve {
        /// Thread identity.
        id: Uuid,
        /// Desired state.
        resolved: bool,
    },
    /// Persist the current content as a nonrecursive named snapshot.
    SaveVersion {
        /// Version identity.
        id: Uuid,
        /// Display name.
        name: String,
        /// Optional parent version.
        parent: Option<Uuid>,
    },
    /// Restore snapshot content while preserving the version library.
    RestoreVersion {
        /// Version identity.
        id: Uuid,
        /// Captured live history head.
        expected_head: Option<Uuid>,
    },
}

// Command recipes use the same compact coordinate tuples as document volumes,
// including slot zero for erasure. Voxel's public object representation is unchanged.
mod voxel_tuples {
    use super::Voxel;
    use serde::{Deserialize, Deserializer, Serializer, de::Error, ser::SerializeSeq};

    pub fn serialize<S: Serializer>(voxels: &[Voxel], serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(voxels.len()))?;
        for voxel in voxels {
            seq.serialize_element(&[voxel.at[0], voxel.at[1], voxel.at[2], i32::from(voxel.slot)])?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Voxel>, D::Error> {
        Vec::<[i32; 4]>::deserialize(deserializer)?
            .into_iter()
            .map(|[x, y, z, slot]| {
                Ok(Voxel {
                    at: [x, y, z],
                    slot: u16::try_from(slot).map_err(D::Error::custom)?,
                })
            })
            .collect()
    }
}
