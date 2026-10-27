use super::*;
use crate::{Origin, Swatch};

fn previous() -> Document {
    let mut doc = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    doc.content.bounds = Region::new([0; 3], [31; 3]).unwrap();
    doc.content.palette =
        Palette::from_colors(vec![Swatch::default(), Swatch::rgb(1, 2, 3)]).unwrap();
    doc.content.layers[0].voxels.set([0; 3], 1);
    doc.content.layers[0].voxels.set([30; 3], 2);
    doc.validate().unwrap();
    doc
}

fn assert_invalid(doc: &Document, previous: &Document, expected: Error) {
    assert_eq!(doc.validate(), Err(expected.clone()));
    assert_eq!(doc.validate_against(previous), Err(expected));
}

#[test]
fn edited_and_new_chunks_are_validated_even_in_hidden_locked_layers() {
    let previous = previous();
    for (at, slot, error) in [
        ([31, 30, 30], 1, Error::OutOfBounds([31, 30, 30])),
        (
            [30; 3],
            3,
            Error::Invalid("undefined palette slot 3".into()),
        ),
        (
            [16, 0, 0],
            3,
            Error::Invalid("undefined palette slot 3".into()),
        ),
        ([-1, 0, 0], 1, Error::OutOfBounds([-1, 0, 0])),
    ] {
        let mut doc = previous.clone();
        doc.content.layers[0].visible = false;
        doc.content.layers[0].locked = true;
        doc.content.layers[0].voxels.set(at, slot);
        assert_invalid(&doc, &previous, error);
    }
}

#[test]
fn changed_bounds_and_palette_recheck_shared_chunks() {
    let previous = previous();
    let mut doc = previous.clone();
    doc.content.bounds.max = [16; 3];
    assert_invalid(&doc, &previous, Error::OutOfBounds([30; 3]));
    let mut doc = previous.clone();
    doc.content.palette.slots.truncate(1);
    doc.content.palette.order.truncate(1);
    assert_invalid(
        &doc,
        &previous,
        Error::Invalid("undefined palette slot 2".into()),
    );
}

#[test]
fn new_layers_are_fully_checked_and_existing_layers_match_by_id() {
    let previous = previous();
    for (at, slot, error) in [
        ([31; 3], 1, Error::OutOfBounds([31; 3])),
        ([0; 3], 3, Error::Invalid("undefined palette slot 3".into())),
    ] {
        let mut doc = previous.clone();
        let mut layer = Layer::new(Uuid::from_u128(3), "New layer");
        layer.voxels.set(at, slot);
        doc.content.layers.insert(0, layer);
        assert_invalid(&doc, &previous, error);
    }
    let mut previous = previous;
    let mut layer = Layer::new(Uuid::from_u128(3), "Other layer");
    layer.voxels.set([2; 3], 2);
    previous.content.layers.push(layer);
    previous.validate().unwrap();
    let mut doc = previous.clone();
    doc.content.layers.reverse();
    doc.content.layers[1].voxels.set([30; 3], 0);
    assert_eq!(doc.validate_against(&previous), Ok(()));
    doc.content.layers[0].voxels.set([2; 3], 3);
    assert_invalid(
        &doc,
        &previous,
        Error::Invalid("undefined palette slot 3".into()),
    );
}

#[test]
fn incremental_validation_matches_full_error_order() {
    let previous = previous();
    let mut doc = previous.clone();
    doc.content.layers[0].voxels.set([0; 3], 3);
    doc.content.layers[0].voxels.set([30; 3], 4);
    doc.content.layers[0].voxels.set([-1; 3], 1);
    assert!(doc.validate().is_err());
    assert_eq!(doc.validate_against(&previous), doc.validate());
}

#[test]
fn independently_built_equal_chunks_and_removed_chunks_are_valid() {
    let previous = previous();
    let mut doc = previous.clone();
    doc.content.layers[0].voxels = Volume::default();
    for (p, s) in previous.content.layers[0].voxels.iter() {
        doc.content.layers[0].voxels.set(p, s);
    }
    assert_eq!(doc.validate_against(&previous), Ok(()));
    doc.content.layers[0].voxels.set([30; 3], 0);
    assert_eq!(doc.validate_against(&previous), Ok(()));
    assert_eq!(doc.validate(), Ok(()));
}

#[test]
fn incremental_validation_retains_metadata_and_version_checks() {
    let previous = previous();
    let mut doc = previous.clone();
    doc.content.layers.push(doc.content.layers[0].clone());
    assert_invalid(&doc, &previous, Error::Invalid("duplicate layer ID".into()));
    let mut doc = previous.clone();
    doc.content.extra.insert("extensions".into(), true.into());
    assert_invalid(
        &doc,
        &previous,
        Error::Invalid("extensions must use the project namespace".into()),
    );
    let mut doc = previous.clone();
    let id = Uuid::from_u128(10);
    doc.versions.insert(
        id,
        NamedVersion {
            id,
            name: "Version".into(),
            parent: None,
            origin: Origin::Human,
            content_hash: doc.content.content_hash().unwrap(),
            snapshot: Arc::new(doc.content.clone()),
            extra: Extra::new(),
        },
    );
    doc.validate().unwrap();
    let previous = doc.clone();
    doc.versions.get_mut(&id).unwrap().content_hash.clear();
    assert_invalid(
        &doc,
        &previous,
        Error::Invalid("version snapshot hash mismatch".into()),
    );
    let mut doc = previous.clone();
    Arc::make_mut(&mut doc.versions.get_mut(&id).unwrap().snapshot).layers[0]
        .voxels
        .set([0; 3], 3);
    assert_invalid(
        &doc,
        &previous,
        Error::Invalid("undefined palette slot 3".into()),
    );
}

#[test]
fn incremental_validation_counts_all_shared_voxels_and_chunks() {
    let mut previous = previous();
    previous.content.bounds = Region::new([0; 3], [256; 3]).unwrap();
    previous.content.layers[0].voxels = Volume::default();
    for p in previous.content.bounds.cells().take(MAX_VOXELS / 64 + 1) {
        previous.content.layers[0].voxels.set(p, 1);
    }
    previous.validate().unwrap();
    let mut doc = previous.clone();
    for i in 3..66 {
        let mut layer = previous.content.layers[0].clone();
        layer.id = Uuid::from_u128(i);
        doc.content.layers.push(layer);
    }
    assert_eq!(doc.content.layers.len(), 64);
    assert_invalid(
        &doc,
        &previous,
        Error::Limit("document voxel/chunk budget".into()),
    );

    previous.content.layers[0].voxels = Volume::default();
    for p in previous
        .content
        .bounds
        .cells()
        .filter(|p| p.iter().all(|c| c % 16 == 0))
        .take(MAX_CHUNKS / 64 + 1)
    {
        previous.content.layers[0].voxels.set(p, 1);
    }
    previous.validate().unwrap();
    let mut doc = previous.clone();
    for i in 3..66 {
        let mut layer = previous.content.layers[0].clone();
        layer.id = Uuid::from_u128(i);
        doc.content.layers.push(layer);
    }
    assert_invalid(
        &doc,
        &previous,
        Error::Limit("document voxel/chunk budget".into()),
    );
}
