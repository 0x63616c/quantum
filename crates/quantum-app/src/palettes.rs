//! Host adapters for the registered palette panel. Feature behavior lives in the plugin.
use crate::{
    editor::{Editor, EditorError, EditorResult},
    local_settings,
};
use quantum_plugin_api::{
    Command, Palette, PaletteAction, PaletteCx, PaletteOutcome, PaletteState, PaletteView,
    SettingValue, Swatch,
};
use quantum_plugin_palette::PANEL_ID;

impl Editor {
    /// Retain the window's custom library and selection when replacing its document.
    pub fn retain_palette_library(&mut self, previous: &Self) {
        self.palette_state = previous.palette_state.clone();
    }
    /// Whether the palette panel is present in the live registry.
    pub fn palette_enabled(&self) -> bool {
        self.plugins
            .borrow()
            .registry()
            .panels()
            .any(|(_, p)| p.id == PANEL_ID)
    }

    /// Effective tool color; without the palette feature tools use the core default.
    pub fn active_color(&self) -> &Swatch {
        static DEFAULT: std::sync::LazyLock<Swatch> = std::sync::LazyLock::new(Swatch::default);
        if self.palette_enabled() {
            &self.color
        } else {
            &DEFAULT
        }
    }

    /// Read the enabled panel's portable presentation state.
    pub fn palette_view(&self) -> Option<PaletteView> {
        self.plugins
            .borrow_mut()
            .palette_view(PANEL_ID, &EditorPaletteCx(self))
            .ok()
            .flatten()
    }

    /// Load the existing local library through host-owned bounded storage.
    pub fn load_palette_library(&mut self) -> EditorResult<()> {
        if self.palette_enabled() {
            let value = local_settings::json_store::<Option<serde_json::Value>>(
                &local_settings::data_dir().join("palettes.json"),
            )?
            .load()?;
            let bytes = value.map(|value| serde_json::to_vec(&value)).transpose()?;
            self.palette_action(PaletteAction::LoadLibrary(bytes))?;
        }
        Ok(())
    }

    /// Dispatch a palette request and persist any library edit before installing it.
    pub fn palette_action(
        &mut self,
        action: PaletteAction,
    ) -> EditorResult<Option<PaletteOutcome>> {
        self.palette_action_with(action, |bytes| {
            let value: serde_json::Value = serde_json::from_slice(bytes)?;
            Ok(local_settings::json_store::<Option<serde_json::Value>>(
                &local_settings::data_dir().join("palettes.json"),
            )?
            .save(&Some(value))?)
        })
    }

    /// Admit palette effects using a host-provided atomic library writer.
    /// A failed write or core transaction preserves selection and library state.
    pub fn palette_action_with(
        &mut self,
        action: PaletteAction,
        persist: impl FnOnce(&[u8]) -> EditorResult<()>,
    ) -> EditorResult<Option<PaletteOutcome>> {
        let outcome =
            self.plugins
                .borrow_mut()
                .dispatch_palette(PANEL_ID, &EditorPaletteCx(self), action)?;
        let Some(outcome) = outcome else {
            return Ok(None);
        };
        // A request has one durable authority: a local library or the core engine.
        if outcome.library_bytes.is_some() && !outcome.commands.is_empty() {
            return Err(EditorError::MixedPaletteEdits);
        }
        if !outcome.commands.is_empty() {
            self.execute("Edit palette", Command::Batch(outcome.commands.clone()))?;
        }
        if let Some(bytes) = &outcome.library_bytes {
            persist(bytes)?;
        }
        if let Some((key, value)) = &outcome.setting {
            self.plugins.get_mut().set_setting(key, value.clone())?;
        }
        self.palette_state = outcome.state.clone();
        if let Some(color) = &outcome.color {
            self.color = color.clone();
        }
        Ok(Some(outcome))
    }
}

struct EditorPaletteCx<'a>(&'a Editor);
impl PaletteCx for EditorPaletteCx<'_> {
    fn palette(&self) -> &Palette {
        &self.0.display_snapshot().palette
    }
    fn active_color(&self) -> &Swatch {
        &self.0.color
    }
    fn state(&self) -> &PaletteState {
        &self.0.palette_state
    }
    fn setting(&self, _: &str) -> Option<&SettingValue> {
        None
    }
}

#[cfg(test)]
#[path = "palette_tests.rs"]
mod tests;
