//! Independent surface-area and greedy-mesh geometry checks.
use proptest::prelude::*;
use voxelite_core::*;
use voxelite_render::*;
fn model(cells: Vec<Voxel>) -> Snapshot {
    let mut e = Engine::new(Document::new(Uuid::from_u128(1), Uuid::from_u128(2))).unwrap();
    e.execute(Transaction {
        expected_revision: 0,
        id: Uuid::from_u128(3),
        actor: Actor::new(Origin::Human),
        label: "Mesh fixture".into(),
        command: Command::SetVoxels {
            layer: Uuid::from_u128(2),
            voxels: cells,
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
    })
    .unwrap();
    e.document().content.clone()
}
#[test]
fn cuboid_merges_to_six_outward_faces_across_chunk_seams() {
    let cells = Region::new([-20, 0, -1], [20, 5, 2])
        .unwrap()
        .cells()
        .map(|at| Voxel { at, slot: 1 })
        .collect();
    let mesh = greedy_mesh(&model(cells)).unwrap();
    assert_eq!(mesh.quads.len(), 6);
    assert_eq!(mesh.exposed_faces, 2 * (40 * 5 + 40 * 3 + 5 * 3));
    for q in mesh.quads {
        let p = q.vertices();
        let a = glam::Vec3::from(p[1]) - glam::Vec3::from(p[0]);
        let b = glam::Vec3::from(p[2]) - glam::Vec3::from(p[0]);
        assert!(a.cross(b).dot(glam::Vec3::from(q.normal())) > 0.0);
    }
}
#[test]
fn empty_and_admission() {
    assert!(greedy_mesh(&model(vec![])).unwrap().quads.is_empty());
    let s = model(vec![Voxel {
        at: [0; 3],
        slot: 1,
    }]);
    assert!(greedy_mesh_with_limit(&s, 5).is_err());
    assert_eq!(greedy_mesh_with_limit(&s, 6).unwrap().quads.len(), 6);
}
proptest! {
    #[test]fn surface_matches_independent_neighbor_count(cells in prop::collection::btree_set((-5i32..5,0i32..5,-5i32..5),0..100)){
        let s=model(cells.iter().map(|&(x,y,z)|Voxel{at:[x,y,z],slot:1}).collect());let mesh=greedy_mesh(&s).unwrap();
        let expected:usize=cells.iter().map(|&(x,y,z)|NEIGHBORS.iter().filter(|d|!cells.contains(&(x+d[0],y+d[1],z+d[2]))).count()).sum();
        prop_assert_eq!(mesh.exposed_faces,expected);prop_assert_eq!(mesh.quads.iter().map(|q|q.area()).sum::<usize>(),expected);prop_assert_eq!(greedy_mesh(&s).unwrap(),mesh);
    }
}

fn edit(engine: &mut Engine, at: Pos, mode: Mode) {
    engine
        .execute(Transaction {
            expected_revision: engine.revision(),
            id: Uuid::new_v4(),
            actor: Actor::new(Origin::Human),
            label: "Chunk regression".into(),
            command: Command::SetVoxels {
                layer: engine.document().content.layers[0].id,
                voxels: vec![Voxel { at, slot: 1 }],
                mode,
                mirror: Mirror::default(),
            },
        })
        .unwrap();
}
fn assert_cached_matches_fresh(cache: &ChunkMeshes, snapshot: &Snapshot) {
    let mut fresh = ChunkMeshes::default();
    fresh.sync_validated(snapshot).unwrap();
    assert_eq!(
        cache.meshes.keys().collect::<Vec<_>>(),
        fresh.meshes.keys().collect::<Vec<_>>()
    );
    for (at, mesh) in &cache.meshes {
        assert_eq!(mesh.vertices, fresh.meshes[at].vertices, "chunk {at:?}");
    }
    assert_eq!(
        cache
            .meshes
            .values()
            .map(|m| m.exposed_faces)
            .sum::<usize>(),
        greedy_mesh(snapshot).unwrap().exposed_faces
    );
    for m in cache.meshes.values() {
        for triangle in m.vertices.as_chunks::<3>().0 {
            let a = glam::Vec3::from(triangle[1].position) - glam::Vec3::from(triangle[0].position);
            let b = glam::Vec3::from(triangle[2].position) - glam::Vec3::from(triangle[0].position);
            assert!(a.cross(b).dot(glam::Vec3::from(triangle[0].normal)) > 0.);
        }
    }
}
#[test]
fn incremental_mesh_handles_negative_seams_ao_removal_and_undo_redo() {
    let mut engine = Engine::new(Document::default()).unwrap();
    let mut cache = ChunkMeshes::default();
    for at in [
        [-17, 15, -17],
        [-16, 15, -17],
        [-16, 16, -16],
        [-15, 16, -16],
        [15, 15, 15],
        [16, 15, 15],
        [16, 16, 16],
        [8, 8, 8],
    ] {
        edit(&mut engine, at, Mode::Place);
        cache.sync_validated(&engine.document().content).unwrap();
        assert_cached_matches_fresh(&cache, &engine.document().content);
    }
    let updates = cache.sync_validated(&engine.document().content).unwrap();
    assert!(updates.is_empty());
    edit(&mut engine, [8, 8, 9], Mode::Place);
    assert_eq!(
        cache.sync_validated(&engine.document().content).unwrap(),
        vec![[0, 0, 0]]
    );
    edit(&mut engine, [-16, 16, -16], Mode::Erase);
    cache.sync_validated(&engine.document().content).unwrap();
    assert_cached_matches_fresh(&cache, &engine.document().content);
    let head = engine.head().unwrap();
    engine.undo(engine.revision(), head).unwrap();
    cache.sync_validated(&engine.document().content).unwrap();
    assert_cached_matches_fresh(&cache, &engine.document().content);
    engine.redo(engine.revision(), head).unwrap();
    cache.sync_validated(&engine.document().content).unwrap();
    assert_cached_matches_fresh(&cache, &engine.document().content);
    let mut hidden = engine.document().content.clone();
    hidden.layers[0].visible = false;
    cache.sync_validated(&hidden).unwrap();
    assert!(cache.meshes.is_empty());
    cache.sync_validated(&engine.document().content).unwrap();
    assert_cached_matches_fresh(&cache, &engine.document().content);
    let mut recolored = engine.document().content.clone();
    recolored.palette.slots[0].color.rgba = [200, 30, 90, 255];
    cache.sync_validated(&recolored).unwrap();
    assert_cached_matches_fresh(&cache, &recolored);
}
