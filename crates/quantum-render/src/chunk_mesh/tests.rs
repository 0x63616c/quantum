//! Differential coverage against the verbatim pre-local-block mesher in reference.rs.
use super::*;
use quantum_core::*;

fn random(seed: &mut u64) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed as u32
}

fn volume(cells: impl IntoIterator<Item = (Pos, u16)>) -> Volume {
    let tuples: Vec<_> = cells
        .into_iter()
        .map(|([x, y, z], slot)| [x, y, z, i32::from(slot)])
        .collect();
    serde_json::from_value(serde_json::json!(tuples)).unwrap()
}

fn scene() -> Snapshot {
    let mut snapshot = Document::default().content;
    snapshot.bounds = Region::new([-48; 3], [48; 3]).unwrap();
    snapshot.palette = Palette {
        slots: vec![
            PaletteSlot {
                index: 1,
                color: Swatch::rgb(0, 10, 11),
            },
            PaletteSlot {
                index: 7,
                color: Swatch::rgb(255, 127, 63),
            },
            PaletteSlot {
                index: 256,
                color: Swatch::rgb(30, 200, 90),
            },
        ],
        order: vec![256, 1, 7],
    };
    snapshot
}

fn sync(
    cache: &mut ChunkMeshes,
    old: &mut reference::ChunkMeshes,
    snapshot: &Snapshot,
) -> Vec<Pos> {
    snapshot.validate().unwrap();
    let updates = cache.sync_validated(snapshot);
    assert_eq!(updates, old.sync_validated(snapshot));
    assert!(cache.meshes.keys().eq(old.meshes.keys()));
    for (at, mesh) in &cache.meshes {
        let expected = &old.meshes[at];
        assert_eq!(mesh.exposed_faces, expected.exposed_faces, "chunk {at:?}");
        assert_eq!(mesh.vertices.len(), expected.vertices.len(), "chunk {at:?}");
        for (i, (actual, expected)) in mesh.vertices.iter().zip(&expected.vertices).enumerate() {
            assert_eq!(
                actual.position.map(f32::to_bits),
                expected.position.map(f32::to_bits),
                "position {at:?}/{i}"
            );
            assert_eq!(
                actual.normal.map(f32::to_bits),
                expected.normal.map(f32::to_bits),
                "normal {at:?}/{i}"
            );
            assert_eq!(
                actual.color_ao.map(f32::to_bits),
                expected.color_ao.map(f32::to_bits),
                "color/AO {at:?}/{i}"
            );
        }
    }
    updates.unwrap()
}

#[test]
fn random_sparse_and_dense_composites_match_reference() {
    for (mut seed, density) in [(0x123456789abcdef, 5), (0xfedcba987654321, 85), (41, 100)] {
        let mut snapshot = scene();
        snapshot.layers[0].voxels = volume(
            Region::new([-18; 3], [19; 3])
                .unwrap()
                .cells()
                .filter_map(|p| {
                    if random(&mut seed) % 100 < density {
                        Some((p, [1, 7, 256][random(&mut seed) as usize % 3]))
                    } else {
                        None
                    }
                }),
        );
        let mut cache = ChunkMeshes::default();
        let mut old = reference::ChunkMeshes::default();
        assert!(sync(&mut cache, &mut old, &snapshot).len() >= 4);
        assert!(sync(&mut cache, &mut old, &snapshot).is_empty());
        // Repeated full remesh also checks ordering across worker schedules.
        snapshot.palette.slots[1].color = Swatch::rgb(11, 10, 240);
        sync(&mut cache, &mut old, &snapshot);
    }
}

#[test]
fn layers_visibility_reorder_palette_and_hidden_edits_match_reference() {
    let mut snapshot = scene();
    let region = Region::new([-17, -2, -17], [18, 3, 18]).unwrap();
    snapshot.layers[0].voxels = volume(region.cells().map(|p| (p, 1)));
    let mut top = Layer::new(Uuid::from_u128(2), "Top");
    top.voxels = volume(region.cells().filter(|p| p[1] >= 0).map(|p| (p, 7)));
    let mut hidden = Layer::new(Uuid::from_u128(3), "Hidden");
    hidden.visible = false;
    hidden.voxels = volume(region.cells().map(|p| (p, 256)));
    snapshot.layers.extend([top, hidden]);
    let mut cache = ChunkMeshes::default();
    let mut old = reference::ChunkMeshes::default();
    sync(&mut cache, &mut old, &snapshot);

    // Changing or removing cells under an opaque top layer changes no buffers.
    snapshot.layers[0].voxels = volume(
        region
            .cells()
            .filter_map(|p| (p[1] != 1).then_some((p, if p[1] == 0 { 256 } else { 1 }))),
    );
    assert!(sync(&mut cache, &mut old, &snapshot).is_empty());
    snapshot.layers[2].voxels = volume([([31, 31, 31], 256)]);
    assert!(sync(&mut cache, &mut old, &snapshot).is_empty());

    snapshot.layers.swap(0, 1);
    sync(&mut cache, &mut old, &snapshot);
    snapshot.layers[2].visible = true;
    sync(&mut cache, &mut old, &snapshot);
    snapshot.layers[1].visible = false;
    sync(&mut cache, &mut old, &snapshot);
    snapshot.palette.slots[2].color = Swatch::rgb(90, 11, 10);
    sync(&mut cache, &mut old, &snapshot);
    snapshot.bounds = Region::new([-47; 3], [48; 3]).unwrap();
    sync(&mut cache, &mut old, &snapshot);
    snapshot.layers.remove(2);
    sync(&mut cache, &mut old, &snapshot);
    for layer in &mut snapshot.layers {
        layer.visible = false;
    }
    assert!(!sync(&mut cache, &mut old, &snapshot).is_empty());
    assert!(cache.meshes.is_empty());
}

#[test]
fn successive_random_voxel_and_box_edits_match_reference() {
    let mut snapshot = scene();
    let mut seed = 0x123456789;
    snapshot.layers[0].voxels = volume(
        Region::new([-18; 3], [19; 3])
            .unwrap()
            .cells()
            .filter_map(|p| random(&mut seed).is_multiple_of(5).then_some((p, 1))),
    );
    let layer = snapshot.layers[0].id;
    let mut engine = Engine::new(Document {
        content: snapshot,
        ..Document::default()
    })
    .unwrap();
    let mut cache = ChunkMeshes::default();
    let mut old = reference::ChunkMeshes::default();
    sync(&mut cache, &mut old, &engine.document().content);
    for i in 0..48 {
        let at = std::array::from_fn(|_| (random(&mut seed) % 38) as i32 - 19);
        let slot = [0, 1, 7, 256][random(&mut seed) as usize % 4];
        let command = if i % 3 != 0 {
            Command::SetVoxels {
                layer,
                voxels: vec![Voxel { at, slot }],
                mode: Mode::Place,
                mirror: Mirror::default(),
            }
        } else {
            let max = at.map(|a| a + (random(&mut seed) % 5 + 2) as i32);
            Command::Primitive {
                layer,
                shape: Shape::Box {
                    region: Region::new(at, max).unwrap(),
                    hollow: false,
                },
                mode: if slot == 0 { Mode::Erase } else { Mode::Place },
                slot: (slot != 0).then_some(slot),
                mirror: Mirror::default(),
            }
        };
        engine
            .execute(Transaction {
                expected_revision: engine.revision(),
                id: Uuid::new_v4(),
                actor: Actor::new(Origin::Human),
                label: "Differential mesh edit".into(),
                command,
            })
            .unwrap();
        sync(&mut cache, &mut old, &engine.document().content);
    }
}

#[test]
fn padded_composite_matches_sampling_across_all_neighbor_chunks() {
    let mut snapshot = scene();
    let mut seed = 31;
    for i in 0..3 {
        let mut layer = Layer::new(Uuid::from_u128(i + 10), "Composite");
        layer.visible = i != 1;
        layer.voxels = volume(
            Region::new([-19; 3], [19; 3])
                .unwrap()
                .cells()
                .filter_map(|p| {
                    random(&mut seed)
                        .is_multiple_of(3)
                        .then_some((p, [1, 7, 256][i as usize]))
                }),
        );
        snapshot.layers.push(layer);
    }
    for at in Region::new([-1; 3], [1; 3]).unwrap().cells() {
        let block = LocalCells::new(at, &snapshot);
        let region = Region::new(block.min, block.min.map(|v| v + PADDED_SIZE as i32)).unwrap();
        for p in region.cells() {
            assert_eq!(
                block.get(p),
                snapshot.sample(p).map_or(0, |s| s.0),
                "{at:?}: {p:?}"
            );
        }
    }
}

#[test]
fn ao_and_cull_fit_halo_two_even_at_coordinate_limits() {
    let mut extrema = [[i32::MAX; 3], [i32::MIN; 3]];
    // Include rounding around zero and the largest admitted world coordinates.
    for base in [-32768, -16, 0, 16, 32752] {
        let cells = LocalCells {
            min: [base - HALO; 3],
            slots: [1; PADDED_SIZE.pow(3)],
        };
        for local in Region::new([0; 3], [16; 3]).unwrap().cells() {
            for axis in 0..3 {
                for positive in [false, true] {
                    let mut origin = local.map(|v| v + base);
                    let mut neighbor = origin;
                    neighbor[axis] += if positive { 1 } else { -1 };
                    assert_eq!(cells.get(neighbor), 1);
                    origin[axis] += i32::from(positive);
                    let quad = Quad {
                        origin,
                        size: [1, 1],
                        slot: 1,
                        axis: axis as u8,
                        positive,
                    };
                    let center = std::array::from_fn(|a| {
                        origin[a] as f32 + if a == axis { 0. } else { 0.5 }
                    });
                    for vertex in quad.vertices() {
                        // Exercise the actual sampler's indexing as well as the bound proof.
                        assert_eq!(ambient(vertex, center, quad.normal(), axis, &cells), 8);
                        let p = std::array::from_fn::<_, 3, _>(|a| {
                            vertex[a]
                                + (center[a] - vertex[a]).signum() * 0.001
                                + quad.normal()[a] * 0.65
                        });
                        for [du, dv] in [
                            [-1., 0.],
                            [1., 0.],
                            [0., -1.],
                            [0., 1.],
                            [-0.7, -0.7],
                            [-0.7, 0.7],
                            [0.7, -0.7],
                            [0.7, 0.7],
                        ] {
                            let mut q = p;
                            q[(axis + 1) % 3] += du * 1.1;
                            q[(axis + 2) % 3] += dv * 1.1;
                            for (a, v) in q.map(|v| v.floor() as i32 - base).into_iter().enumerate()
                            {
                                assert!((-2..=17).contains(&v));
                                extrema[0][a] = extrema[0][a].min(v);
                                extrema[1][a] = extrema[1][a].max(v);
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(extrema, [[-2; 3], [17; 3]]);
}

#[test]
fn world_space_rounding_at_coordinate_limits_matches_reference() {
    for base in [-32768, 32736] {
        let mut snapshot = scene();
        snapshot.bounds = Region::new([base; 3], [base + 32; 3]).unwrap();
        let mut seed = 63;
        snapshot.layers[0].voxels = volume(
            snapshot
                .bounds
                .cells()
                .filter_map(|p| random(&mut seed).is_multiple_of(10).then_some((p, 256))),
        );
        sync(
            &mut ChunkMeshes::default(),
            &mut reference::ChunkMeshes::default(),
            &snapshot,
        );
    }
}

#[test]
fn quad_budget_checks_order_and_recovers_like_reference() {
    // Seed a resident mesh at the budget rather than generating 1.5M quads.
    // Run caches separately to bound test memory to one such vertex allocation.
    let mut snapshot = scene();
    macro_rules! check {
        ($cache:ty, $mesh:ty, $vertex:ty, $at:expr) => {{
            type Mesh = $mesh;
            type Vertex = $vertex;
            let mut cache = <$cache>::default();
            cache.meshes.insert(
                $at,
                Mesh {
                    vertices: vec![
                        Vertex {
                            position: [0.; 3],
                            normal: [0.; 3],
                            color_ao: [0.; 4],
                        };
                        MAX_QUADS * 6
                    ],
                    exposed_faces: 0,
                },
            );
            let mut occupied: Vec<_> = snapshot.layers[0].voxels.chunk_positions().collect();
            occupied.sort_unstable();
            let result = cache.sync_validated(&snapshot);
            if $at[0] == 0 {
                assert_eq!(
                    result,
                    Err(RenderError::Limit("GPU chunk mesh quad budget".into()))
                );
                assert!(cache.meshes.is_empty());
                assert_eq!(cache.sync_validated(&snapshot).unwrap(), occupied);
            } else {
                let expected = std::iter::once($at).chain(occupied).collect();
                assert_eq!(result, Ok(expected));
            }
            assert!(cache.sync_validated(&snapshot).unwrap().is_empty());
        }};
    }
    // Two dirty chunks use serial meshing; four exercise the parallel threshold.
    for positions in [vec![[-1, 0, 0]], vec![[-33, 0, 0], [-1, 0, 0], [32, 0, 0]]] {
        snapshot.layers[0].voxels = volume(positions.into_iter().map(|p| (p, 1)));
        for at in [[0, 0, 0], [-4, 0, 0]] {
            check!(ChunkMeshes, ChunkMesh, MeshVertex, at);
            check!(
                reference::ChunkMeshes,
                reference::ChunkMesh,
                reference::MeshVertex,
                at
            );
        }
    }
}

#[test]
fn rayon_chunk_batches_preserve_serial_order_and_vertex_bits() {
    let mut snapshot = scene();
    let mut seed = 95;
    snapshot.layers[0].voxels = volume(
        Region::new([-18; 3], [19; 3])
            .unwrap()
            .cells()
            .filter_map(|p| random(&mut seed).is_multiple_of(7).then_some((p, 1))),
    );
    let palette = vec![Some([0.25, 0.5, 0.75]); 257];
    let dirty: Vec<_> = Region::new([-2; 3], [2; 3]).unwrap().cells().collect();
    // Explicit pools exercise Rayon even on single-CPU CI hosts. None covers
    // the fallback used when worker creation fails or only one CPU is available.
    let pools: Vec<_> = [1, 2, 8]
        .into_iter()
        .map(|threads| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap()
        })
        .collect();
    // Exercise the serial threshold and uneven work divisions, including empty
    // chunks, negative seams and AO. Compare raw floats rather than tolerances.
    for count in [0, 1, 3, 4, 7, 8, 27, 64] {
        let dirty = &dirty[..count];
        let expected: Vec<_> = dirty
            .iter()
            .map(|&at| mesh_chunk(at, &LocalCells::new(at, &snapshot), &palette).unwrap())
            .collect();
        for pool in std::iter::once(None).chain(pools.iter().map(Some)) {
            for _ in 0..3 {
                let actual = mesh_dirty_with_pool(dirty, &snapshot, &palette, pool);
                assert_eq!(actual.len(), expected.len());
                for (actual, expected) in actual.into_iter().zip(&expected) {
                    let actual = actual.unwrap();
                    assert_eq!(actual.exposed_faces, expected.exposed_faces);
                    assert_eq!(actual.vertices.len(), expected.vertices.len());
                    for (actual, expected) in actual.vertices.iter().zip(&expected.vertices) {
                        assert_eq!(
                            actual.position.map(f32::to_bits),
                            expected.position.map(f32::to_bits)
                        );
                        assert_eq!(
                            actual.normal.map(f32::to_bits),
                            expected.normal.map(f32::to_bits)
                        );
                        assert_eq!(
                            actual.color_ao.map(f32::to_bits),
                            expected.color_ao.map(f32::to_bits)
                        );
                    }
                }
            }
        }
    }
}
