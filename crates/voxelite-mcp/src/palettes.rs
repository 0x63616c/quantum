//! Typed palette requests resolved through the editor's enabled plugin registry.
use crate::error::{Failure, Result};
use voxelite_plugin_api::{
    PALETTE_PANEL_ID as PANEL_ID, PALETTE_PRESET_SETTING as PRESET_SETTING, Palette, PaletteAction,
    PaletteCx, PaletteError, PaletteOutcome, PaletteState, SettingKind, SettingValue, Swatch,
};
use voxelite_plugin_host::{Host, HostError};

pub(crate) fn enabled(host: &Host) -> bool {
    host.registry().panels().any(|(_, p)| p.id == PANEL_ID)
}

fn unavailable() -> Failure {
    Failure::new("palette_unavailable", "Palette plugin is unavailable")
}

pub(crate) fn presets(host: &Host) -> Result<Vec<String>> {
    host.registry()
        .settings()
        .find_map(|(_, setting)| {
            if setting.key == PRESET_SETTING
                && let SettingKind::Choice(choices) = &setting.kind
            {
                Some(choices.iter().skip(1).cloned().collect())
            } else {
                None
            }
        })
        .ok_or_else(unavailable)
}

pub(crate) fn dispatch(
    host: &mut Host,
    palette: &Palette,
    action: PaletteAction,
) -> Result<PaletteOutcome> {
    host.dispatch_palette(
        PANEL_ID,
        &DocumentPalette {
            palette,
            state: PaletteState::default(),
            color: Swatch::default(),
        },
        action,
    )
    .map_err(|error| match error {
        HostError::Palette(error @ PaletteError::Capacity) => {
            Failure::new("limit_exceeded", error.to_string())
        }
        HostError::Palette(PaletteError::Core(error)) => error.into(),
        HostError::Palette(PaletteError::Message(message)) => Failure::invalid(message),
        error => Failure::new("palette_failed", error.to_string()),
    })?
    .ok_or_else(unavailable)
}

struct DocumentPalette<'a> {
    palette: &'a Palette,
    state: PaletteState,
    color: Swatch,
}
impl PaletteCx for DocumentPalette<'_> {
    fn palette(&self) -> &Palette {
        self.palette
    }
    fn state(&self) -> &PaletteState {
        &self.state
    }
    fn active_color(&self) -> &Swatch {
        &self.color
    }
    fn setting(&self, _: &str) -> Option<&SettingValue> {
        None
    }
}
