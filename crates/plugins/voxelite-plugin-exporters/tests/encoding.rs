//! Parse generated GLB with an independent glTF implementation and decode PNG.
use proptest::prelude::*;
use voxelite_core::*;
use voxelite_plugin_exporters::encoding::*;
#[expect(
    clippy::unwrap_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn model(points: Vec<Pos>) -> Snapshot {
    let mut e = Engine::new(Document::new(Uuid::from_u128(1), Uuid::from_u128(2))).unwrap();
    e.execute(Transaction {
        expected_revision: 0,
        id: Uuid::from_u128(3),
        actor: Actor::new(Origin::Agent),
        label: "Export fixture".into(),
        command: Command::SetVoxels {
            layer: Uuid::from_u128(2),
            voxels: points.into_iter().map(|at| Voxel { at, slot: 1 }).collect(),
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
    })
    .unwrap();
    e.document().content.clone()
}
#[expect(
    clippy::unwrap_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn check_glb(s: &Snapshot) {
    let bytes = glb(s).unwrap();
    assert_eq!(bytes, glb(s).unwrap());
    assert_eq!(
        u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
        bytes.len()
    );
    let parsed = gltf::Gltf::from_slice(&bytes).unwrap();
    assert_eq!(
        parsed.document.buffers().count(),
        usize::from(!s.composite().is_empty())
    );
    for buffer in parsed.document.buffers() {
        assert!(matches!(buffer.source(), gltf::buffer::Source::Bin));
    }
    for mesh in parsed.document.meshes() {
        for primitive in mesh.primitives() {
            assert_eq!(primitive.mode(), gltf::mesh::Mode::Triangles);
            let blob = parsed.blob.as_deref().unwrap();
            let reader = primitive.reader(|_| Some(blob));
            let positions: Vec<_> = reader.read_positions().unwrap().collect();
            let normals: Vec<_> = reader.read_normals().unwrap().collect();
            let colors: Vec<_> = reader.read_colors(0).unwrap().into_rgba_f32().collect();
            let indices: Vec<_> = reader.read_indices().unwrap().into_u32().collect();
            assert_eq!(positions.len(), normals.len());
            assert_eq!(positions.len(), colors.len());
            assert!(
                colors
                    .iter()
                    .all(|c| c[3] == 1.0 && c[..3].iter().all(|c| (0.0..=1.0).contains(c)))
            );
            assert!(indices.iter().all(|i| (*i as usize) < positions.len()));
            for t in indices.as_chunks::<3>().0 {
                let p = positions[t[0] as usize];
                let q = positions[t[1] as usize];
                let r = positions[t[2] as usize];
                let a: [f32; 3] = std::array::from_fn(|i| q[i] - p[i]);
                let b: [f32; 3] = std::array::from_fn(|i| r[i] - p[i]);
                let cross = [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ];
                assert!(
                    (0..3)
                        .map(|i| cross[i] * normals[t[0] as usize][i])
                        .sum::<f32>()
                        > 0.0
                );
            }
        }
    }
}
#[test]
fn cube_validates_and_is_greedy() {
    let s = model(vec![[0; 3], [1, 0, 0]]);
    check_glb(&s);
    let parsed = gltf::Gltf::from_slice(&glb(&s).unwrap()).unwrap();
    assert_eq!(parsed.document.accessors().next().unwrap().count(), 24);
}
#[test]
fn empty_glb_is_valid_scene() {
    check_glb(&model(vec![]));
}
#[test]
fn hidden_layers_are_excluded() {
    let mut s = model(vec![[0; 3]]);
    s.layers[0].visible = false;
    check_glb(&s);
    assert_eq!(
        gltf::Gltf::from_slice(&glb(&s).unwrap())
            .unwrap()
            .document
            .meshes()
            .count(),
        0
    );
}
#[test]
fn png_roundtrip_and_determinism() {
    let pixels = [
        255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 123, 45, 67, 255,
    ];
    let bytes = png_rgba(2, 2, &pixels).unwrap();
    assert_eq!(bytes, png_rgba(2, 2, &pixels).unwrap());
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut decoded = vec![0; decoder.output_buffer_size().unwrap()];
    let info = decoder.next_frame(&mut decoded).unwrap();
    assert_eq!(&decoded[..info.buffer_size()], &pixels);
    assert_eq!(info.color_type, png::ColorType::Rgba);
    assert!(decoder.info().srgb.is_some());
}
#[test]
fn invalid_png_dimensions_reject() {
    for (w, h) in [(0, 1), (1, 0), (2049, 1), (u32::MAX, u32::MAX), (1, 1)] {
        assert!(png_rgba(w, h, &[]).is_err());
    }
}
#[test]
fn output_io_failure_propagates() {
    struct Fail;
    impl std::io::Write for Fail {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disk full"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(write_glb(&model(vec![[0; 3]]), Fail).is_err());
}
proptest! {
    #[test]fn generated_geometry_parses(points in prop::collection::vec((-4i32..4,0i32..5,-4i32..4),0..40)){check_glb(&model(points.into_iter().map(|(x,y,z)|[x,y,z]).collect()));}
}
