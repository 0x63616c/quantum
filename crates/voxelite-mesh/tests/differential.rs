//! Exact snapshot geometry, ordering and admission checks against the old mesher.
use proptest::prelude::*;
use voxelite_core::*;
use voxelite_mesh::*;

#[path = "support/reference.rs"]
mod reference;

#[expect(clippy::unwrap_used, reason = "test fixture construction")]
fn scene(bounds: Region, layers: [Vec<Voxel>; 2]) -> Snapshot {
    let mut document = Document::default();
    document.content.bounds = bounds;
    document.content.palette.slots.push(PaletteSlot {
        index: 256,
        color: Swatch::rgb(10, 20, 30),
    });
    document.content.palette.order.push(256);
    document
        .content
        .layers
        .push(Layer::new(Uuid::new_v4(), "Top"));
    let mut hidden = Layer::new(Uuid::new_v4(), "Hidden");
    hidden.visible = false;
    document.content.layers.push(hidden);
    let mut engine = Engine::new(document).unwrap();
    for (i, voxels) in layers.into_iter().enumerate() {
        engine
            .execute(Transaction {
                expected_revision: engine.revision(),
                id: Uuid::new_v4(),
                actor: Actor::new(Origin::Human),
                label: "Differential mesh fixture".into(),
                command: Command::SetVoxels {
                    layer: engine.document().content.layers[i].id,
                    voxels,
                    mode: Mode::Place,
                    mirror: Mirror::default(),
                },
            })
            .unwrap();
    }
    let mut snapshot = engine.document().content.clone();
    snapshot.layers[2].voxels = snapshot.layers[0].voxels.clone();
    snapshot
}

#[expect(clippy::unwrap_used, reason = "test oracle assertion")]
fn assert_reference(snapshot: &Snapshot) {
    let mesh = greedy_mesh(snapshot).unwrap();
    assert_eq!(
        mesh,
        reference::greedy_mesh_with_limit(snapshot, MAX_QUADS).unwrap()
    );
    for limit in [
        0,
        5,
        mesh.quads.len().saturating_sub(1),
        mesh.quads.len(),
        MAX_QUADS + 1,
    ] {
        assert_eq!(
            greedy_mesh_with_limit(snapshot, limit),
            reference::greedy_mesh_with_limit(snapshot, limit),
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]
    #[test]
    fn colored_layered_geometry_and_limits_match_reference(
        cells in prop::collection::btree_map((-18i32..19, -2i32..3, -18i32..19), any::<bool>(), 0..100),
    ) {
        let base: Vec<_> = cells.iter().map(|(&(x, y, z), &color)| Voxel {
            at: [x, y, z], slot: if color { 256 } else { 1 },
        }).collect();
        let top = base.iter().step_by(2).map(|voxel| Voxel {
            at: voxel.at, slot: if voxel.slot == 1 { 256 } else { 1 },
        }).collect();
        let mut snapshot = scene(Region::new([-32; 3], [32; 3]).unwrap(), [base, top]);
        assert_reference(&snapshot);
        snapshot.layers.swap(0, 1);
        assert_reference(&snapshot);
        snapshot.layers[0].visible = false;
        snapshot.layers[2].visible = true;
        assert_reference(&snapshot);
    }
}

#[test]
fn full_width_masks_and_coordinate_extrema_match_reference() {
    for base in [-32768, 32512] {
        let bounds = Region::new([base; 3], [base + 256; 3]).unwrap();
        let voxels = (0..256)
            .map(|x| Voxel {
                at: [base + x, base, base],
                slot: 256,
            })
            .collect();
        assert_reference(&scene(bounds, [voxels, vec![]]));
    }
}

#[test]
fn invalid_snapshots_preserve_validation_precedence() {
    let mut snapshot = Document::default().content;
    snapshot.palette.order.clear();
    assert_eq!(
        greedy_mesh_with_limit(&snapshot, MAX_QUADS + 1),
        reference::greedy_mesh_with_limit(&snapshot, MAX_QUADS + 1),
    );
}
