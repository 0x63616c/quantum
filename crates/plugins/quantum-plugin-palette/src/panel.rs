use crate::{PRESET_SETTING, PalettePlugin, Preset, import, library, manager};
use quantum_plugin_api::{
    Command, Palette, PaletteAction, PaletteChoice, PaletteCx, PaletteEdit, PaletteError,
    PaletteExport, PaletteId, PaletteKind, PaletteOutcome, PalettePanel, PalettePresentation,
    PalettePreview, PaletteState, PaletteView, RecentColorTarget, SettingValue, Swatch,
};

const CHOICES: [(&str, &str, Option<Preset>); 8] = [
    ("project", "Project colors", None),
    ("pico8", "PICO-8", Some(Preset::Pico8)),
    ("endesga32", "ENDESGA 32", Some(Preset::Endesga32)),
    ("sweetie16", "Sweetie 16", Some(Preset::Sweetie16)),
    ("grayscale", "Grayscale", Some(Preset::Grayscale)),
    ("neutral", "Neutral", Some(Preset::Neutral)),
    ("warm", "Warm", Some(Preset::Warm)),
    ("vivid", "Vivid", Some(Preset::Vivid)),
];

pub(crate) fn choices() -> Vec<String> {
    CHOICES.iter().map(|(key, _, _)| (*key).into()).collect()
}

impl PalettePanel for PalettePlugin {
    fn view(&self, cx: &dyn PaletteCx) -> PaletteView {
        view(cx, cx.state())
    }

    fn interact(
        &self,
        cx: &dyn PaletteCx,
        action: PaletteAction,
    ) -> Result<PaletteOutcome, PaletteError> {
        let mut out = PaletteOutcome {
            state: cx.state().clone(),
            ..Default::default()
        };
        let keep_preview = matches!(
            action,
            PaletteAction::SetColor { .. } | PaletteAction::CommitColor
        );
        if !keep_preview {
            cancel_preview(&mut out.state);
        }
        let view = view(cx, &out.state);
        let before = out
            .state
            .preview
            .as_ref()
            .map(|p| &p.library)
            .unwrap_or(&out.state.library)
            .clone();
        let before_recent = out.state.recent_colors.clone();
        let mut recent = None;
        let mut persist = true;
        match action {
            PaletteAction::LoadLibrary(bytes) => {
                let loaded = library::load_state(bytes.as_deref())?;
                out.state.library = loaded.library;
                out.state.recent_colors = loaded.recent_colors;
                if !out
                    .state
                    .library
                    .entries
                    .iter()
                    .any(|e| Some(e.id) == out.state.custom)
                {
                    out.state.custom = None;
                }
                persist = false;
            }
            PaletteAction::ChoosePreset(index)
            | PaletteAction::ChoosePalette(PaletteId::Source(index)) => {
                let choice = CHOICES
                    .get(index)
                    .ok_or_else(|| PaletteError::Message("Palette no longer exists".into()))?;
                out.setting = Some((PRESET_SETTING.into(), SettingValue::Choice(choice.0.into())));
                out.state.custom = None;
            }
            PaletteAction::ChooseCustom(id)
            | PaletteAction::ChoosePalette(PaletteId::Custom(id)) => {
                out.state
                    .library
                    .entries
                    .iter()
                    .find(|e| e.id == id)
                    .ok_or_else(|| PaletteError::Message("Palette no longer exists".into()))?;
                out.state.custom = Some(id);
            }
            PaletteAction::SelectSlot(slot) => {
                out.state.slot = slot.min(view.palette.order.len().saturating_sub(1))
            }
            PaletteAction::SelectSwatch(index) => {
                let slot =
                    view.palette.order.get(index).ok_or_else(|| {
                        PaletteError::Message("Palette slot no longer exists".into())
                    })?;
                out.color = view.palette.get(*slot).cloned();
                out.selected = Some(index);
            }
            PaletteAction::StepSwatch(delta) => {
                view.palette.validate()?;
                let (index, color) = library::step_swatch(&view.palette, cx.active_color(), delta);
                out.color = Some(color);
                out.selected = Some(index);
            }
            PaletteAction::Edit {
                operation,
                name,
                color,
            } => {
                let edit = manager::legacy(&view, operation, name, color)?;
                recent = manager::edit(&view, &mut out, edit)?;
            }
            PaletteAction::SetColor { rgb, commit } => {
                if !commit && out.state.preview.is_none() {
                    out.state.preview = Some(PalettePreview {
                        library: out.state.library.clone(),
                        custom: out.state.custom,
                        slot: out.state.slot,
                    });
                }
                recent = manager::edit(&view, &mut out, manager::Edit::SetColor(rgb))?;
                persist = commit;
                if commit {
                    out.state.preview = None;
                }
            }
            PaletteAction::CommitColor => {
                if out.state.preview.take().is_some() {
                    recent = view
                        .palette
                        .order
                        .get(view.state.slot)
                        .and_then(|slot| view.palette.get(*slot))
                        .map(|s| {
                            let [r, g, b, _] = s.rgba;
                            [r, g, b]
                        });
                }
            }
            PaletteAction::CancelColor => {}
            PaletteAction::AddSwatch { rgb: [r, g, b], at } => {
                recent = manager::edit(
                    &view,
                    &mut out,
                    manager::Edit::Add {
                        color: Swatch::rgb(r, g, b),
                        at,
                    },
                )?;
            }
            PaletteAction::RemoveSwatch(position) => {
                manager::edit(&view, &mut out, manager::Edit::Remove(position))?;
            }
            PaletteAction::MoveSwatch { from, to } => {
                manager::edit(&view, &mut out, manager::Edit::Move { from, to })?;
            }
            PaletteAction::DuplicateSwatch(position) => {
                recent = manager::edit(&view, &mut out, manager::Edit::DuplicateSwatch(position))?;
            }
            PaletteAction::UseRecentColor { index, target } => {
                let [r, g, b] =
                    *out.state.recent_colors.get(index).ok_or_else(|| {
                        PaletteError::Message("Recent color no longer exists".into())
                    })?;
                let edit = match target {
                    RecentColorTarget::SelectedSwatch => manager::Edit::SetColor([r, g, b]),
                    RecentColorTarget::NewSwatch { at } => manager::Edit::Add {
                        color: Swatch::rgb(r, g, b),
                        at,
                    },
                };
                recent = manager::edit(&view, &mut out, edit)?;
            }
            PaletteAction::ImportLibrary { name, format, text } => {
                let palette = import(&format, &text)?;
                out.state.custom = Some(library::create(
                    &mut out.state.library,
                    library::import_name(&name),
                    palette.clone(),
                )?);
                out.state.slot = out.state.slot.min(palette.order.len().saturating_sub(1));
            }
            PaletteAction::ExportLibrary { format } => {
                out.export = Some(PaletteExport {
                    format: crate::format(&format)?,
                    text: crate::export(&format, &view.palette, &view.name)?,
                });
            }
            PaletteAction::Command(id) => {
                out.presentation = Some(match id.as_str() {
                    "dev.quantum.palette.show_palette" => PalettePresentation::Focus,
                    "dev.quantum.palette.palette" => PalettePresentation::Choose,
                    "dev.quantum.palette.manage_palettes" => PalettePresentation::Manage,
                    _ => {
                        return Err(PaletteError::Message(
                            "Palette command is unavailable".into(),
                        ));
                    }
                })
            }
            PaletteAction::EditDocument(edit) => out.commands.push(Command::Palette(edit)),
            PaletteAction::SetSlot { slot, color } => {
                out.commands
                    .push(Command::Palette(if cx.palette().get(slot).is_some() {
                        PaletteEdit::Set { slot, color }
                    } else {
                        PaletteEdit::Add {
                            slot: Some(slot),
                            color,
                        }
                    }))
            }
            PaletteAction::AppendPreset(name) => append(cx.palette(), preset(&name)?, &mut out)?,
            PaletteAction::AppendImport { format, text } => {
                append(cx.palette(), import(&format, &text)?, &mut out)?
            }
        }
        if persist {
            if let Some(rgb) = recent.or_else(|| {
                out.color.as_ref().map(|s| {
                    let [r, g, b, _] = s.rgba;
                    [r, g, b]
                })
            }) {
                library::remember(&mut out.state, rgb);
            }
            if out.state.library != before || out.state.recent_colors != before_recent {
                out.library_bytes = Some(library::encode_state(&out.state)?);
            }
        }
        Ok(out)
    }
}

fn cancel_preview(state: &mut PaletteState) {
    if let Some(preview) = state.preview.take() {
        state.library = preview.library;
        state.custom = preview.custom;
        state.slot = preview.slot;
    }
}

fn view(cx: &dyn PaletteCx, state: &PaletteState) -> PaletteView {
    let preset = CHOICES
        .iter()
        .position(|(key, _, _)| {
            cx.setting(PRESET_SETTING) == Some(&SettingValue::Choice((*key).into()))
        })
        .unwrap_or(0);
    let mut choices: Vec<_> = CHOICES
        .iter()
        .enumerate()
        .map(|(index, (_, name, preset))| PaletteChoice {
            id: PaletteId::Source(index),
            kind: if index == 0 {
                PaletteKind::Project
            } else {
                PaletteKind::BuiltIn
            },
            read_only: true,
            name: (*name).into(),
            palette: preset
                .map(Preset::palette)
                .unwrap_or_else(|| cx.palette().clone()),
        })
        .collect();
    choices.extend(state.library.entries.iter().map(|entry| PaletteChoice {
        id: PaletteId::Custom(entry.id),
        kind: PaletteKind::Custom,
        read_only: false,
        name: entry.name.clone(),
        palette: entry.palette.clone(),
    }));
    let selected = state
        .custom
        .and_then(|id| state.library.entries.iter().find(|e| e.id == id));
    let (name, palette) = selected
        .map(|e| (e.name.clone(), e.palette.clone()))
        .unwrap_or_else(|| {
            (
                choices[preset].name.clone(),
                choices[preset].palette.clone(),
            )
        });
    let mut state = state.clone();
    state.slot = state.slot.min(palette.order.len().saturating_sub(1));
    PaletteView {
        formats: crate::FORMATS.to_vec(),
        selected: selected
            .map(|e| PaletteId::Custom(e.id))
            .unwrap_or(PaletteId::Source(preset)),
        kind: if selected.is_some() {
            PaletteKind::Custom
        } else {
            choices[preset].kind
        },
        read_only: selected.is_none(),
        name,
        palette,
        choices,
        preset,
        state,
    }
}

fn preset(name: &str) -> Result<Palette, PaletteError> {
    CHOICES
        .iter()
        .find(|(key, _, preset)| *key == name && preset.is_some())
        .and_then(|(_, _, preset)| *preset)
        .map(Preset::palette)
        .ok_or_else(|| PaletteError::Message("unknown preset".into()))
}

#[expect(clippy::expect_used, reason = "internal invariant: validated palette")]
fn append(
    current: &Palette,
    palette: Palette,
    out: &mut PaletteOutcome,
) -> Result<(), PaletteError> {
    let free: Vec<_> = (1..=256).filter(|i| current.get(*i).is_none()).collect();
    if free.len() < palette.slots.len() {
        return Err(PaletteError::Capacity);
    }
    let mut commands = Vec::new();
    for (source, target) in palette.order.iter().zip(free) {
        commands.push(Command::Palette(PaletteEdit::Add {
            slot: Some(target),
            color: palette.get(*source).expect("validated palette").clone(),
        }));
        out.slot_mapping.push((*source, target));
    }
    out.commands = commands;
    Ok(())
}
