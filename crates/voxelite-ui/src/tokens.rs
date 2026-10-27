//! Shared color, typography, geometry and elevation tokens.
use gpui::{Hsla, rgb};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Application appearance, shared by every shell component.
pub enum Theme {
    /// Dark surfaces.
    Dark,
    /// Light surfaces.
    Light,
}
impl Theme {
    /// Canvas background.
    pub fn background(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x000000
        } else {
            0xffffff
        })
        .into()
    }
    /// Primary text.
    pub fn text(self) -> Hsla {
        rgb(if self == Self::Dark {
            0xfafafa
        } else {
            0x171717
        })
        .into()
    }
    /// Secondary text.
    pub fn muted(self) -> Hsla {
        rgb(if self == Self::Dark {
            0xa3a3a3
        } else {
            0x525252
        })
        .into()
    }
    /// Canvas separators.
    pub fn line(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x232323
        } else {
            0xe5e5e5
        })
        .into()
    }
    /// Canvas hover fill.
    pub fn hover(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x141414
        } else {
            0xf5f5f5
        })
        .into()
    }
    /// A quiet inset edge keeps black and white swatches visible on the canvas.
    pub fn swatch_edge(self) -> Hsla {
        self.text().opacity(0.18)
    }
    /// Legible accent foreground.
    pub fn accent(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x93c5fd
        } else {
            0x1d4ed8
        })
        .into()
    }
    /// Quiet selected fill.
    pub fn selected(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x142033
        } else {
            0xeff5ff
        })
        .into()
    }
    /// Selected text background, distinct from a selected control surface.
    pub fn text_selection(self) -> Hsla {
        self.accent().opacity(0.25)
    }
    /// Destructive foreground.
    pub fn danger(self) -> Hsla {
        rgb(if self == Self::Dark {
            0xfca5a5
        } else {
            0xb91c1c
        })
        .into()
    }
    /// Disabled foreground.
    pub fn disabled(self) -> Hsla {
        rgb(0x737373).into()
    }
    /// Conventional world-axis colors with legible inverse labels in both themes.
    pub fn axis(self, axis: usize) -> Hsla {
        rgb(match self {
            Self::Dark => [0xf87171, 0x86efac, 0x93c5fd][axis],
            Self::Light => [0xb91c1c, 0x15803d, 0x1d4ed8][axis],
        })
        .into()
    }
}

impl Theme {
    /// Pick the dark or light color for a semantic token.
    fn pair(self, dark: u32, light: u32) -> Hsla {
        rgb(if self == Self::Dark { dark } else { light }).into()
    }
    /// Window surface used by settings and dialogs.
    pub fn surface(self) -> Hsla {
        self.pair(0x1e1e1e, 0xf5f5f7)
    }
    /// Raised controls and search fields.
    pub fn surface_raised(self) -> Hsla {
        self.pair(0x2b2b2b, 0xe0e0e3)
    }
    /// Grouped form background.
    pub fn card(self) -> Hsla {
        self.pair(0x262626, 0xffffff)
    }
    /// Sidebar background.
    pub fn sidebar(self) -> Hsla {
        self.pair(0x191919, 0xebebed)
    }
    /// Secondary control fill.
    pub fn control(self) -> Hsla {
        self.pair(0x353535, 0xe6e6e9)
    }
    /// Group and control hairlines.
    pub fn separator(self) -> Hsla {
        self.pair(0x393939, 0xd7d7db)
    }
    /// Filled selection / switch tint.
    pub fn tint(self) -> Hsla {
        rgb(0x007aff).into()
    }
    /// Foreground on the filled tint.
    pub fn accent_fg(self) -> Hsla {
        rgb(0xffffff).into()
    }
    /// Modal backdrop.
    pub fn scrim(self) -> Hsla {
        gpui::black().opacity(0.38)
    }
    /// Quiet backdrop for the small About panel.
    pub fn quiet_scrim(self) -> Hsla {
        gpui::black().opacity(0.12)
    }
}

/// Logical pixel spacing scale.
pub mod space {
    /// Two pixels.
    pub const XXS: f32 = 2.;
    /// Four pixels.
    pub const XS: f32 = 4.;
    /// Six pixels.
    pub const SM: f32 = 6.;
    /// Eight pixels.
    pub const MD: f32 = 8.;
    /// Twelve pixels.
    pub const LG: f32 = 12.;
    /// Sixteen pixels.
    pub const XL: f32 = 16.;
    /// Twenty-four pixels.
    pub const XXL: f32 = 24.;
    /// Thirty-two pixels.
    pub const XXXL: f32 = 32.;
}
/// Corner radii.
pub mod radius {
    /// Small controls and swatches.
    pub const SM: f32 = 4.;
    /// Standard controls.
    pub const MD: f32 = 6.;
    /// Panels.
    pub const LG: f32 = 8.;
    /// Grouped cards and dialogs.
    pub const XL: f32 = 12.;
}
/// Font families, type scale and weights.
pub mod type_size {
    /// UI font.
    pub const UI: &str = "Geist";
    /// Fixed advance font for values and code.
    pub const MONO: &str = "Geist Mono";
    /// Captions and status values.
    pub const CAPTION: f32 = 11.;
    /// Compact body text.
    pub const BODY: f32 = 12.;
    /// Form body text.
    pub const BODY_LG: f32 = 13.;
    /// Section titles.
    pub const TITLE: f32 = 14.;
    /// Dialog titles.
    pub const HEADLINE: f32 = 17.;
    /// Page titles.
    pub const DISPLAY: f32 = 24.;
    /// Body line height.
    pub const LINE: f32 = 20.;
    /// Compact line height.
    pub const LINE_SM: f32 = 16.;
    /// Normal text.
    pub const NORMAL: gpui::FontWeight = gpui::FontWeight::NORMAL;
    /// Control labels.
    pub const MEDIUM: gpui::FontWeight = gpui::FontWeight::MEDIUM;
    /// Section headings.
    pub const SEMIBOLD: gpui::FontWeight = gpui::FontWeight::SEMIBOLD;
    /// Window headings.
    pub const BOLD: gpui::FontWeight = gpui::FontWeight::BOLD;
    /// Geist Mono advance, in em units (from the bundled font's metrics).
    pub const MONO_ADVANCE: f32 = 0.6;
}
/// Shared control sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    /// Compact 24-pixel control.
    Sm,
    /// Standard 32-pixel control.
    #[default]
    Md,
    /// Large 40-pixel control.
    Lg,
}
impl ControlSize {
    /// Height in logical pixels.
    pub const fn height(self) -> f32 {
        match self {
            Self::Sm => 24.,
            Self::Md => 32.,
            Self::Lg => 40.,
        }
    }
}
/// Geometry shared across component implementations.
pub mod metrics {
    /// Hairline width.
    pub const EDGE: f32 = 1.;
    /// Standard icon.
    pub const ICON: f32 = 17.;
    /// Compact icon.
    pub const ICON_SM: f32 = 12.;
    /// Status bar height.
    pub const STATUS_HEIGHT: f32 = 28.;
    /// Minimum window width for full-range selection telemetry.
    pub const STATUS_COMPACT_WIDTH: f32 = 1360.;
    /// Toolbar height.
    pub const TOOLBAR_HEIGHT: f32 = 64.;
    /// Form row minimum height.
    pub const FORM_HEIGHT: f32 = 68.;
    /// Slider track.
    pub const TRACK: f32 = 4.;
    /// Slider / switch thumb.
    pub const THUMB: f32 = 16.;
    /// Switch width.
    pub const SWITCH: f32 = 40.;
    /// Default dialog width.
    pub const DIALOG_WIDTH: f32 = 560.;
    /// About panel width.
    pub const ABOUT_WIDTH: f32 = 320.;
    /// About panel height.
    pub const ABOUT_HEIGHT: f32 = 400.;
    /// About icon size.
    pub const APP_ICON: f32 = 96.;
}
/// Keyboard focus ring, outside the control without affecting layout.
pub fn focus_ring(theme: Theme) -> Vec<gpui::BoxShadow> {
    use gpui::{point, px};
    vec![gpui::BoxShadow {
        inset: false,
        color: theme.accent(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(space::XXS),
    }]
}
/// Subtle panel elevation; no layout impact.
pub fn panel_shadow() -> Vec<gpui::BoxShadow> {
    use gpui::{point, px};
    vec![gpui::BoxShadow {
        inset: false,
        color: gpui::black().opacity(0.18),
        offset: point(px(0.), px(space::XXS)),
        blur_radius: px(space::MD),
        spread_radius: px(0.),
    }]
}
