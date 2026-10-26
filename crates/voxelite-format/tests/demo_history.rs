//! The original artwork is frozen independently of the regenerated demo file.
#[path = "../../../examples/support/autumn_workshop.rs"]
mod autumn_workshop;

use std::collections::BTreeSet;
use voxelite_core::*;
use voxelite_format::*;

const DEMO: &[u8] = include_bytes!("../../../examples/autumn-workshop.voxl");
// Captured from the bundled asset at 5afad1f, before adding any build history.
// Snapshot hash includes every layer-cell, palette slot/order, metadata and comment.
const ORIGINAL_CONTENT_HASH: &str =
    "5d86c79aa8831a0ece6614b76a00845d6881f34135d8578cbf97bbb2d176bd39";
// Project hash additionally freezes document identity, lineage and named versions.
const ORIGINAL_PROJECT_HASH: &str =
    "ed053f9e9a8e5d0e4f407ae31eb88f829f4ad05af43a505f8c6428423f895d86";

#[test]
fn generated_demo_is_identical_to_original_and_reproducible() {
    let generated = autumn_workshop::build().unwrap();
    assert_eq!(
        generated.document.content.content_hash().unwrap(),
        ORIGINAL_CONTENT_HASH
    );
    assert_eq!(generated.semantic_hash().unwrap(), ORIGINAL_PROJECT_HASH);
    let bytes = generated.to_bytes().unwrap();
    assert_eq!(bytes, DEMO, "regenerate with the demo-history example");
    let reopened = from_bytes(&bytes).unwrap();
    assert_eq!(reopened.document, generated.document);
    assert_eq!(reopened.to_bytes().unwrap(), bytes);
    assert_eq!(autumn_workshop::build().unwrap().to_bytes().unwrap(), bytes);
}

#[test]
fn stored_demo_scrubs_undoes_and_redoes_every_real_edit() {
    let project = from_bytes(DEMO).unwrap();
    let recipe = project.build_history().unwrap();
    assert!((20..=60).contains(&recipe.steps.len()));
    let mut engine = project
        .open_engine(Limits {
            history_bytes: 32 * 1024 * 1024,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(engine.history().count(), recipe.steps.len());
    assert!(engine.initial_retained());
    assert!(!engine.is_dirty());
    let initial = engine.preview(None).unwrap();
    assert_eq!(
        *initial,
        Document::new(project.document.id, recipe.base_layer)
    );
    let revision = engine.revision();
    let head = engine.head();
    let mut previous = initial.clone();
    let mut labels = BTreeSet::new();
    let mut added = 0;
    let mut erased = 0;
    let mut painted = 0;
    for (index, (step, entry)) in recipe.steps.iter().zip(engine.history()).enumerate() {
        assert_eq!(entry.id, step.id);
        assert_eq!(entry.label, step.label);
        assert!(labels.insert(&step.label), "duplicate label");
        assert_eq!(step.elapsed_ms, index as u32 * 60_000);
        assert_eq!(entry.result_revision, entry.base_revision + 1);
        let preview = engine.preview(Some(entry.id)).unwrap();
        assert_ne!(preview, previous, "empty history entry: {}", entry.label);
        previous = preview;
        added += usize::from(entry.changes.added > 0);
        erased += usize::from(entry.changes.removed > 0);
        if matches!(
            step.command,
            Command::SetVoxels {
                mode: Mode::Paint,
                ..
            }
        ) {
            assert!(entry.changes.recolored > 0);
            assert_eq!(entry.changes.added + entry.changes.removed, 0);
            painted += 1;
        }
    }
    assert!(added >= 25 && erased >= 1 && painted >= 3);
    assert_eq!(engine.revision(), revision);
    assert_eq!(engine.head(), head);
    assert_eq!(engine.document(), &project.document);
    assert!(!engine.is_dirty());
    for step in recipe.steps.iter().rev() {
        engine.undo(engine.revision(), step.id).unwrap();
        assert_eq!(
            engine.document(),
            engine.preview(engine.head()).unwrap().as_ref()
        );
    }
    assert_eq!(engine.document(), initial.as_ref());
    assert!(engine.is_dirty());
    for step in &recipe.steps {
        engine.redo(engine.revision(), step.id).unwrap();
        assert_eq!(
            engine.document(),
            engine.preview(Some(step.id)).unwrap().as_ref()
        );
    }
    assert_eq!(engine.document(), &project.document);
    assert!(!engine.is_dirty());
}

#[test]
fn smaller_host_budget_can_evict_history_without_rejecting_the_demo() {
    let project = from_bytes(DEMO).unwrap();
    let budget = 16 * 1024 * 1024;
    let engine = project
        .open_engine(Limits {
            history_bytes: budget,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(engine.document(), &project.document);
    assert!(engine.history_bytes() <= budget);
    assert!(!engine.initial_retained());
    assert!(engine.history().count() > 0);
    assert!(!engine.is_dirty());
}
