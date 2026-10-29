use super::*;
use quantum_plugin_api::{PaletteEdit, PaletteOperation};

fn edit(operation: PaletteOperation, name: &str, color: &str) -> PaletteAction {
    PaletteAction::Edit {
        operation,
        name: name.into(),
        color: color.into(),
    }
}

#[test]
fn library_persistence_restart_and_failures_preserve_the_original_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("palettes.json");
    let mut editor = Editor::blank().unwrap();
    let before = editor.project().to_bytes().unwrap();
    editor
        .palette_action(PaletteAction::ChoosePreset(3))
        .unwrap();
    let save = |bytes: &[u8]| local_settings::write_local(&path, bytes);
    editor
        .palette_action_with(edit(PaletteOperation::Duplicate, "Studio", ""), save)
        .unwrap();
    let saved = editor.palette_view().unwrap();
    assert_eq!(saved.name, "Studio copy");
    assert_eq!(saved.palette.slots.len(), 16);
    let id = saved.state.custom.unwrap();
    let bytes = std::fs::read(&path).unwrap();
    for (operation, name, color) in [
        (PaletteOperation::Rename, "", ""),
        (PaletteOperation::ApplyColor, "", "#xyz000"),
    ] {
        assert!(
            editor
                .palette_action_with(edit(operation, name, color), save)
                .is_err()
        );
        assert_eq!(editor.palette_view().unwrap(), saved);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    assert!(
        editor
            .palette_action_with(
                edit(PaletteOperation::Rename, "Should not install", ""),
                |bytes| local_settings::write_local(dir.path(), bytes)
            )
            .is_err()
    );
    assert_eq!(editor.palette_view().unwrap(), saved);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let mut restarted = Editor::blank().unwrap();
    restarted
        .palette_action(PaletteAction::LoadLibrary(Some(bytes)))
        .unwrap();
    restarted
        .palette_action(PaletteAction::ChooseCustom(id))
        .unwrap();
    assert_eq!(restarted.palette_view().unwrap().palette, saved.palette);
    assert_eq!(restarted.palette_view().unwrap().name, saved.name);
    editor
        .palette_action_with(edit(PaletteOperation::Delete, "", ""), save)
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "{\n  \"entries\": []\n}"
    );
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    assert_eq!(editor.engine.history().count(), 0);
}

#[test]
fn selection_only_changes_new_voxels_and_undo_redo_preserve_exact_document_bytes() {
    let mut editor = Editor::blank().unwrap();
    editor.apply_at([1, 1, 1]).unwrap();
    let before = editor.project().to_bytes().unwrap();
    let history = editor.engine.history().count();
    let color = editor.color.clone();
    editor
        .palette_action(PaletteAction::ChoosePreset(1))
        .unwrap();
    assert_eq!(editor.color, color);
    editor
        .palette_action_with(PaletteAction::SelectSwatch(12), |_| Ok(()))
        .unwrap();
    assert_eq!(editor.color.rgba, [41, 173, 255, 255]);
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    assert_eq!(editor.engine.history().count(), history);
    editor.apply_at([2, 1, 1]).unwrap();
    let after = editor.project().to_bytes().unwrap();
    assert_eq!(editor.snapshot().palette.get(1), Some(&color));
    assert_eq!(editor.snapshot().palette.get(2), Some(&editor.color));
    editor.undo().unwrap();
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    editor.redo().unwrap();
    assert_eq!(editor.project().to_bytes().unwrap(), after);
}

#[test]
fn disablement_and_absence_remove_panel_commands_and_use_the_core_tool_color() {
    let mut editor = Editor::blank().unwrap();
    editor
        .palette_action(PaletteAction::ChoosePreset(2))
        .unwrap();
    editor
        .palette_action_with(PaletteAction::SelectSwatch(5), |_| Ok(()))
        .unwrap();
    let selected = editor.color.clone();
    let before = editor.project().to_bytes().unwrap();
    editor
        .set_plugin_enabled("dev.quantum.palette", false)
        .unwrap();
    assert!(editor.palette_view().is_none());
    assert!(!editor.palette_enabled());
    assert!(
        editor
            .plugin_host()
            .registry()
            .commands()
            .all(|(_, c)| !c.id.starts_with("dev.quantum.palette."))
    );
    assert_eq!(editor.active_color(), &Swatch::default());
    assert!(
        editor
            .palette_action_with(edit(PaletteOperation::Create, "", ""), |_| panic!(
                "disabled library cannot write"
            ))
            .unwrap()
            .is_none()
    );
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    assert!(editor.apply_at([1, 1, 1]).unwrap());
    assert_eq!(editor.snapshot().palette.slots.len(), 1);
    let placed = editor.project().to_bytes().unwrap();
    editor.undo().unwrap();
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    editor.redo().unwrap();
    assert_eq!(editor.project().to_bytes().unwrap(), placed);
    editor
        .set_plugin_enabled("dev.quantum.palette", true)
        .unwrap();
    assert_eq!(editor.palette_view().unwrap().preset, 2);
    assert_eq!(editor.active_color(), &selected);
    editor.plugins = std::cell::RefCell::new(quantum_plugin_host::Host::new(vec![Box::new(
        quantum_plugin_tools::ToolsPlugin,
    )]));
    assert_eq!(editor.active_color(), &Swatch::default());
    assert!(editor.apply_at([2, 1, 1]).unwrap());
    assert_eq!(editor.snapshot().palette.slots.len(), 1);
}

#[test]
fn typed_document_edits_remain_core_validated_and_atomic() {
    let mut editor = Editor::blank().unwrap();
    let before = editor.project().to_bytes().unwrap();
    let state = editor.palette_view().unwrap();
    assert!(
        editor
            .palette_action(PaletteAction::EditDocument(PaletteEdit::Remove {
                slot: 1,
                replacement: None
            }))
            .is_err()
    );
    assert_eq!(editor.palette_view().unwrap(), state);
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    editor
        .palette_action(PaletteAction::AppendImport {
            format: "hex".into(),
            text: "112233\n445566".into(),
        })
        .unwrap();
    let after = editor.project().to_bytes().unwrap();
    assert_eq!(editor.snapshot().palette.slots.len(), 3);
    editor.undo().unwrap();
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    editor.redo().unwrap();
    assert_eq!(editor.project().to_bytes().unwrap(), after);
}

#[test]
fn picker_preview_and_recent_colors_are_adopted_only_after_successful_persistence() {
    let mut editor = Editor::blank().unwrap();
    let original = editor.palette_view().unwrap();
    let brush = editor.color.clone();
    for value in 0..60 {
        editor
            .palette_action_with(
                PaletteAction::SetColor {
                    rgb: [value; 3],
                    commit: false,
                },
                |_| panic!("preview cannot write"),
            )
            .unwrap();
    }
    let preview = editor.palette_view().unwrap();
    assert_eq!(preview.state.library.entries.len(), 1);
    assert!(preview.state.recent_colors.is_empty());
    assert_eq!(editor.color, brush);
    let failed = |_: &[u8]| Err(std::io::Error::other("write failed").into());
    assert!(
        editor
            .palette_action_with(PaletteAction::CommitColor, failed)
            .is_err()
    );
    assert_eq!(editor.palette_view().unwrap(), preview);
    let mut writes = 0;
    let committed = editor
        .palette_action_with(PaletteAction::CommitColor, |_| {
            writes += 1;
            Ok(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(writes, 1);
    assert_eq!(committed.state.recent_colors, [[59; 3]]);
    assert!(committed.state.preview.is_none());
    assert_eq!(editor.color, brush);
    editor
        .palette_action(PaletteAction::ChoosePreset(0))
        .unwrap();
    let before_brush = editor.palette_view().unwrap();
    assert!(
        editor
            .palette_action_with(PaletteAction::SelectSwatch(0), failed)
            .is_err()
    );
    assert_eq!(editor.palette_view().unwrap(), before_brush);
    assert_eq!(editor.color, brush);
    assert_eq!(editor.snapshot().palette, original.palette);
    assert_eq!(editor.engine.history().count(), 0);
}

#[test]
fn manager_copy_cancel_commit_and_interchange_keep_the_document_and_brush_untouched() {
    use quantum_plugin_api::{PaletteId, RecentColorTarget};
    let mut editor = Editor::blank().unwrap();
    let document = editor.project().to_bytes().unwrap();
    let brush = editor.color.clone();
    editor
        .palette_action(PaletteAction::ChoosePalette(PaletteId::Source(1)))
        .unwrap();
    let original = editor.palette_view().unwrap();
    let preview = editor
        .palette_action_with(
            PaletteAction::SetColor {
                rgb: [29, 132, 218],
                commit: false,
            },
            |_| panic!("preview must not persist"),
        )
        .unwrap()
        .unwrap();
    assert_eq!(preview.copied_from.as_deref(), Some(original.name.as_str()));
    assert!(!editor.palette_view().unwrap().read_only);
    editor
        .palette_action_with(PaletteAction::CancelColor, |_| {
            panic!("cancel must not persist")
        })
        .unwrap();
    assert_eq!(editor.palette_view().unwrap(), original);

    let mut persisted = Vec::new();
    let mut request = |editor: &mut Editor, action| {
        editor
            .palette_action_with(action, |bytes| {
                persisted = bytes.to_vec();
                Ok(())
            })
            .unwrap()
            .unwrap()
    };
    let copy = request(
        &mut editor,
        PaletteAction::SetColor {
            rgb: [29, 132, 218],
            commit: true,
        },
    );
    assert_eq!(copy.copied_from.as_deref(), Some(original.name.as_str()));
    let custom = editor.palette_view().unwrap();
    request(
        &mut editor,
        PaletteAction::DuplicateSwatch(custom.state.slot),
    );
    let duplicated = editor.palette_view().unwrap();
    assert_eq!(duplicated.state.slot, custom.state.slot + 1);
    request(
        &mut editor,
        PaletteAction::UseRecentColor {
            index: 0,
            target: RecentColorTarget::NewSwatch { at: None },
        },
    );
    let added = editor.palette_view().unwrap();
    assert_eq!(added.palette.order.len(), original.palette.order.len() + 2);
    assert_eq!(added.state.slot, added.palette.order.len() - 1);
    assert_eq!(added.state.recent_colors, [[29, 132, 218]]);
    request(
        &mut editor,
        PaletteAction::UseRecentColor {
            index: 0,
            target: RecentColorTarget::SelectedSwatch,
        },
    );
    for format in &added.formats {
        let export = request(
            &mut editor,
            PaletteAction::ExportLibrary {
                format: format.id.into(),
            },
        )
        .export
        .unwrap();
        request(
            &mut editor,
            PaletteAction::ImportLibrary {
                name: format!("Imported {}", format.label),
                format: format.id.into(),
                text: export.text,
            },
        );
        let imported = editor.palette_view().unwrap();
        let colors = |palette: &quantum_plugin_api::Palette| {
            palette
                .order
                .iter()
                .map(|slot| palette.get(*slot).unwrap().rgba)
                .collect::<Vec<_>>()
        };
        assert_eq!(colors(&imported.palette), colors(&added.palette));
        request(&mut editor, PaletteAction::ChoosePalette(added.selected));
    }
    let mut restarted = Editor::blank().unwrap();
    restarted
        .palette_action(PaletteAction::LoadLibrary(Some(persisted)))
        .unwrap();
    restarted
        .palette_action(PaletteAction::ChoosePalette(added.selected))
        .unwrap();
    assert_eq!(restarted.palette_view().unwrap().palette, added.palette);
    assert_eq!(
        restarted.palette_view().unwrap().state.recent_colors,
        added.state.recent_colors
    );
    assert_eq!(editor.project().to_bytes().unwrap(), document);
    assert_eq!(editor.color, brush);
    assert_eq!(editor.engine.history().count(), 0);
}

#[test]
fn manager_selection_tracks_identity_through_every_remove_and_move() {
    let mut editor = Editor::blank().unwrap();
    let mut saved = Vec::new();
    editor
        .palette_action_with(
            PaletteAction::ImportLibrary {
                name: "Order study".into(),
                format: "hex".into(),
                text: "110000\n220000\n330000\n440000\n550000\n660000\n770000".into(),
            },
            |bytes| {
                saved = bytes.to_vec();
                Ok(())
            },
        )
        .unwrap();
    let original = editor.palette_view().unwrap();
    let reset = |editor: &mut Editor, selected| {
        editor
            .palette_action(PaletteAction::LoadLibrary(Some(saved.clone())))
            .unwrap();
        editor
            .palette_action(PaletteAction::ChoosePalette(original.selected))
            .unwrap();
        editor
            .palette_action(PaletteAction::SelectSlot(selected))
            .unwrap();
    };
    for selected in 0..7 {
        for from in 0..7 {
            for to in 0..=8 {
                reset(&mut editor, selected);
                editor
                    .palette_action_with(PaletteAction::MoveSwatch { from, to }, |_| Ok(()))
                    .unwrap();
                let view = editor.palette_view().unwrap();
                assert_eq!(
                    view.palette.order[view.state.slot],
                    original.palette.order[selected]
                );
                let mut expected = original.palette.order.clone();
                let moved = expected.remove(from);
                expected.insert(to.min(6), moved);
                assert_eq!(view.palette.order, expected);
                assert_eq!(view.palette.slots, original.palette.slots);
            }
            reset(&mut editor, selected);
            editor
                .palette_action_with(PaletteAction::RemoveSwatch(from), |_| Ok(()))
                .unwrap();
            let view = editor.palette_view().unwrap();
            if selected == from {
                assert_eq!(view.state.slot, from.min(5));
            } else {
                assert_eq!(
                    view.palette.order[view.state.slot],
                    original.palette.order[selected]
                );
            }
        }
    }
}

#[test]
fn manager_capacity_and_last_swatch_errors_are_atomic() {
    let mut editor = Editor::blank().unwrap();
    editor
        .palette_action_with(edit(PaletteOperation::Create, "Single", "#123456"), |_| {
            Ok(())
        })
        .unwrap();
    let single = editor.palette_view().unwrap();
    let error = editor
        .palette_action_with(PaletteAction::RemoveSwatch(0), |_| {
            panic!("invalid edit must not write")
        })
        .unwrap_err();
    assert!(matches!(
        &error,
        EditorError::Palette(quantum_plugin_api::PaletteError::Message(_))
    ));
    assert!(error.to_string().contains("Cannot remove the last swatch"));
    assert_eq!(editor.palette_view().unwrap(), single);
    editor
        .palette_action_with(
            PaletteAction::ImportLibrary {
                name: "Full".into(),
                format: "hex".into(),
                text: (0..=255)
                    .map(|v| format!("{v:02X}{v:02X}{v:02X}\n"))
                    .collect(),
            },
            |_| Ok(()),
        )
        .unwrap();
    let full = editor.palette_view().unwrap();
    for action in [
        PaletteAction::AddSwatch {
            rgb: [1, 2, 3],
            at: None,
        },
        PaletteAction::DuplicateSwatch(0),
    ] {
        let error = editor
            .palette_action_with(action, |_| panic!("capacity error must not write"))
            .unwrap_err();
        assert!(matches!(
            &error,
            EditorError::Palette(quantum_plugin_api::PaletteError::Message(_))
        ));
        assert!(error.to_string().contains("256 swatches maximum"));
        assert_eq!(editor.palette_view().unwrap(), full);
    }
}
