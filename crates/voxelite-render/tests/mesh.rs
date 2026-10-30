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
