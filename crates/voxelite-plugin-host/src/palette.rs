use super::*;
use voxelite_plugin_api::{
    Palette, PaletteAction, PaletteCx, PaletteOutcome, PaletteState, PaletteView, Swatch,
};

impl Host {
    /// Whether a declared, enabled panel supplies portable palette behavior.
    pub fn has_palette_panel(&mut self, id: &str) -> Result<bool, HostError> {
        let Some(index) = self.palette_owner(id) else {
            return Ok(false);
        };
        self.invoke(index, Phase::Palette, |plugin| {
            plugin.palette_panel(id).is_some()
        })
    }

    /// Read presentation data from an enabled panel through owner-scoped capabilities.
    pub fn palette_view(
        &mut self,
        id: &str,
        cx: &dyn PaletteCx,
    ) -> Result<Option<PaletteView>, HostError> {
        let Some(index) = self.palette_owner(id) else {
            return Ok(None);
        };
        let cx = PaletteContext {
            source: cx,
            settings: self.plugins[index].settings.clone(),
        };
        self.invoke(index, Phase::Palette, |plugin| {
            plugin.palette_panel(id).map(|panel| panel.view(&cx))
        })
    }

    /// Propose palette effects without admitting any document, state or filesystem changes.
    /// Stale shell commands are checked against the same owner's enabled declarations.
    pub fn dispatch_palette(
        &mut self,
        id: &str,
        cx: &dyn PaletteCx,
        action: PaletteAction,
    ) -> Result<Option<PaletteOutcome>, HostError> {
        let Some(index) = self.palette_owner(id) else {
            return Ok(None);
        };
        if let PaletteAction::Command(command) = &action
            && !self.plugins[index]
                .contributions
                .iter()
                .any(|c| matches!(c, Contribution::Command(spec) if spec.id == *command))
        {
            return Ok(None);
        }
        let cx = PaletteContext {
            source: cx,
            settings: self.plugins[index].settings.clone(),
        };
        let outcome = self
            .invoke(index, Phase::Palette, |plugin| {
                plugin
                    .palette_panel(id)
                    .map(|panel| panel.interact(&cx, action))
            })?
            .transpose()?;
        if let Some((key, value)) = outcome.as_ref().and_then(|out| out.setting.as_ref()) {
            let valid = self.plugins[index].contributions.iter().any(|c| {
                let Contribution::Setting(setting) = c else {
                    return false;
                };
                setting.key == *key
                    && match (&setting.kind, value) {
                        (SettingKind::Bool, SettingValue::Bool(_)) => true,
                        (SettingKind::Choice(choices), SettingValue::Choice(value)) => {
                            choices.contains(value)
                        }
                        (SettingKind::Number, SettingValue::Number(value)) => value.is_finite(),
                        _ => false,
                    }
            });
            if !valid {
                return Err(HostError::InvalidSetting { key: key.clone() });
            }
        }
        Ok(outcome)
    }

    fn palette_owner(&self, id: &str) -> Option<usize> {
        self.plugins.iter().position(|plugin| {
            plugin.state == PluginState::Enabled
                && plugin
                    .contributions
                    .iter()
                    .any(|c| matches!(c, Contribution::Panel(panel) if panel.id == id))
        })
    }
}

struct PaletteContext<'a> {
    source: &'a dyn PaletteCx,
    settings: BTreeMap<String, SettingValue>,
}
impl PaletteCx for PaletteContext<'_> {
    fn palette(&self) -> &Palette {
        self.source.palette()
    }
    fn active_color(&self) -> &Swatch {
        self.source.active_color()
    }
    fn state(&self) -> &PaletteState {
        self.source.state()
    }
    fn setting(&self, key: &str) -> Option<&SettingValue> {
        self.settings.get(key)
    }
}
