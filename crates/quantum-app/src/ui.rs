//! Native GPUI shell state and shared helpers. Portable editing logic lives in the library.
//!
//! `lifecycle` constructs and replaces editor sessions; `dispatch` runs commands;
//! `keyboard` routes keys; `layout` composes frames. `chrome`, `viewport_controls`,
//! `sidebar` and `panel_*` build the editor surfaces; `dialogs` selects overlays.
mod about;

/// Version shown in the UI. Pinned in `ui-snapshots` builds so release bumps
/// do not invalidate the committed goldens.
pub(crate) const APP_VERSION: &str = if cfg!(feature = "ui-snapshots") {
    "0.0.0"
} else {
    env!("CARGO_PKG_VERSION")
};

/// Optional build identifier shown in About; omitted from `ui-snapshots` builds.
pub(crate) const APP_BUILD: Option<&str> = if cfg!(feature = "ui-snapshots") {
    None
} else {
    option_env!("QUANTUM_BUILD")
};
mod agent;
mod camera_anim;
#[cfg(target_os = "linux")]
mod capture;
mod chrome;
mod command_palette;
mod comments;
mod debug_overlay;
mod dialogs;
mod dispatch;
pub(crate) mod feedback;
mod history;
mod keyboard;
mod keyboard_guide;
pub(crate) mod launch;
mod layout;
mod lifecycle;
mod loading;
mod overlays;
mod palette_layout;
mod palettes;
mod panel_history;
mod panel_layers;
mod panel_palette;
mod save_flow;
mod settings;
mod shortcut_hint;
mod sidebar;
#[cfg(feature = "ui-snapshots")]
pub(crate) mod snapshots;
mod status;
mod tooltip;
mod viewport;
mod viewport_controls;
pub(crate) mod welcome;
mod welcome_style;

use crate::{
    actions::{CommandContext, EditorState, MENUS, Op},
    shortcuts::{ShortcutPlatform, navigation_key},
    theme::Theme,
};
use chrome::ActionPresentation;
use comments::SidebarTab;
use glam::DVec3;
use gpui::{prelude::*, *};
use keyboard_guide::keyboard_guide;
use quantum_app::editor::{Editor, EditorResult, OrbitCamera, Tool};
use quantum_core::{Palette, Pos};
use quantum_plugin_api::{PaletteAction, PalettePresentation};
use quantum_render::{PresetView, RenderOptions, View};
use quantum_ui::tokens::{focus_ring, metrics, radius, type_size};
use quantum_ui::{
    Button, ButtonStyle, ButtonVariant, Dialog, Toolbar, ToolbarSeparator, icon, mono, row,
};
use shortcut_hint::{HintText, action_shortcut, shortcut};
use std::{cell::Cell, path::PathBuf, rc::Rc, sync::Arc};
use tooltip::{AnchoredTooltip, TooltipSide};

// Render sizing and native layout use the same shared logical-pixel metrics.
use quantum_ui::tokens::metrics::{
    MENUBAR_HEIGHT, PANEL_GUTTER, RAIL_WIDTH as TOOL_RAIL_WIDTH, SIDEBAR_WIDTH,
    STATUS_HEIGHT as STATUSBAR_HEIGHT, TOOLBAR_HEIGHT,
};
use quantum_ui::{IconButton, Panel, Popover, Segment, SegmentedControl, Tab, TabBar};

pub(crate) fn updates_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::updater::is_available()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

// Both windows use Sparkle's own progress and result dialogs when available.
fn check_for_updates() {
    #[cfg(target_os = "macos")]
    if updates_available() {
        crate::updater::check_for_updates();
    }
}

pub struct EditorView {
    overlays: Entity<quantum_ui::OverlayState>,
    capture_state: Option<launch::LaunchState>,
    palettes: palettes::PaletteManager,
    discussion: comments::Discussion,
    panel: SidebarTab,
    timeline: history::Timeline,
    agent: agent::Agent,
    editor: Editor,
    theme: Theme,
    preferences: quantum_app::preferences::Preferences,
    reduced_motion: bool,
    settings_window: Option<WindowHandle<settings::SettingsView>>,
    palette_focus: FocusHandle,
    modal_focus: FocusHandle,
    sidebar_scroll: ScrollHandle,
    focus: FocusHandle,
    viewport: viewport::Viewport,
    last_title: String,
    last_edited: Option<bool>,
    zoom_reference_span: f64,
    camera_view: Option<Op>,
    camera_fitted: bool,
    camera_anim: quantum_render::CameraAnimator,
    anim_epoch: std::time::Instant,
    hover: Option<Pos>,
    cursor: Pos,
    drag: Option<Point<Pixels>>,
    overlay_drag: Option<(String, quantum_plugin_api::OverlayDrag)>,
    overlay_hover: Option<(String, u32)>,
    overlay_bounds: Rc<std::cell::RefCell<std::collections::BTreeMap<String, Bounds<Pixels>>>>,
    menu: Option<usize>,
    menu_bounds: Rc<Cell<Bounds<Pixels>>>,
    menu_index: usize,
    commands: command_palette::CommandPalette,
    command_input: Entity<crate::text_field::TextField>,
    palette_open: bool,
    palette_cursor: usize,
    palette_scroll: ScrollHandle,
    swatch_scroll: ScrollHandle,
    save_flow: save_flow::SaveFlow,
    info: Option<(&'static str, InfoContent)>,
    status: HintText,
    error: bool,
}

enum InfoContent {
    About,
    Acknowledgements,
    Text(String),
    Guide,
}

fn section_title(text: &'static str) -> Div {
    Panel::title(text)
}

fn color_fill(color: [u8; 4]) -> Hsla {
    rgb(u32::from(color[0]) << 16 | u32::from(color[1]) << 8 | u32::from(color[2])).into()
}

/// Choose a legible checkmark on the actual sRGB swatch, in either theme.
fn swatch_ink(color: [u8; 4]) -> Hsla {
    let linear = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance =
        0.2126 * linear(color[0]) + 0.7152 * linear(color[1]) + 0.0722 * linear(color[2]);
    Theme::contrast_ink(luminance > 0.179)
}
