//! Built-in camera orientation widget and ground grid. All edits remain typed commands.
use voxelite_plugin_api::*;
mod gizmo;

/// Built-in identity, independently disableable from voxel editing tools.
pub const PLUGIN_ID: &str = "dev.voxelite.gizmo-grid";
/// Stable grid command ID, retained for existing shortcuts and shell mappings.
pub const GRID_COMMAND: &str = "dev.voxelite.core.grid";
/// Persisted by the app through its existing `grid` preference key.
pub const GRID_SETTING: &str = "dev.voxelite.gizmo-grid.grid_enabled";
/// Ground grid render-pass binding.
pub const GRID_OVERLAY: &str = "dev.voxelite.gizmo-grid.grid";
/// Interactive orientation render-pass binding.
pub const GIZMO_OVERLAY: &str = "dev.voxelite.gizmo-grid.gizmo";

/// Camera gizmo interaction, spatial reference drawing and grid metadata.
pub struct GizmoGridPlugin;
impl Plugin for GizmoGridPlugin {
    fn manifest(&self) -> Manifest {
        Manifest::builtin(PLUGIN_ID, "plugins.gizmo-grid", env!("CARGO_PKG_VERSION"))
    }
    fn contribute(&self, registrar: &mut Registrar) {
        registrar.command(CommandContribution {
            id: GRID_COMMAND.into(),
            label_key: "Toggle grid".into(),
            args_schema: Value::Object(
                [
                    ("type".into(), Value::String("object".into())),
                    ("properties".into(), Value::Object(Default::default())),
                    ("additionalProperties".into(), Value::Bool(false)),
                ]
                .into_iter()
                .collect(),
            ),
            default_keybinding: None,
            context: None,
            icon: Some("grid".into()),
            menu: Some("View".into()),
            order: 150,
            application_section: None,
            agent_visible: false,
        });
        registrar.setting(SettingContribution {
            key: GRID_SETTING.into(),
            label_key: "Ground grid".into(),
            kind: SettingKind::Bool,
            default: SettingValue::Bool(true),
        });
        registrar.render_pass(RenderPassContribution {
            id: GRID_OVERLAY.into(),
            slot: RenderPassSlot::BeforeScene,
        });
        registrar.render_pass(RenderPassContribution {
            id: GIZMO_OVERLAY.into(),
            slot: RenderPassSlot::Overlay,
        });
    }
    fn overlay(&self, id: &str) -> Option<&dyn Overlay> {
        match id {
            GRID_OVERLAY => Some(&Grid),
            GIZMO_OVERLAY => Some(&gizmo::Gizmo),
            _ => None,
        }
    }
}

struct Grid;
impl Overlay for Grid {
    fn draw(&self, cx: &mut dyn OverlayCx, _: Option<u32>) {
        if cx.setting(GRID_SETTING) != Some(&SettingValue::Bool(true)) {
            return;
        }
        let bounds = cx.bounds();
        cx.emit(OverlayPrimitive::Grid(OverlayGrid {
            plane: bounds.min[1],
            offset: -0.05,
            min: bounds.min[0].min(bounds.min[2]) - 16,
            max: bounds.max[0].max(bounds.max[2]) + 16,
            spacing: 4,
            width: 0.6,
            color: OverlayColor::Line,
            opacity: 0.24,
        }));
    }
}

#[cfg(test)]
mod tests;
