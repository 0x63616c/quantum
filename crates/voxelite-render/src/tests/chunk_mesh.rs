//! Incremental GPU chunk geometry must match a fresh mesh.
use crate::chunk_mesh::ChunkMeshes;
use voxelite_core::*;
use voxelite_mesh::greedy_mesh;

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
