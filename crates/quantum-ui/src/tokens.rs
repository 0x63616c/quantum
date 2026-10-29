//! The design system's sole palette, type and geometry scales.
//! Compatibility aliases retain shared layout metrics for existing callers.
mod legacy;
use gpui::{FontWeight, Hsla, Styled, px, rgb};
pub use legacy::*;

/// Application appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    /// Dark surfaces.
    Dark,
    /// Light surfaces.
    Light,
}

/// Semantic colors. Never choose colors by screen or component name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorToken {
    /// Viewport and page ground.
    Canvas,
    /// Sidebar and chrome.
    Surface,
    /// Cards and inputs.
    Raised,
    /// Floating content.
    Overlay,
    /// Internal dividers.
    BorderSubtle,
    /// Control and panel edges.
    Border,
    /// Body text and headings.
    TextPrimary,
    /// Descriptions and metadata.
    TextSecondary,
    /// Placeholders.
    TextTertiary,
    /// Filled primary actions.
    Accent,
    /// Links, selected icons and focus.
    AccentText,
    /// Destructive text and icons.
    Danger,
    /// Foreground on filled primary actions.
    OnAccent,
    /// Positive feedback.
    Success,
}
impl ColorToken {
    /// Every entry, in audit order followed by semantic extensions.
    pub const ALL: [Self; 14] = [
        Self::Canvas,
        Self::Surface,
        Self::Raised,
        Self::Overlay,
        Self::BorderSubtle,
        Self::Border,
        Self::TextPrimary,
        Self::TextSecondary,
        Self::TextTertiary,
        Self::Accent,
        Self::AccentText,
        Self::Danger,
        Self::OnAccent,
        Self::Success,
    ];
}
/// A theme-resolved semantic palette; contains no component-specific colors.
#[derive(Clone, Copy, Debug)]
pub struct Palette(Theme);
impl Palette {
    /// Resolve an exact sRGB value for exports and contrast audits.
    pub const fn rgb(self, token: ColorToken) -> u32 {
        use ColorToken::*;
        let (dark, light) = match token {
            Canvas => (0x0b0b0c, 0xf5f5f7),
            Surface => (0x111113, 0xffffff),
            Raised => (0x18181b, 0xf4f4f5),
            Overlay => (0x1f1f23, 0xffffff),
            BorderSubtle => (0x232326, 0xe4e4e7),
            Border => (0x2e2e33, 0xd4d4d8),
            TextPrimary => (0xededef, 0x18181b),
            TextSecondary => (0xa1a1aa, 0x52525b),
            TextTertiary => (0x8b8b94, 0x65656f),
            Accent => (0x2563eb, 0x2563eb),
            AccentText => (0x93c5fd, 0x1d4ed8),
            Danger => (0xf87171, 0xb91c1c),
            OnAccent => (0xffffff, 0xffffff),
            Success => (0x86efac, 0x166534),
        };
        match self.0 {
            Theme::Dark => dark,
            Theme::Light => light,
        }
    }
    /// Resolve a GPUI color.
    pub fn color(self, token: ColorToken) -> Hsla {
        rgb(self.rgb(token)).into()
    }
}
impl Theme {
    /// Resolve the single design-system palette.
    pub const fn palette(self) -> Palette {
        Palette(self)
    }
    /// Resolve a semantic color.
    pub fn color(self, token: ColorToken) -> Hsla {
        self.palette().color(token)
    }
}

/// Complete spacing scale, named after the audit's space-0…space-8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Space {
    /// Flush.
    S0,
    /// 2 px.
    S1,
    /// 4 px.
    S2,
    /// 8 px.
    S3,
    /// 12 px.
    S4,
    /// 16 px.
    S5,
    /// 24 px.
    S6,
    /// 32 px.
    S7,
    /// 48 px.
    S8,
}
impl Space {
    /// Logical pixels.
    pub const fn px(self) -> f32 {
        [0., 2., 4., 8., 12., 16., 24., 32., 48.][self as usize]
    }
}
/// Complete corner scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Radius {
    /// Keycaps, swatches and thumbnails: 4 px.
    Xs,
    /// Controls: 6 px.
    Sm,
    /// Panels and floating surfaces: 10 px.
    Md,
    /// Dialogs: 14 px.
    Lg,
    /// Circular shapes.
    Full,
}
impl Radius {
    /// Logical pixels.
    pub const fn px(self) -> f32 {
        [4., 6., 10., 14., 9999.][self as usize]
    }
}
/// Shared control heights. Rail controls retain their separate 36 px metric.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlHeight {
    /// Small / dense: 24 px.
    Dense,
    /// Medium / default: 28 px.
    #[default]
    Default,
    /// Large: 32 px.
    Large,
}
impl ControlHeight {
    /// Logical pixels.
    pub const fn px(self) -> f32 {
        [24., 28., 32.][self as usize]
    }
}
/// The only text sizes for new UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    /// 11/16, medium.
    Caption,
    /// 12/16, normal.
    Small,
    /// 13/20, normal.
    Body,
    /// 15/20, semibold.
    Title,
    /// 20/28, semibold.
    Heading,
    /// 28/34, semibold.
    Display,
    /// 12/16, normal, tabular Geist Mono.
    Mono,
}
impl TextSize {
    /// Font size in logical pixels.
    pub const fn px(self) -> f32 {
        [11., 12., 13., 15., 20., 28., 12.][self as usize]
    }
    /// Line height in logical pixels.
    pub const fn line_height(self) -> f32 {
        [16., 16., 20., 20., 28., 34., 16.][self as usize]
    }
    /// Font weight.
    pub const fn weight(self) -> FontWeight {
        match self {
            Self::Caption => FontWeight::MEDIUM,
            Self::Title | Self::Heading | Self::Display => FontWeight::SEMIBOLD,
            _ => FontWeight::NORMAL,
        }
    }
    /// Apply size, line height, weight and family together.
    pub fn apply<T: Styled>(self, element: T) -> T {
        let element = element
            .font_family(if self == Self::Mono {
                type_size::MONO
            } else {
                type_size::UI
            })
            .text_size(px(self.px()))
            .line_height(px(self.line_height()))
            .font_weight(self.weight());
        if self == Self::Mono {
            element.font_features(gpui::FontFeatures(std::sync::Arc::new(vec![(
                "tnum".into(),
                1,
            )])))
        } else {
            element
        }
    }
}

/// Interaction opacities and geometry, shared by every new control.
pub mod state {
    /// Short ease-out for pointer entry, exit and release, with no layout motion.
    pub const HOVER_DURATION: std::time::Duration = std::time::Duration::from_millis(120);
    /// Press feedback reaches its target sooner than hover feedback.
    pub const PRESS_DURATION: std::time::Duration = std::time::Duration::from_millis(80);
    /// Translucency of diagnostic chrome over the viewport.
    pub const DEBUG_OVERLAY: f32 = 0.82;
    /// Quiet thumbnail wash between canvas and hover fills.
    pub const THUMBNAIL_WASH: f32 = 0.5;
    /// Keyboard focus wash behind a palette's document-color cells.
    pub const PALETTE_FOCUS: f32 = 0.22;
    /// Subtle selected tint inside the illustrative appearance preview.
    pub const PREVIEW_SELECTED: f32 = 0.08;
    /// Neutral hover overlay.
    pub const HOVER: f32 = 0.05;
    /// Neutral pressed overlay.
    pub const PRESSED: f32 = 0.08;
    /// Accent selection overlay.
    pub const SELECTED: f32 = 0.16;
    /// Whole-control disabled opacity.
    pub const DISABLED: f32 = 0.4;
    /// The single modal scrim color.
    pub fn scrim() -> gpui::Hsla {
        gpui::black().opacity(SCRIM)
    }
    /// Modal backdrop opacity.
    pub const SCRIM: f32 = 0.5;
    /// Primary hover mixes 6% white into the accent in sRGB.
    pub const PRIMARY_HOVER: f32 = 0.06;
    /// Focus stroke thickness.
    pub const FOCUS_WIDTH: f32 = 2.;
    /// Focus stroke separation from the control edge.
    pub const FOCUS_OFFSET: f32 = 1.;
    /// Navigation selection bar thickness.
    pub const INDICATOR: f32 = 2.;
    /// Default tooltip dwell time.
    pub const TOOLTIP_DELAY: std::time::Duration = std::time::Duration::from_millis(500);
}
/// Component geometry that is not spacing or typography.
pub mod geometry {
    /// Width of the development-only control state audit surface.
    pub const CONTROL_AUDIT_WIDTH: f32 = 800.;
    /// Fixed state label slot in the control audit.
    pub const STATE_LABEL_WIDTH: f32 = 96.;
    /// Largest dialog fraction of the available window.
    pub const DIALOG_MAX_FRACTION: f32 = 0.95;
    /// Sidebar fraction inside the illustrative appearance preview.
    pub const PREVIEW_SIDEBAR_FRACTION: f32 = 0.28;
    /// Artwork overscan on each edge of a sample thumbnail.
    pub const THUMBNAIL_OVERSCAN: f32 = 0.06;
    /// Full artwork scale, including both overscanned edges.
    pub const THUMBNAIL_SCALE: f32 = 1. + 2. * THUMBNAIL_OVERSCAN;
    /// Half-pixel overlap prevents seams between neighboring color-plane cells.
    pub const COLOR_CELL_OVERLAP: f32 = 0.5;
    /// Stroke for the projected voxel hover outline.
    pub const VOXEL_OUTLINE: f32 = 1.4;
    /// Settings slider width, independent of its changing value label.
    pub const SETTINGS_SLIDER: f32 = 200.;
    /// Hairline border.
    pub const EDGE: f32 = 1.;
    /// Standard glyph within dense/default controls.
    pub const ICON: f32 = 16.;
    /// Glyph within a rail action.
    pub const RAIL_ICON: f32 = 20.;
    /// Rail square; unchanged from the editor.
    pub const RAIL: f32 = 36.;
    /// Fixed default menu width.
    pub const MENU_WIDTH: f32 = 224.;
    /// Default dialog width.
    pub const DIALOG_WIDTH: f32 = 560.;
}

#[cfg(test)]
mod tests;
