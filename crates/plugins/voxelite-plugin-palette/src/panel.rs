use crate::{PRESET_SETTING, PalettePlugin, Preset, library};
use voxelite_plugin_api::{
    Command, Palette, PaletteAction, PaletteChoice, PaletteCx, PaletteEdit, PaletteError,
    PaletteOperation, PaletteOutcome, PalettePanel, PalettePresentation, PaletteView, SettingValue,
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
        let preset = CHOICES
            .iter()
            .position(|(key, _, _)| {
                cx.setting(PRESET_SETTING) == Some(&SettingValue::Choice((*key).into()))
            })
            .unwrap_or(0);
        let choices: Vec<_> = CHOICES
            .iter()
            .map(|(_, name, preset)| PaletteChoice {
                name: (*name).into(),
                palette: preset
                    .map(Preset::palette)
                    .unwrap_or_else(|| cx.palette().clone()),
            })
            .collect();
        let selected = cx
            .state()
            .custom
            .and_then(|id| cx.state().library.entries.iter().find(|e| e.id == id));
        let (name, palette) = selected
            .map(|e| (e.name.clone(), e.palette.clone()))
            .unwrap_or_else(|| {
                (
                    choices[preset].name.clone(),
                    choices[preset].palette.clone(),
                )
            });
        let mut state = cx.state().clone();
        state.slot = state.slot.min(palette.order.len() - 1);
        PaletteView {
            name,
            palette,
            choices,
            preset,
            state,
        }
    }

    fn interact(
        &self,
        cx: &dyn PaletteCx,
        action: PaletteAction,
    ) -> Result<PaletteOutcome, PaletteError> {
        let view = self.view(cx);
        let mut out = PaletteOutcome {
            state: cx.state().clone(),
            ..Default::default()
        };
        match action {
            PaletteAction::LoadLibrary(bytes) => {
                out.state.library = library::load(bytes.as_deref())?
            }
            PaletteAction::ChoosePreset(index) => {
                let choice = CHOICES
                    .get(index)
                    .ok_or_else(|| PaletteError::Message("Palette no longer exists".into()))?;
                out.setting = Some((PRESET_SETTING.into(), SettingValue::Choice(choice.0.into())));
                out.state.custom = None;
            }
            PaletteAction::ChooseCustom(id) => {
                out.state
                    .library
                    .entries
                    .iter()
                    .find(|e| e.id == id)
                    .ok_or_else(|| PaletteError::Message("Palette no longer exists".into()))?;
                out.state.custom = Some(id);
            }
            PaletteAction::SelectSlot(slot) => {
                out.state.slot = slot.min(view.palette.order.len() - 1)
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
                let (index, color) = library::step_swatch(&view.palette, cx.active_color(), delta);
                out.color = Some(color);
                out.selected = Some(index);
            }
            PaletteAction::Edit {
                operation,
                name,
                color,
            } => {
                out.state.slot = view.state.slot;
                edit_library(&view, &mut out, operation, name, color)?;
                out.library_bytes = Some(library::encode(&out.state.library)?);
            }
            PaletteAction::ImportLibrary { name, format, text } => {
                let palette = import(&format, &text)?;
                out.state.custom = Some(library::create(
                    &mut out.state.library,
                    name.chars().take(100).collect(),
                    palette.clone(),
                )?);
                out.state.slot = out.state.slot.min(palette.order.len() - 1);
                out.library_bytes = Some(library::encode(&out.state.library)?);
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
        Ok(out)
    }
}

fn edit_library(
    view: &PaletteView,
    out: &mut PaletteOutcome,
    operation: PaletteOperation,
    name: String,
    color: String,
) -> Result<(), PaletteError> {
    use PaletteOperation::*;
    let slot = out.state.slot;
    if matches!(operation, Create | Duplicate) {
        out.state.custom = Some(library::create(
            &mut out.state.library,
            if operation == Create {
                "Untitled palette".into()
            } else {
                format!("{} copy", name.chars().take(100).collect::<String>())
            },
            if operation == Create {
                Palette::default()
            } else {
                view.palette.clone()
            },
        )?);
    } else {
        let i = out
            .state
            .custom
            .and_then(|id| out.state.library.entries.iter().position(|e| e.id == id))
            .ok_or_else(|| {
                PaletteError::Message("Duplicate this palette to customize it".into())
            })?;
        match operation {
            Delete => {
                out.state.library.entries.remove(i);
                out.state.custom = None;
            }
            Rename => out.state.library.entries[i].name = name,
            _ => {
                let mut colors = view
                    .palette
                    .order
                    .iter()
                    .map(|s| view.palette.get(*s).unwrap().clone())
                    .collect::<Vec<_>>();
                match operation {
                    ApplyColor => colors[slot] = library::parse_color(&color)?,
                    AddSlot => colors.push(library::parse_color(&color)?),
                    RemoveSlot => {
                        colors.remove(slot);
                    }
                    MoveLeft if slot > 0 => colors.swap(slot, slot - 1),
                    MoveRight if slot + 1 < colors.len() => colors.swap(slot, slot + 1),
                    _ => {}
                }
                library::set_colors(&mut out.state.library, i, colors)?;
            }
        }
    }
    out.state.slot = match operation {
        MoveLeft => slot.saturating_sub(1),
        MoveRight => (slot + 1).min(view.palette.order.len() - 1),
        _ => slot,
    };
    let palette = out
        .state
        .custom
        .and_then(|id| out.state.library.entries.iter().find(|e| e.id == id))
        .map(|e| &e.palette)
        .unwrap_or(&view.choices[view.preset].palette);
    out.state.slot = out.state.slot.min(palette.order.len() - 1);
    Ok(())
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
