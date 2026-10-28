use super::{strategies::*, *};
use proptest::prelude::*;
use std::sync::Arc;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn mixed_commands_round_trip_exact_documents(
        initial in document(), steps in prop::collection::vec(step(), 1..20),
    ) {
        let mut e = Engine::new(initial.clone()).unwrap();
        let mut states = vec![e.snapshot()];
        let mut entries = Vec::new();
        for (i, step) in steps.iter().enumerate() {
            let before = e.snapshot();
            let revision = e.revision();
            let bytes = e.history_bytes();
            let command = step.command(&e, i as u128);
            let encoded = serde_json::to_vec(&command).unwrap();
            prop_assert_eq!(serde_json::from_slice::<Command>(&encoded).unwrap(), command.clone());
            let tx = transaction(&e, i as u128, command);
            let preflight = e.validate(&tx).unwrap();
            prop_assert_eq!(e.snapshot(), before.clone());
            prop_assert_eq!(e.revision(), revision);
            prop_assert_eq!(e.history_bytes(), bytes);
            let actor = tx.actor.clone();
            let receipt = e.execute(tx).unwrap();
            prop_assert_eq!(receipt.changes, preflight.changes);
            prop_assert_eq!(receipt.work_units, preflight.work_units);
            prop_assert_eq!(receipt.entry, preflight.entry);
            prop_assert!(e.document().validate().is_ok());
            if let Some(entry) = receipt.entry {
                let after = e.snapshot();
                prop_assert_ne!(&before, &after);
                prop_assert_eq!(e.revision(), revision + 1);
                prop_assert_eq!(&e.history().next_back().unwrap().actor, &actor);
                let bytes = e.history_bytes();
                e.undo(e.revision(), entry).unwrap();
                prop_assert_eq!(e.snapshot(), before);
                prop_assert_eq!(e.revision(), revision + 2);
                e.redo(e.revision(), entry).unwrap();
                prop_assert_eq!(e.snapshot(), after.clone());
                prop_assert_eq!(e.revision(), revision + 3);
                prop_assert_eq!(e.history_bytes(), bytes);
                entries.push(entry);
                states.push(after);
            } else {
                prop_assert_eq!(e.snapshot(), before);
                prop_assert_eq!(e.revision(), revision);
                prop_assert_eq!(e.history_bytes(), bytes);
            }
            // Every changed transaction is one entry, including a multi-edit
            // batch. There is no implicit merging of edits by the same actor.
            prop_assert_eq!(e.history().map(|h| h.id).collect::<Vec<_>>(), entries.clone());
            prop_assert!(e.initial_retained());
        }
        for (i, entry) in entries.iter().enumerate() {
            prop_assert_eq!(e.preview(Some(*entry)).unwrap(), states[i + 1].clone());
        }
        for (i, entry) in entries.iter().enumerate().rev() {
            e.undo(e.revision(), *entry).unwrap();
            prop_assert_eq!(e.snapshot(), states[i].clone());
        }
        prop_assert_eq!(e.document(), &initial);
        prop_assert!(!e.is_dirty());
        prop_assert_eq!(e.head(), None);
        for (i, entry) in entries.iter().enumerate() {
            e.redo(e.revision(), *entry).unwrap();
            prop_assert_eq!(e.snapshot(), states[i + 1].clone());
        }
        prop_assert_eq!(e.redo_head(), None);
        // Held snapshots must remain immutable through all later edits.
        prop_assert_eq!(states[0].as_ref(), &initial);
    }

    #[test]
    fn history_budget_retains_whole_entries_and_reversible_suffix(
        initial in document(), capacity in 1usize..=4, count in 5usize..16,
    ) {
        // Renames leave charged document sizes constant. Measure one admission
        // through the public API instead of depending on a byte-count formula.
        let mut probe = Engine::new(initial.clone()).unwrap();
        execute(&mut probe, 0, rename("Probe"));
        let entry_bytes = probe.history_bytes();
        let budget = entry_bytes * capacity;
        let mut e = Engine::with_limits(initial, Limits { history_bytes: budget, ..Limits::default() }).unwrap();
        let mut states = vec![e.snapshot()];
        let mut entries = Vec::new();
        for i in 0..count {
            let receipt = execute(&mut e, i as u128, rename(&format!("Name {i}")));
            entries.push(receipt.entry.unwrap());
            states.push(e.snapshot());
            let retained = entries.len().min(capacity);
            prop_assert_eq!(e.history_bytes(), entry_bytes * retained);
            prop_assert!(e.history_bytes() <= budget);
            prop_assert_eq!(e.history().map(|h| h.id).collect::<Vec<_>>(), entries[entries.len() - retained..].to_vec());
            prop_assert_eq!(e.initial_retained(), entries.len() <= capacity);
        }
        prop_assert!(e.preview(None).is_err());
        for entry in &entries[..count - capacity] {
            prop_assert!(e.preview(Some(*entry)).is_err());
        }
        for i in (count - capacity..count).rev() {
            e.undo(e.revision(), entries[i]).unwrap();
            prop_assert_eq!(e.snapshot(), states[i].clone());
            prop_assert_eq!(e.history_bytes(), budget);
        }
        prop_assert_eq!(e.head(), None);
        prop_assert_eq!(e.snapshot(), states[count - capacity].clone());
        for i in count - capacity..count {
            e.redo(e.revision(), entries[i]).unwrap();
            prop_assert_eq!(e.snapshot(), states[i + 1].clone());
        }
    }

    #[test]
    fn rejected_batches_preserve_document_revision_and_redo(initial in document(), name in "[a-z]{1,16}") {
        let mut e = Engine::new(initial).unwrap();
        let receipt = execute(&mut e, 0, rename("Before undo"));
        e.undo(e.revision(), receipt.entry.unwrap()).unwrap();
        let before = e.snapshot();
        let revision = e.revision();
        let bytes = e.history_bytes();
        let command = Command::Batch(vec![rename(&name), Command::Layer(LayerEdit::Reorder(vec![]))]);
        prop_assert!(matches!(e.execute(transaction(&e, 1, command)), Err(Error::Batch { index: 1, .. })), "second batch command must fail");
        prop_assert!(Arc::ptr_eq(&e.snapshot(), &before));
        prop_assert_eq!(e.revision(), revision);
        prop_assert_eq!(e.head(), None);
        prop_assert_eq!(e.redo_head(), receipt.entry);
        prop_assert_eq!(e.history_bytes(), bytes);
        prop_assert!(!e.is_dirty());
    }
}
