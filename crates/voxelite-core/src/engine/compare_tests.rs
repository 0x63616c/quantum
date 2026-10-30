use super::*;
use proptest::prelude::*;

// Original full-volume implementation, retained as an independent oracle.
fn reference_compare(a: &Snapshot, b: &Snapshot) -> Changes {
    let mut out = Changes {
        palette_changed: a.palette != b.palette,
        metadata_changed: a.name != b.name
            || a.bounds != b.bounds
            || a.comments != b.comments
            || a.extra != b.extra
            || a.layers.iter().map(|l| l.id).collect::<Vec<_>>()
                != b.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        ..Changes::default()
    };
    let ids: BTreeSet<_> = a.layers.iter().chain(&b.layers).map(|l| l.id).collect();
    for id in ids {
        let before = a.layers.iter().find(|l| l.id == id);
        let after = b.layers.iter().find(|l| l.id == id);
        if before == after {
            continue;
        }
        out.layers.insert(id);
        let coords: BTreeSet<_> = before
            .into_iter()
            .flat_map(|l| l.voxels.iter().map(|(p, _)| p))
            .chain(
                after
                    .into_iter()
                    .flat_map(|l| l.voxels.iter().map(|(p, _)| p)),
            )
            .collect();
        for p in coords {
            let old = before.map_or(0, |l| l.voxels.get(p));
            let new = after.map_or(0, |l| l.voxels.get(p));
            if old == new {
                continue;
            }
            if old == 0 {
                out.added += 1;
            } else if new == 0 {
                out.removed += 1;
            } else {
                out.recolored += 1;
            }
            if let Some(r) = &mut out.bounds {
                for (axis, value) in p.iter().enumerate() {
                    r.min[axis] = r.min[axis].min(*value);
                    r.max[axis] = r.max[axis].max(*value + 1);
                }
            } else {
                out.bounds = Some(Region {
                    min: p,
                    max: p.map(|v| v + 1),
                });
            }
        }
    }
    out
}

fn assert_matches_reference(a: &Snapshot, b: &Snapshot) {
    assert_eq!(compare(a, b), reference_compare(a, b));
    assert_eq!(compare(b, a), reference_compare(b, a));
}

#[test]
fn compare_dense_sparse_equal_and_edited_chunks() {
    let mut sparse = Document::new(Uuid::from_u128(1), Uuid::from_u128(2)).content;
    for i in 0..200 {
        sparse.layers[0].voxels.set([i % 16, i / 16, 0], 1);
    }
    sparse.layers[0].voxels.set([-17, 0, -1], 1);
    let mut dense = sparse.clone();
    // Promotion at 256 followed by removal leaves 200 cells in dense storage.
    for i in 200..256 {
        dense.layers[0].voxels.set([i % 16, i / 16, 0], 1);
    }
    for i in 200..256 {
        dense.layers[0].voxels.set([i % 16, i / 16, 0], 0);
    }
    assert!(!sparse.layers[0].voxels.chunk([0; 3]).unwrap().is_dense());
    assert!(dense.layers[0].voxels.chunk([0; 3]).unwrap().is_dense());
    assert_eq!(sparse, dense);
    assert_matches_reference(&sparse, &dense);
    assert_eq!(compare(&sparse, &dense), Changes::default());

    let mut independent = dense.clone();
    independent.layers[0].voxels = Volume::default();
    for (p, slot) in dense.layers[0].voxels.iter() {
        independent.layers[0].voxels.set(p, slot);
    }
    assert_matches_reference(&dense, &independent);
    independent.layers[0].name = "Metadata only".into();
    assert_matches_reference(&dense, &independent);
    assert_eq!(compare(&dense, &independent).voxel_count(), 0);

    let mut edited = dense.clone();
    edited.layers[0].voxels.set([0, 0, 0], 0);
    edited.layers[0].voxels.set([1, 0, 0], 2);
    edited.layers[0].voxels.set([17, 0, 0], 1);
    edited.layers[0].voxels.set([-17, 0, -1], 0);
    assert_matches_reference(&dense, &edited);
    assert_matches_reference(&sparse, &edited);
    let changes = compare(&dense, &edited);
    assert_eq!(
        (changes.added, changes.removed, changes.recolored),
        (1, 2, 1)
    );
    assert_eq!(
        changes.bounds,
        Some(Region {
            min: [-17, 0, -1],
            max: [18, 1, 1]
        })
    );

    let mut added = dense.clone();
    let mut layer = sparse.layers[0].clone();
    layer.id = Uuid::from_u128(3);
    added.layers.push(layer);
    added.layers.push(Layer::new(Uuid::from_u128(4), "Empty"));
    assert_matches_reference(&dense, &added);
    assert_eq!(compare(&dense, &added).added, 201);
    let mut empty = dense.clone();
    empty.layers.clear();
    assert_matches_reference(&empty, &added);
}

proptest! {
    #[test]
    fn compare_matches_full_volume_reference(
        cells in prop::collection::vec((0usize..4, -32i32..32, 0i32..32, -32i32..32, 0u16..4), 0..180),
        edits in prop::collection::vec((0usize..4, -32i32..32, 0i32..32, -32i32..32, 0u16..4), 0..90),
        before_mask in 0u8..16,
        after_mask in 0u8..16,
        rebuild in any::<bool>(),
        metadata in any::<u8>(),
    ) {
        let mut before = Document::new(Uuid::from_u128(1), Uuid::from_u128(2)).content;
        before.layers = (0..4).map(|i| Layer::new(Uuid::from_u128(i + 2), "Layer")).collect();
        for (layer, x, y, z, slot) in cells {
            before.layers[layer].voxels.set([x, y, z], slot);
        }
        let mut after = before.clone();
        if rebuild {
            for layer in &mut after.layers {
                let mut voxels = Volume::default();
                for (p, slot) in layer.voxels.iter() {
                    voxels.set(p, slot);
                }
                layer.voxels = voxels;
            }
        }
        for (layer, x, y, z, slot) in edits {
            after.layers[layer].voxels.set([x, y, z], slot);
        }
        if metadata & 1 != 0 { after.layers[0].name = "Renamed".into(); }
        if metadata & 2 != 0 { after.layers[1].visible = false; }
        if metadata & 4 != 0 { after.layers[2].locked = true; }
        if metadata & 8 != 0 { after.layers[3].extra.insert("custom".into(), true.into()); }
        before.layers.retain(|l| before_mask & (1 << (l.id.as_u128() - 2)) != 0);
        after.layers.retain(|l| after_mask & (1 << (l.id.as_u128() - 2)) != 0);
        if metadata & 16 != 0 { after.layers.reverse(); }
        if metadata & 32 != 0 { after.palette.slots[0].color = Swatch::rgb(1, 2, 3); }
        if metadata & 64 != 0 { after.name = "Renamed document".into(); }
        if metadata & 128 != 0 { after.bounds.max[0] += 1; }
        assert_matches_reference(&before, &after);
        assert_matches_reference(&before, &before.clone());
    }
}
