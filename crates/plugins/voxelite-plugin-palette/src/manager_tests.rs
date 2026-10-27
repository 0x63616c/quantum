use super::*;
use voxelite_plugin_api::*;

#[derive(Default)]
struct Context {
    state: PaletteState,
    palette: Palette,
    brush: Swatch,
}
impl PaletteCx for Context {
    fn palette(&self) -> &Palette {
        &self.palette
    }
    fn active_color(&self) -> &Swatch {
        &self.brush
    }
    fn state(&self) -> &PaletteState {
        &self.state
    }
    fn setting(&self, _: &str) -> Option<&SettingValue> {
        None
    }
}
impl Context {
    fn apply(&mut self, action: PaletteAction) -> PaletteOutcome {
        let out = PalettePlugin.interact(self, action).unwrap();
        self.state = out.state.clone();
        if let Some(color) = &out.color {
            self.brush = color.clone();
        }
        out
    }
    fn view(&self) -> PaletteView {
        PalettePlugin.view(self)
    }
    fn custom(palette: Palette) -> Self {
        let mut cx = Self::default();
        cx.state.custom =
            Some(library::create(&mut cx.state.library, "Test".into(), palette).unwrap());
        cx
    }
    fn rejects(&self, action: PaletteAction, message: &str) {
        assert!(
            PalettePlugin
                .interact(self, action)
                .unwrap_err()
                .to_string()
                .contains(message)
        );
    }
}

#[test]
fn drag_moves_preserve_every_slot_and_selected_identity_for_all_small_permutations() {
    fn permutations(ids: &mut [u16], start: usize, visit: &mut impl FnMut(&[u16])) {
        if start == ids.len() {
            visit(ids);
            return;
        }
        for i in start..ids.len() {
            ids.swap(start, i);
            permutations(ids, start + 1, visit);
            ids.swap(start, i);
        }
    }
    for count in 1..=4 {
        let palette =
            Palette::from_colors((0..count).map(|i| Swatch::rgb(i as u8, 0, 0)).collect()).unwrap();
        let mut order = palette.order.clone();
        permutations(&mut order, 0, &mut |order| {
            for from in 0..count {
                for to in (0..count).chain([usize::MAX]) {
                    for selected in 0..count {
                        let mut original = palette.clone();
                        original.order = order.to_vec();
                        let mut cx = Context::custom(original.clone());
                        cx.state.slot = selected;
                        let out = cx.apply(PaletteAction::MoveSwatch { from, to });
                        let actual = cx.view().palette;
                        actual.validate().unwrap();
                        assert_eq!(actual.slots, original.slots);
                        assert_eq!(actual.order[cx.state.slot], original.order[selected]);
                        let to = to.min(count - 1);
                        let mut expected = original.order.clone();
                        let moved = expected.remove(from);
                        expected.insert(to, moved);
                        assert_eq!(actual.order, expected);
                        assert_eq!(out.library_bytes.is_none(), from == to);
                    }
                }
            }
        });
    }
}

#[test]
fn removing_and_adding_use_display_positions_without_renumbering_survivors() {
    let mut palette = Palette::from_colors(vec![Swatch::default(); 4]).unwrap();
    palette.order = vec![4, 2, 1, 3];
    let mut cx = Context::custom(palette.clone());
    cx.apply(PaletteAction::SelectSlot(2));
    cx.apply(PaletteAction::RemoveSwatch(0));
    assert_eq!(cx.view().palette.order, [2, 1, 3]);
    assert_eq!(cx.state.slot, 1);
    assert_eq!(cx.view().palette.slots, palette.slots[..3]);
    cx.apply(PaletteAction::AddSwatch {
        rgb: [1, 2, 3],
        at: Some(1),
    });
    assert_eq!(cx.view().palette.order, [2, 4, 1, 3]);
    assert_eq!(cx.state.slot, 1);
    assert_eq!(cx.view().palette.get(4).unwrap(), &Swatch::rgb(1, 2, 3));
    cx.apply(PaletteAction::RemoveSwatch(1));
    assert_eq!(cx.view().palette.order[cx.state.slot], 1);
    cx.apply(PaletteAction::RemoveSwatch(2));
    cx.apply(PaletteAction::RemoveSwatch(1));
    let before = cx.state.clone();
    cx.rejects(PaletteAction::RemoveSwatch(0), "last swatch");
    cx.rejects(PaletteAction::RemoveSwatch(usize::MAX), "no longer exists");
    cx.rejects(
        PaletteAction::MoveSwatch {
            from: usize::MAX,
            to: 0,
        },
        "no longer exists",
    );
    cx.rejects(
        PaletteAction::DuplicateSwatch(usize::MAX),
        "no longer exists",
    );
    assert_eq!(cx.state, before);
}

#[test]
fn duplicate_swatch_copies_name_and_color_and_capacity_failures_are_atomic() {
    let mut cx = Context::custom(Palette::default());
    cx.apply(PaletteAction::DuplicateSwatch(0));
    assert_eq!(cx.view().palette.order, [1, 2]);
    assert_eq!(cx.view().palette.get(1), cx.view().palette.get(2));
    assert_eq!(cx.state.slot, 1);
    for i in 2..256 {
        cx.apply(PaletteAction::AddSwatch {
            rgb: [i as u8, 0, 0],
            at: Some(usize::MAX),
        });
    }
    assert_eq!(cx.view().palette.order, (1..=256).collect::<Vec<_>>());
    assert_eq!(cx.state.slot, 255);
    let before = cx.state.clone();
    cx.rejects(
        PaletteAction::AddSwatch {
            rgb: [0; 3],
            at: None,
        },
        "256 swatches",
    );
    cx.rejects(PaletteAction::DuplicateSwatch(0), "256 swatches");
    assert_eq!(cx.state, before);
    cx.apply(PaletteAction::RemoveSwatch(42));
    cx.apply(PaletteAction::AddSwatch {
        rgb: [10; 3],
        at: Some(0),
    });
    assert_eq!(cx.view().palette.order[0], 43);
    assert_eq!(cx.view().palette.slots.len(), 256);
}

#[test]
fn preview_copies_only_once_commits_once_and_cancel_restores_library_and_selection() {
    for custom in [false, true] {
        let mut cx = if custom {
            Context::custom(Palette::default())
        } else {
            Context::default()
        };
        let before = cx.state.clone();
        let brush = cx.brush.clone();
        let first = cx.apply(PaletteAction::SetColor {
            rgb: [1, 2, 3],
            commit: false,
        });
        assert_eq!(first.copied_from.is_some(), !custom);
        assert!(first.library_bytes.is_none());
        let id = cx.state.custom;
        for value in 4..64 {
            let out = cx.apply(PaletteAction::SetColor {
                rgb: [value; 3],
                commit: false,
            });
            assert!(out.library_bytes.is_none() && out.copied_from.is_none());
            assert_eq!(cx.state.custom, id);
            assert_eq!(cx.state.library.entries.len(), 1);
            assert_eq!(
                cx.view().palette.get(1).unwrap().rgba,
                [value, value, value, 255]
            );
        }
        assert_eq!(cx.brush, brush);
        assert_eq!(cx.palette, Palette::default());
        let cancelled = cx.apply(PaletteAction::CancelColor);
        assert!(cancelled.library_bytes.is_none());
        assert_eq!(cx.state, before);
        cx.apply(PaletteAction::SetColor {
            rgb: [9, 8, 7],
            commit: false,
        });
        let committed = cx.apply(PaletteAction::CommitColor);
        assert!(cx.state.preview.is_none());
        assert!(committed.library_bytes.is_some());
        assert_eq!(
            library::load(committed.library_bytes.as_deref()).unwrap(),
            cx.state.library
        );
        assert!(cx.apply(PaletteAction::CommitColor).library_bytes.is_none());
        cx.apply(PaletteAction::SetColor {
            rgb: [1; 3],
            commit: false,
        });
        let committed = cx.apply(PaletteAction::SetColor {
            rgb: [2; 3],
            commit: true,
        });
        assert!(committed.library_bytes.is_some() && cx.state.preview.is_none());
        assert_eq!(cx.view().palette.get(1).unwrap().rgba, [2, 2, 2, 255]);
        assert_eq!(cx.view().palette.get(1).unwrap().name, Some("clay".into()));
    }
}

#[test]
fn selection_cancels_preview_and_loading_never_requests_a_write() {
    let mut cx = Context::default();
    let before = cx.state.clone();
    cx.apply(PaletteAction::SetColor {
        rgb: [0; 3],
        commit: false,
    });
    let out = cx.apply(PaletteAction::SelectSlot(0));
    assert!(out.library_bytes.is_none());
    assert_eq!(cx.state, before);
    let out = cx.apply(PaletteAction::SetColor {
        rgb: [0; 3],
        commit: true,
    });
    let bytes = out.library_bytes.unwrap();
    let mut fresh = Context::default();
    assert!(
        fresh
            .apply(PaletteAction::LoadLibrary(Some(bytes)))
            .library_bytes
            .is_none()
    );
    assert_eq!(fresh.state.library, cx.state.library);
}

#[test]
fn source_noops_do_not_copy_but_remove_and_reorder_do_copy() {
    let mut cx = Context {
        palette: Preset::Pico8.palette(),
        ..Default::default()
    };
    let out = cx.apply(PaletteAction::MoveSwatch { from: 1, to: 1 });
    assert!(out.copied_from.is_none() && out.library_bytes.is_none());
    let out = cx.apply(PaletteAction::MoveSwatch {
        from: 0,
        to: usize::MAX,
    });
    assert_eq!(out.copied_from.as_deref(), Some("Project colors"));
    assert_eq!(cx.view().palette.order.last(), Some(&1));
    cx.apply(PaletteAction::ChoosePreset(0));
    let out = cx.apply(PaletteAction::RemoveSwatch(1));
    assert!(out.copied_from.is_some());
    assert_eq!(cx.state.library.entries[1].name, "Project colors copy 2");
    assert!(cx.view().palette.get(2).is_none());
    assert_eq!(cx.palette, Preset::Pico8.palette());
}

#[test]
fn stepping_accepts_extreme_deltas_without_overflow() {
    let mut cx = Context {
        palette: Preset::Pico8.palette(),
        ..Default::default()
    };
    cx.brush = cx.palette.get(16).unwrap().clone();
    cx.apply(PaletteAction::StepSwatch(isize::MAX));
    cx.apply(PaletteAction::StepSwatch(isize::MIN));
}

#[test]
fn recent_colors_are_committed_mru_distinct_bounded_and_persisted() {
    let mut cx = Context::custom(Palette::default());
    for value in 0..20 {
        cx.apply(PaletteAction::SetColor {
            rgb: [value; 3],
            commit: true,
        });
    }
    assert_eq!(
        cx.state.recent_colors,
        (4..20).rev().map(|v| [v; 3]).collect::<Vec<_>>()
    );
    cx.apply(PaletteAction::SetColor {
        rgb: [10; 3],
        commit: false,
    });
    assert_eq!(cx.state.recent_colors[0], [19; 3]);
    cx.apply(PaletteAction::CommitColor);
    assert_eq!(cx.state.recent_colors[0], [10; 3]);
    assert_eq!(cx.state.recent_colors[1], [19; 3]);
    assert_eq!(cx.state.recent_colors.len(), 16);
    let out = cx.apply(PaletteAction::SetColor {
        rgb: [10; 3],
        commit: true,
    });
    assert!(out.library_bytes.is_none());
    let before = cx.state.recent_colors.clone();
    cx.apply(PaletteAction::SelectSlot(0));
    assert_eq!(cx.state.recent_colors, before);
    let brush = cx.brush.clone();
    cx.apply(PaletteAction::UseRecentColor {
        index: 1,
        target: RecentColorTarget::NewSwatch { at: Some(0) },
    });
    assert_eq!(cx.state.recent_colors[0], [19; 3]);
    assert_eq!(
        cx.view()
            .palette
            .get(cx.view().palette.order[0])
            .unwrap()
            .rgba,
        [19, 19, 19, 255]
    );
    cx.apply(PaletteAction::UseRecentColor {
        index: 1,
        target: RecentColorTarget::SelectedSwatch,
    });
    assert_eq!(cx.state.recent_colors[0], [10; 3]);
    assert_eq!(cx.brush, brush);
    cx.rejects(
        PaletteAction::UseRecentColor {
            index: 16,
            target: RecentColorTarget::SelectedSwatch,
        },
        "no longer exists",
    );
    let bytes = library::encode_state(&cx.state).unwrap();
    let mut fresh = Context::default();
    let out = fresh.apply(PaletteAction::LoadLibrary(Some(bytes)));
    assert!(out.library_bytes.is_none());
    assert_eq!(fresh.state.recent_colors, cx.state.recent_colors);
    assert_eq!(fresh.state.library, cx.state.library);
}

#[test]
fn brush_choices_update_recent_colors_without_touching_manager_selection_or_document() {
    let mut cx = Context {
        palette: Preset::Pico8.palette(),
        ..Default::default()
    };
    cx.apply(PaletteAction::SelectSlot(3));
    let out = cx.apply(PaletteAction::SelectSwatch(12));
    assert!(out.library_bytes.is_some());
    assert_eq!(cx.state.slot, 3);
    assert_eq!(cx.state.recent_colors, [[41, 173, 255]]);
    assert!(
        cx.apply(PaletteAction::SelectSwatch(12))
            .library_bytes
            .is_none()
    );
    let out = cx.apply(PaletteAction::StepSwatch(1));
    assert!(out.library_bytes.is_some());
    assert_eq!(cx.state.recent_colors, [[131, 118, 156], [41, 173, 255]]);
    assert_eq!(cx.palette, Preset::Pico8.palette());
    assert!(cx.state.library.entries.is_empty());
}

#[test]
fn old_library_fixtures_and_optional_recent_colors_load_with_strict_bounds() {
    let old = br#"{"entries":[{"id":"00000000-0000-0000-0000-000000000001","name":"Old","palette":{"slots":[{"index":255,"rgba":[1,2,3,255],"name":"Named"},{"index":8,"rgba":[4,5,6,255]}],"order":[8,255]}}]}"#;
    let state = library::load_state(Some(old)).unwrap();
    assert!(state.recent_colors.is_empty());
    assert_eq!(state.library.entries[0].palette.order, [8, 255]);
    let encoded = library::encode_state(&state).unwrap();
    assert!(
        !String::from_utf8(encoded.clone())
            .unwrap()
            .contains("recent_colors")
    );
    assert_eq!(library::load_state(Some(&encoded)).unwrap(), state);
    for text in [
        r#"{"entries":[],"recent_colors":[[1,2,3],[1,2,3]]}"#.to_owned(),
        r#"{"entries":[],"recent_colors":[[256,0,0]]}"#.to_owned(),
        r#"{"entries":[],"recent_colors":[[1,2]]}"#.to_owned(),
        r#"{"entries":[],"recent_colors":null}"#.to_owned(),
        format!(
            r#"{{"entries":[],"recent_colors":{}}}"#,
            json::to_string(&(0..17).map(|i| [i, 0, 0]).collect::<Vec<_>>()).unwrap()
        ),
    ] {
        assert!(
            library::load_state(Some(text.as_bytes())).is_err(),
            "{text}"
        );
    }
}

#[test]
fn manager_exports_formats_and_registry_pal_encoder_use_the_same_bytes() {
    use voxelite_plugin_host::{Host, SnapshotExport};
    let mut cx = Context {
        palette: Preset::Pico8.palette(),
        ..Default::default()
    };
    assert_eq!(cx.view().formats, FORMATS);
    let before = cx.state.clone();
    for format in FORMATS {
        let out = cx.apply(PaletteAction::ExportLibrary {
            format: format.id.into(),
        });
        assert!(out.library_bytes.is_none() && out.commands.is_empty() && out.color.is_none());
        let exported = out.export.unwrap();
        assert_eq!(exported.format, *format);
        assert_eq!(import(format.id, &exported.text).unwrap(), cx.palette);
        assert_eq!(cx.state, before);
    }
    let mut host = Host::new(vec![Box::new(PalettePlugin)]);
    let snapshot: Snapshot = json::from_slice(include_bytes!(
        "../../../voxelite-export/tests/golden/scene.json"
    ))
    .unwrap();
    let export_cx = SnapshotExport {
        snapshot: &snapshot,
        render_view: &|_| {
            Err(voxelite_plugin_api::ExportError::Invalid(
                "palette export does not render".into(),
            ))
        },
    };
    let bytes = host.dispatch_export(PAL, &export_cx).unwrap().unwrap();
    assert_eq!(bytes, to_pal(&snapshot.palette).unwrap().as_bytes());
    host.disable("dev.voxelite.palette").unwrap();
    assert!(host.dispatch_export(PAL, &export_cx).unwrap().is_none());
    assert_eq!(host.registry().formats().count(), 0);
}

#[test]
fn unchanged_source_color_and_rename_still_create_custom_copies() {
    let mut cx = Context::default();
    let out = cx.apply(PaletteAction::SetColor {
        rgb: [180, 140, 100],
        commit: true,
    });
    assert_eq!(out.copied_from.as_deref(), Some("Project colors"));
    assert_eq!(cx.view().palette, cx.palette);
    cx.apply(PaletteAction::ChoosePreset(0));
    let out = cx.apply(PaletteAction::Edit {
        operation: PaletteOperation::Rename,
        name: "Project colors".into(),
        color: String::new(),
    });
    assert!(out.copied_from.is_some());
    assert_eq!(cx.view().name, "Project colors");
    assert_eq!(cx.state.library.entries.len(), 2);
}

#[test]
fn library_serialization_never_emits_bytes_that_the_bounded_reader_rejects() {
    let color = Swatch {
        rgba: [1, 2, 3, 255],
        name: Some("n".repeat(128)),
    };
    let palette = Palette::from_colors(vec![color; 256]).unwrap();
    let mut state = PaletteState::default();
    for _ in 0..64 {
        library::create(&mut state.library, "Large".into(), palette.clone()).unwrap();
    }
    for recent in [vec![], vec![[1, 2, 3]]] {
        state.recent_colors = recent;
        let bytes = library::encode_state(&state).unwrap();
        assert!(bytes.len() <= 4 * 1024 * 1024);
        assert!(!bytes.contains(&b'\n'));
        assert_eq!(library::load_state(Some(&bytes)).unwrap(), state);
    }
    for _ in 0..64 {
        library::create(&mut state.library, "Large".into(), palette.clone()).unwrap();
    }
    assert!(
        library::encode_state(&state)
            .unwrap_err()
            .to_string()
            .contains("too large")
    );
}

#[test]
fn unicode_import_names_are_truncated_on_byte_boundaries() {
    let mut cx = Context::default();
    cx.apply(PaletteAction::ImportLibrary {
        name: "色".repeat(100),
        format: "hex".into(),
        text: "112233".into(),
    });
    assert_eq!(cx.view().name, "色".repeat(42));
    validate_name(&cx.view().name).unwrap();
}

#[test]
fn full_library_rejects_automatic_copies_atomically_and_duplicate_preserves_exact_slots() {
    let mut cx = Context::default();
    for _ in 0..128 {
        library::create(&mut cx.state.library, "Entry".into(), Palette::default()).unwrap();
    }
    let before = cx.state.clone();
    for commit in [false, true] {
        cx.rejects(
            PaletteAction::SetColor {
                rgb: [1; 3],
                commit,
            },
            "128 custom palettes",
        );
        assert_eq!(cx.state, before);
    }
    cx.rejects(PaletteAction::RemoveSwatch(0), "last swatch");
    assert_eq!(cx.state, before);
    let mut palette = Palette::from_colors(vec![Swatch::default(); 3]).unwrap();
    palette.slots[1].index = 42;
    palette.order = vec![3, 42, 1];
    let mut cx = Context::custom(palette.clone());
    let original = cx.state.custom;
    cx.apply(PaletteAction::Edit {
        operation: PaletteOperation::Duplicate,
        name: "Test".into(),
        color: String::new(),
    });
    assert_ne!(cx.state.custom, original);
    assert_eq!(cx.view().palette, palette);
    assert_eq!(cx.view().name, "Test copy");
}
