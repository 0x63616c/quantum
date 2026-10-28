use super::*;
use crate::scene;
use voxelite_core::*;

fn compare(pipeline: &Pipeline, snapshot: &Snapshot) {
    let mut current = production::ChunkMeshes::default();
    current.sync_validated(snapshot).unwrap();
    let (actual, _) = pipeline.evaluate().unwrap();
    assert!(actual.iter().map(|(at, _)| at).eq(current.meshes.keys()));
    for (at, mesh) in actual.iter() {
        let expected = &current.meshes[at];
        assert_eq!(mesh.exposed_faces, expected.exposed_faces, "{at:?}");
        assert_eq!(mesh.vertices.len(), expected.vertices.len(), "{at:?}");
        for (a, b) in mesh.vertices.iter().zip(&expected.vertices) {
            assert_eq!(a.position.map(f32::to_bits), b.position.map(f32::to_bits));
            assert_eq!(a.normal.map(f32::to_bits), b.normal.map(f32::to_bits));
            assert_eq!(a.color_ao.map(f32::to_bits), b.color_ao.map(f32::to_bits));
        }
    }
}

#[test]
fn workloads_match_production_bit_for_bit_and_noop_reuses_queries() {
    let initial = scene::terrain(48).unwrap();
    let mut pipeline = Pipeline::new(&initial, true);
    compare(&pipeline, &initial);
    for workload in ["single", "boundary", "bulk", "palette", "noop"] {
        let snapshot = scene::variant(&initial, workload).unwrap();
        pipeline.sync(&snapshot, true);
        compare(&pipeline, &snapshot);
        let counts = pipeline.counts();
        pipeline.sync(&snapshot, true);
        compare(&pipeline, &snapshot);
        assert_eq!(counts, pipeline.counts());
        pipeline.sync(&initial, true);
        compare(&pipeline, &initial);
    }
}

#[test]
fn interior_edits_backdate_neighbor_halos_and_settings_do_not_mesh() {
    let snapshot = scene::terrain(48).unwrap();
    let mut pipeline = Pipeline::new(&snapshot, true);
    compare(&pipeline, &snapshot);
    let old = pipeline.counts();
    let edited = scene::writes(
        &snapshot,
        vec![Voxel {
            at: [8, 8, 8],
            slot: 0,
        }],
    )
    .unwrap();
    pipeline.sync(&edited, true);
    compare(&pipeline, &edited);
    let counts = pipeline.counts();
    assert_eq!(counts[0] - old[0], 8); // Only in-bounds neighbors at this corner.
    assert_eq!(counts[1] - old[1], 1);
    assert_eq!(counts[2], old[2]);
    pipeline.sync(&edited, false);
    assert_eq!(pipeline.evaluate().unwrap().1, RenderPlan::default());
    let next = pipeline.counts();
    assert_eq!(next[0..2], counts[0..2]);
    assert_eq!(next[2], counts[2] + 1);
}

#[test]
fn creation_removal_occlusion_layer_changes_palette_and_bounds_match() {
    let mut snapshot = Document::default().content;
    snapshot.bounds = Region::new([-32; 3], [32; 3]).unwrap();
    snapshot.palette =
        Palette::from_colors(vec![Swatch::rgb(100, 120, 130), Swatch::rgb(200, 10, 30)]).unwrap();
    let mut pipeline = Pipeline::new(&snapshot, true);
    for (at, slot) in [
        ([-16, -16, -16], 1),
        ([-17, -17, -17], 2),
        ([0, 0, 0], 1),
        ([0, 0, 0], 0),
    ] {
        snapshot = scene::writes(&snapshot, vec![Voxel { at, slot }]).unwrap();
        pipeline.sync(&snapshot, true);
        compare(&pipeline, &snapshot);
    }
    // Fixture construction only. Live changes above go through Engine.
    let mut top = Layer::new(Uuid::from_u128(9), "Top");
    top.voxels = serde_json::from_value(serde_json::json!([[-16, -16, -16, 2]])).unwrap();
    snapshot.layers.push(top);
    for phase in 0..5 {
        match phase {
            0 => {}
            1 => snapshot.layers[1].visible = false,
            2 => {
                snapshot.layers[1].visible = true;
                snapshot.layers.swap(0, 1);
            }
            3 => snapshot.palette.slots[0].color = Swatch::rgb(2, 20, 200),
            _ => snapshot.bounds = Region::new([-32; 3], [48; 3]).unwrap(),
        }
        snapshot.validate().unwrap();
        pipeline.sync(&snapshot, true);
        compare(&pipeline, &snapshot);
    }
}

#[test]
fn chunk_mesh_can_be_requested_on_a_background_database_clone() {
    let snapshot = scene::terrain(32).unwrap();
    let pipeline = Pipeline::new(&snapshot, true);
    let db = pipeline.db.clone();
    let document = pipeline.document;
    let actual = std::thread::spawn(move || document_meshes(&db, document).unwrap())
        .join()
        .unwrap();
    let expected = pipeline.evaluate().unwrap().0;
    assert!(Arc::ptr_eq(&actual, &expected));
}
