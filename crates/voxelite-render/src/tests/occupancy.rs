//! Incremental occupancy must match a fresh visible composite, including uploads.
use crate::occupancy::{Occupancy, OccupancyCache};
use proptest::prelude::*;
use voxelite_core::*;
fn execute(engine: &mut Engine, command: Command) {
    engine
        .execute(Transaction {
            expected_revision: engine.revision(),
            id: Uuid::new_v4(),
            actor: Actor::new(Origin::Human),
            label: "Occupancy regression".into(),
            command,
        })
        .unwrap();
}
fn edit(engine: &mut Engine, layer: Uuid, at: Pos, slot: u16) {
    execute(
        engine,
        Command::SetVoxels {
            layer,
            voxels: vec![Voxel { at, slot }],
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
    );
}
fn sync(cache: &mut OccupancyCache, snapshot: &Snapshot, uploaded: &mut Vec<u32>) {
    cache.sync(snapshot).unwrap();
    let grid = cache.grid_mut().unwrap();
    let rebuilt = Occupancy::from_snapshot(snapshot).unwrap();
    // Words 12..20 hold the grow-only occupied box: an incremental grid may keep a
    // box larger than a fresh rebuild, but it must always contain it.
    let (a, b) = (grid.words(), rebuilt.words());
    assert_eq!(a[..12], b[..12]);
    assert_eq!(a[20..], b[20..]);
    for axis in 0..3 {
        let empty = b[12 + axis] > b[16 + axis];
        assert!(empty || a[12 + axis] <= b[12 + axis], "box min shrank");
        assert!(empty || a[16 + axis] >= b[16 + axis], "box max shrank");
    }
    if uploaded.len() != grid.words().len() {
        uploaded.resize(grid.words().len(), 0);
    }
    for range in grid.dirty_ranges() {
        uploaded[range.clone()].copy_from_slice(&grid.words()[range]);
    }
    assert_eq!(&uploaded[..], grid.words(), "dirty ranges missed a change");
    grid.clear_dirty();
}
#[test]
fn layers_overlap_hide_delete_reorder_recolor_resize_and_history() {
    let mut engine = Engine::new(Document::default()).unwrap();
    let id = engine.document().content.layers[0].id;
    edit(&mut engine, id, [-17, 0, 15], 1);
    edit(&mut engine, id, [16, 16, -16], 1);
    let mut cache = OccupancyCache::default();
    let mut uploaded = Vec::new();
    sync(&mut cache, &engine.document().content, &mut uploaded);
    let second = Uuid::new_v4();
    execute(
        &mut engine,
        Command::Layer(LayerEdit::Duplicate {
            source: id,
            id: second,
            name: "Overlap".into(),
        }),
    );
    cache.sync(&engine.document().content).unwrap();
    assert!(cache.grid().unwrap().dirty_ranges().is_empty());
    edit(&mut engine, id, [-17, 0, 15], 0);
    sync(&mut cache, &engine.document().content, &mut uploaded);
    assert!(cache.grid().unwrap().contains([-17, 0, 15]));
    let mut snapshot = engine.document().content.clone();
    snapshot.layers.reverse();
    cache.sync(&snapshot).unwrap();
    assert!(cache.grid().unwrap().dirty_ranges().is_empty());
    snapshot.palette.slots[0].color.rgba = [50, 80, 90, 255];
    cache.sync(&snapshot).unwrap();
    assert!(cache.grid().unwrap().dirty_ranges().is_empty());
    snapshot.layers[0].visible = false;
    sync(&mut cache, &snapshot, &mut uploaded);
    assert!(!cache.grid().unwrap().contains([-17, 0, 15]));
    snapshot.layers[0].visible = true;
    sync(&mut cache, &snapshot, &mut uploaded);
    snapshot.layers.remove(0);
    sync(&mut cache, &snapshot, &mut uploaded);
    assert!(!cache.grid().unwrap().contains([-17, 0, 15]));
    snapshot.bounds = Region::new([-30, 0, -30], [33, 63, 33]).unwrap();
    sync(&mut cache, &snapshot, &mut uploaded); // same brick count, different origin/size
    sync(&mut cache, &engine.document().content, &mut uploaded);
    let head = engine.head().unwrap();
    engine.undo(engine.revision(), head).unwrap();
    sync(&mut cache, &engine.document().content, &mut uploaded);
    engine.redo(engine.revision(), head).unwrap();
    sync(&mut cache, &engine.document().content, &mut uploaded);
    // A region refresh changes only that region and produces identical output.
    let mut region_grid = Occupancy::new(snapshot.bounds).unwrap();
    region_grid.update_region(&snapshot, snapshot.bounds);
    assert_eq!(
        region_grid.words(),
        Occupancy::from_snapshot(&snapshot).unwrap().words()
    );
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]
    #[test]
    fn incremental_random_edits_equal_rebuild(edits in prop::collection::vec((0usize..2, prop::array::uniform3(0i32..32), 0u16..=2, any::<bool>()), 1..70)) {
        let mut engine = Engine::new(Document::default()).unwrap();
        let first = engine.document().content.layers[0].id;
        execute(&mut engine, Command::Palette(PaletteEdit::Add { slot: Some(2), color: Swatch::rgb(50, 80, 90) }));
        let second = Uuid::new_v4();
        execute(&mut engine, Command::Layer(LayerEdit::Create { id: second, name: "Second".into(), index: 1 }));
        let mut cache = OccupancyCache::default();
        let mut uploaded = Vec::new();
        sync(&mut cache, &engine.document().content, &mut uploaded);
        for (layer, at, slot, visible) in edits {
            edit(&mut engine, [first, second][layer], at, slot);
            let mut snapshot = engine.document().content.clone();
            snapshot.layers[layer].visible = visible;
            sync(&mut cache, &snapshot, &mut uploaded);
            sync(&mut cache, &snapshot, &mut uploaded); // repeated / style-only snapshot
            prop_assert!(cache.grid().unwrap().dirty_ranges().is_empty());
        }
    }
}
