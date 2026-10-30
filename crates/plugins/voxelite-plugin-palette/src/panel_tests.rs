use super::*;
use voxelite_plugin_api::*;
use voxelite_plugin_host::Host;

#[derive(Default)]
struct Context {
    palette: Palette,
    color: Swatch,
    state: PaletteState,
}
impl PaletteCx for Context {
    fn palette(&self) -> &Palette {
        &self.palette
    }
    fn active_color(&self) -> &Swatch {
        &self.color
    }
    fn state(&self) -> &PaletteState {
        &self.state
    }
    fn setting(&self, _: &str) -> Option<&SettingValue> {
        None
    }
}

fn request(host: &mut Host, cx: &mut Context, action: PaletteAction) -> PaletteOutcome {
    let outcome = host
        .dispatch_palette(PANEL_ID, cx, action)
        .unwrap()
        .unwrap();
    cx.state = outcome.state.clone();
    if let Some(color) = &outcome.color {
        cx.color = color.clone();
    }
    if let Some((key, value)) = &outcome.setting {
        host.set_setting(key, value.clone()).unwrap();
    }
    outcome
}

fn edit(operation: PaletteOperation, name: &str, color: &str) -> PaletteAction {
    PaletteAction::Edit {
        operation,
        name: name.into(),
        color: color.into(),
    }
}

#[test]
fn choosing_a_palette_preserves_the_hidden_manager_cursor_until_fields_sync() {
    let mut host = Host::new(vec![Box::new(PalettePlugin)]);
    let mut cx = Context::default();
    request(&mut host, &mut cx, PaletteAction::ChoosePreset(2));
    request(&mut host, &mut cx, PaletteAction::SelectSlot(31));
    request(&mut host, &mut cx, PaletteAction::ChoosePreset(6));
    assert_eq!(cx.state.slot, 31);
    assert_eq!(
        host.palette_view(PANEL_ID, &cx)
            .unwrap()
            .unwrap()
            .state
            .slot,
        7
    );
    request(&mut host, &mut cx, PaletteAction::StepSwatch(1));
    request(&mut host, &mut cx, PaletteAction::ChoosePreset(2));
    assert_eq!(
        host.palette_view(PANEL_ID, &cx)
            .unwrap()
            .unwrap()
            .state
            .slot,
        31
    );
}

#[test]
fn registered_panel_commands_and_settings_conform_across_disable_enable() {
    let mut host = Host::new(vec![Box::new(PalettePlugin)]);
    let mut cx = Context::default();
    assert_eq!(host.registry().panels().count(), 1);
    assert_eq!(host.registry().commands().count(), 3);
    assert_eq!(host.registry().settings().count(), 1);
    let view = host.palette_view(PANEL_ID, &cx).unwrap().unwrap();
    assert_eq!(
        view.choices
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        [
            "Project colors",
            "PICO-8",
            "ENDESGA 32",
            "Sweetie 16",
            "Grayscale",
            "Neutral",
            "Warm",
            "Vivid"
        ]
    );
    for (id, presentation) in [
        ("show_palette", PalettePresentation::Focus),
        ("palette", PalettePresentation::Choose),
        ("manage_palettes", PalettePresentation::Manage),
    ] {
        let out = request(
            &mut host,
            &mut cx,
            PaletteAction::Command(format!("dev.voxelite.palette.{id}")),
        );
        assert_eq!(out.presentation, Some(presentation));
        assert!(out.commands.is_empty() && out.library_bytes.is_none());
    }
    request(&mut host, &mut cx, PaletteAction::ChoosePreset(2));
    host.disable("dev.voxelite.palette").unwrap();
    assert_eq!(host.registry().panels().count(), 0);
    assert_eq!(host.registry().commands().count(), 0);
    assert_eq!(host.registry().settings().count(), 0);
    assert!(host.palette_view(PANEL_ID, &cx).unwrap().is_none());
    assert!(
        host.dispatch_palette(PANEL_ID, &cx, PaletteAction::StepSwatch(1))
            .unwrap()
            .is_none()
    );
    host.enable("dev.voxelite.palette").unwrap();
    let view = host.palette_view(PANEL_ID, &cx).unwrap().unwrap();
    assert_eq!(view.preset, 2);
    assert_eq!(view.palette, Preset::Endesga32.palette());
    assert!(
        host.dispatch_palette(
            PANEL_ID,
            &cx,
            PaletteAction::Command("dev.voxelite.core.save".into())
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn selection_and_library_operations_preserve_colors_and_order() {
    use PaletteOperation::*;
    let mut host = Host::new(vec![Box::new(PalettePlugin)]);
    let mut cx = Context::default();
    request(&mut host, &mut cx, PaletteAction::ChoosePreset(1));
    assert_eq!(cx.color, Swatch::default());
    let selected = request(&mut host, &mut cx, PaletteAction::SelectSwatch(12));
    assert_eq!(cx.color.rgba, [41, 173, 255, 255]);
    assert!(selected.commands.is_empty());
    assert_eq!(cx.palette, Palette::default());
    request(&mut host, &mut cx, edit(Duplicate, "PICO-8", ""));
    let id = cx.state.custom.unwrap();
    assert_eq!(cx.state.library.entries[0].name, "PICO-8 copy");
    request(&mut host, &mut cx, edit(Rename, "Studio", ""));
    request(&mut host, &mut cx, edit(AddSlot, "", "112233"));
    request(&mut host, &mut cx, PaletteAction::SelectSlot(16));
    request(&mut host, &mut cx, edit(MoveLeft, "", ""));
    assert_eq!(cx.state.slot, 15);
    assert_eq!(
        cx.state.library.entries[0].palette.get(17).unwrap().rgba,
        [17, 34, 51, 255]
    );
    request(&mut host, &mut cx, edit(ApplyColor, "", "445566"));
    request(&mut host, &mut cx, edit(MoveRight, "", ""));
    assert_eq!(cx.state.slot, 16);
    let removed = request(&mut host, &mut cx, edit(RemoveSlot, "", ""));
    assert_eq!(cx.state.slot, 15);
    assert_eq!(cx.state.library.entries[0].palette, Preset::Pico8.palette());
    let bytes = removed.library_bytes.unwrap();
    let mut restarted = Context::default();
    request(
        &mut host,
        &mut restarted,
        PaletteAction::LoadLibrary(Some(bytes)),
    );
    assert_eq!(restarted.state.library, cx.state.library);
    request(&mut host, &mut restarted, PaletteAction::ChooseCustom(id));
    assert_eq!(
        host.palette_view(PANEL_ID, &restarted)
            .unwrap()
            .unwrap()
            .name,
        "Studio"
    );
    request(&mut host, &mut cx, edit(Delete, "", ""));
    assert!(cx.state.library.entries.is_empty() && cx.state.custom.is_none());
    assert_eq!(
        host.palette_view(PANEL_ID, &cx).unwrap().unwrap().palette,
        Preset::Pico8.palette()
    );
}

#[test]
fn bounded_imports_duplicates_and_invalid_library_edits_are_atomic() {
    let mut host = Host::new(vec![Box::new(PalettePlugin)]);
    let mut cx = Context::default();
    request(
        &mut host,
        &mut cx,
        PaletteAction::ImportLibrary {
            name: "Imported".into(),
            format: "hex".into(),
            text: "#112233\r\n112233\n".into(),
        },
    );
    assert_eq!(
        cx.state.library.entries[0].palette.to_hex(),
        "112233\n112233\n"
    );
    let before = cx.state.clone();
    for action in [
        PaletteAction::ChoosePreset(8),
        PaletteAction::ChooseCustom(Uuid::nil()),
        PaletteAction::SelectSwatch(256),
        edit(PaletteOperation::Rename, "", ""),
        edit(PaletteOperation::ApplyColor, "", "112233\n445566"),
        PaletteAction::LoadLibrary(Some(b"not json".to_vec())),
        PaletteAction::LoadLibrary(Some(vec![b' '; 4 * 1024 * 1024 + 1])),
        PaletteAction::AppendImport {
            format: "hex".into(),
            text: "123456\n".repeat(10000),
        },
    ] {
        assert!(host.dispatch_palette(PANEL_ID, &cx, action).is_err());
        assert_eq!(cx.state, before);
    }
    let mut duplicate = before.library.clone();
    duplicate.entries.push(duplicate.entries[0].clone());
    assert!(library::validate(&duplicate).is_err());
    assert!(parse_color("#xyz000").is_err());
    assert_eq!(parse_color("#A0b1C2").unwrap().rgba, [160, 177, 194, 255]);
}

#[test]
fn document_import_proposes_typed_commands_in_free_slot_order() {
    let mut host = Host::new(vec![Box::new(PalettePlugin)]);
    let cx = Context::default();
    let out = host
        .dispatch_palette(
            PANEL_ID,
            &cx,
            PaletteAction::AppendImport {
                format: "hex".into(),
                text: "112233\n112233".into(),
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(out.slot_mapping, [(1, 2), (2, 3)]);
    assert_eq!(
        out.commands,
        vec![
            Command::Palette(PaletteEdit::Add {
                slot: Some(2),
                color: Swatch::rgb(17, 34, 51)
            }),
            Command::Palette(PaletteEdit::Add {
                slot: Some(3),
                color: Swatch::rgb(17, 34, 51)
            }),
        ]
    );
    assert_eq!(out.state, cx.state);
    assert!(out.library_bytes.is_none() && out.color.is_none());
    let full = Context {
        palette: Palette::from_colors(vec![Swatch::default(); 256]).unwrap(),
        ..Default::default()
    };
    assert!(
        host.dispatch_palette(PANEL_ID, &full, PaletteAction::AppendPreset("pico8".into()))
            .is_err()
    );
}

#[test]
fn choices_expose_origins_and_stable_selection_addresses() {
    let mut host = Host::new(vec![Box::new(PalettePlugin)]);
    let mut cx = Context::default();
    let view = host.palette_view(PANEL_ID, &cx).unwrap().unwrap();
    assert_eq!(view.kind, PaletteKind::Project);
    assert!(view.read_only);
    for (i, choice) in view.choices.iter().enumerate() {
        assert_eq!(choice.id, PaletteId::Source(i));
        assert!(choice.read_only);
        assert_eq!(
            choice.kind,
            if i == 0 {
                PaletteKind::Project
            } else {
                PaletteKind::BuiltIn
            }
        );
    }
    request(&mut host, &mut cx, edit(PaletteOperation::Create, "", ""));
    let id = cx.state.custom.unwrap();
    let view = host.palette_view(PANEL_ID, &cx).unwrap().unwrap();
    assert_eq!(view.selected, PaletteId::Custom(id));
    assert_eq!(view.kind, PaletteKind::Custom);
    assert!(!view.read_only);
    assert_eq!(view.choices.last().unwrap().id, PaletteId::Custom(id));
    assert!(!view.choices.last().unwrap().read_only);
    request(
        &mut host,
        &mut cx,
        PaletteAction::ChoosePalette(PaletteId::Source(2)),
    );
    assert_eq!(
        host.palette_view(PANEL_ID, &cx).unwrap().unwrap().kind,
        PaletteKind::BuiltIn
    );
    request(
        &mut host,
        &mut cx,
        PaletteAction::ChoosePalette(PaletteId::Custom(id)),
    );
    assert_eq!(cx.state.custom, Some(id));
}

#[test]
fn source_edits_copy_once_and_leave_document_and_builtin_unchanged() {
    for source in [0, 1] {
        for operation in [
            PaletteOperation::ApplyColor,
            PaletteOperation::AddSlot,
            PaletteOperation::Rename,
        ] {
            let mut host = Host::new(vec![Box::new(PalettePlugin)]);
            let mut cx = Context::default();
            request(&mut host, &mut cx, PaletteAction::ChoosePreset(source));
            let view = host.palette_view(PANEL_ID, &cx).unwrap().unwrap();
            let out = request(&mut host, &mut cx, edit(operation, "Renamed", "123456"));
            assert_eq!(out.copied_from, Some(view.name.clone()));
            assert!(out.library_bytes.is_some());
            assert_eq!(cx.state.library.entries.len(), 1);
            let id = cx.state.custom;
            let out = request(
                &mut host,
                &mut cx,
                edit(PaletteOperation::ApplyColor, "", "aabbcc"),
            );
            assert!(out.copied_from.is_none());
            assert_eq!(cx.state.custom, id);
            assert_eq!(cx.palette, Palette::default());
            request(&mut host, &mut cx, PaletteAction::ChoosePreset(source));
            assert_eq!(
                host.palette_view(PANEL_ID, &cx).unwrap().unwrap().palette,
                view.palette
            );
            assert!(
                host.dispatch_palette(PANEL_ID, &cx, edit(PaletteOperation::Delete, "", ""))
                    .unwrap_err()
                    .to_string()
                    .contains("Only custom")
            );
        }
    }
}

#[test]
fn copy_names_are_unique_and_reserve_utf8_bytes_for_numbered_suffixes() {
    for source in ["PICO-8".to_owned(), "x".repeat(128), "色".repeat(42)] {
        let mut library = PaletteLibrary::default();
        for number in 1..=128 {
            let name = library::copy_name(&library, &source).unwrap();
            validate_name(&name).unwrap();
            assert!(name.ends_with(&if number == 1 {
                " copy".into()
            } else {
                format!(" copy {number}")
            }));
            library::create(&mut library, name, Palette::default()).unwrap();
        }
        assert!(library::create(&mut library, "Overflow".into(), Palette::default()).is_err());
    }
    assert!(library::copy_name(&PaletteLibrary::default(), "bad\nname").is_err());
}
