use crate::{PRESET_SETTING, PalettePlugin, Preset, library, manager};
use voxelite_plugin_api::{
    Command, Palette, PaletteAction, PaletteChoice, PaletteCx, PaletteEdit, PaletteError,
    PaletteId, PaletteKind, PaletteOutcome, PalettePanel, PalettePresentation, PalettePreview,
    PaletteState, PaletteView, SettingValue, Swatch,
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
        let mut persist = true;
        match action {
            PaletteAction::LoadLibrary(bytes) => {
                out.state.library = library::load(bytes.as_deref())?;
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
                manager::edit(&view, &mut out, edit)?;
            }
            PaletteAction::SetColor { rgb, commit } => {
                if !commit && out.state.preview.is_none() {
                    out.state.preview = Some(PalettePreview {
                        library: out.state.library.clone(),
                        custom: out.state.custom,
                        slot: out.state.slot,
                    });
                }
                manager::edit(&view, &mut out, manager::Edit::SetColor(rgb))?;
                persist = commit;
                if commit {
                    out.state.preview = None;
                }
            }
            PaletteAction::CommitColor => {
                out.state.preview = None;
            }
            PaletteAction::CancelColor => {}
            PaletteAction::AddSwatch { rgb: [r, g, b], at } => {
                manager::edit(
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
                manager::edit(&view, &mut out, manager::Edit::DuplicateSwatch(position))?;
            }
            PaletteAction::ImportLibrary { name, format, text } => {
                let palette = import(&format, &text)?;
                out.state.custom = Some(library::create(
                    &mut out.state.library,
                    name.chars().take(100).collect(),
                    palette.clone(),
                )?);
                out.state.slot = out.state.slot.min(palette.order.len().saturating_sub(1));
            }
            PaletteAction::Command(id) => {
                out.presentation = Some(match id.as_str() {
                    "dev.voxelite.palette.show_palette" => PalettePresentation::Focus,
                    "dev.voxelite.palette.palette" => PalettePresentation::Choose,
                    "dev.voxelite.palette.manage_palettes" => PalettePresentation::Manage,
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
        if persist && out.state.library != before {
            out.library_bytes = Some(library::encode(&out.state.library)?);
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

/// Parse an existing bounded palette interchange format.
pub fn import(format: &str, text: &str) -> Result<Palette, PaletteError> {
    Ok(match format {
        "hex" => crate::from_hex(text)?,
        "gpl" => crate::from_gpl(text)?,
        _ => {
            return Err(PaletteError::Message(
                "Choose a .hex or .gpl palette".into(),
            ));
        }
    })
}

fn preset(name: &str) -> Result<Palette, PaletteError> {
    CHOICES
        .iter()
        .find(|(key, _, preset)| *key == name && preset.is_some())
        .and_then(|(_, _, preset)| *preset)
        .map(Preset::palette)
        .ok_or_else(|| PaletteError::Message("unknown preset".into()))
}

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
