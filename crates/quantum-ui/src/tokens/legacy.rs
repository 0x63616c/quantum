//! Compatibility names for shared layout metrics and the typed design tokens.
//! New components must use the typed tokens in the parent module.
use super::{ColorToken, Theme};
use gpui::{Hsla, rgb};

impl Theme {
    /// Existing application-modal scrim, retaining its exact 8-bit alpha.
    pub fn app_scrim(self) -> Hsla {
        super::state::scrim()
    }
    /// Appearance-preview artwork, separate from live UI surfaces.
    pub fn preview_background(self) -> Hsla {
        self.pair(0x303030, 0xfafafa)
    }
    /// Translucent titlebar/sidebar in appearance preview artwork.
    pub fn preview_chrome(self) -> Hsla {
        gpui::rgba(0x88888822).into()
    }
    /// Placeholder text strokes in appearance preview artwork.
    pub fn preview_content(self) -> Hsla {
        gpui::rgba(0x99999988).into()
    }
    /// Traffic-light artwork colors (not interactive controls).
    pub fn preview_lights() -> [Hsla; 3] {
        [
            rgb(0xff6058).into(),
            rgb(0xffbd2e).into(),
            rgb(0x28c840).into(),
        ]
    }
    /// Black or white ink for a color sample, chosen from its actual luminance.
    pub fn contrast_ink(light_background: bool) -> Hsla {
        rgb(if light_background { 0x000000 } else { 0xffffff }).into()
    }
    /// Canvas background.
    pub fn background(self) -> Hsla {
        self.color(ColorToken::Canvas)
    }
    /// Primary text.
    pub fn text(self) -> Hsla {
        self.color(ColorToken::TextPrimary)
    }
    /// Secondary text.
    pub fn muted(self) -> Hsla {
        self.color(ColorToken::TextSecondary)
    }
    /// Canvas separators.
    pub fn line(self) -> Hsla {
        self.color(ColorToken::BorderSubtle)
    }
    /// Canvas hover fill.
    pub fn hover(self) -> Hsla {
        self.color(ColorToken::Surface).blend(
            self.color(ColorToken::OnAccent)
                .opacity(super::state::HOVER),
        )
    }
    /// A quiet inset edge keeps black and white swatches visible on the canvas.
    pub fn swatch_edge(self) -> Hsla {
        self.text().opacity(0.18)
    }
    /// Legible accent foreground.
    pub fn accent(self) -> Hsla {
        self.color(ColorToken::AccentText)
    }
    /// Quiet selected fill.
    pub fn selected(self) -> Hsla {
        self.color(ColorToken::Accent)
            .opacity(super::state::SELECTED)
    }
    /// Selected text background, distinct from a selected control surface.
    pub fn text_selection(self) -> Hsla {
        self.accent().opacity(0.25)
    }
    /// Destructive foreground.
    pub fn danger(self) -> Hsla {
        self.color(ColorToken::Danger)
    }
    /// Disabled foreground.
    pub fn disabled(self) -> Hsla {
        self.color(ColorToken::TextPrimary)
            .opacity(super::state::DISABLED)
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
        self.color(ColorToken::Surface)
    }
    /// Raised controls and search fields.
    pub fn surface_raised(self) -> Hsla {
        self.color(ColorToken::Raised)
    }
    /// Grouped form background.
    pub fn card(self) -> Hsla {
        self.color(ColorToken::Raised)
    }
    /// Sidebar background.
    pub fn sidebar(self) -> Hsla {
        self.color(ColorToken::Surface)
    }
    /// Secondary control fill.
    pub fn control(self) -> Hsla {
        self.color(ColorToken::Raised)
    }
    /// Group and control hairlines.
    pub fn separator(self) -> Hsla {
        self.color(ColorToken::Border)
    }
    /// Filled selection / switch tint.
    pub fn tint(self) -> Hsla {
        self.color(ColorToken::Accent)
    }
    /// Foreground on the filled tint.
    pub fn accent_fg(self) -> Hsla {
        self.color(ColorToken::OnAccent)
    }
    /// Modal backdrop.
    pub fn scrim(self) -> Hsla {
        super::state::scrim()
    }
    /// Quiet backdrop for the small About panel.
    pub fn quiet_scrim(self) -> Hsla {
        super::state::scrim()
    }
}

/// Compatibility spacing aliases and shared optical exceptions.
pub mod space {
    /// No inset; also useful for flexible layout minimums and coordinate origins.
    pub const NONE: f32 = 0.;
    /// Three-pixel compact row separation.
    pub const COMPACT: f32 = 3.;
    /// Five-pixel optical spacing around small glyphs.
    pub const GLYPH: f32 = 5.;
    /// Seven-pixel shortcut and preview inset.
    pub const OPTICAL: f32 = 7.;
    /// Nine-pixel appearance caption offset.
    pub const CAPTION_OFFSET: f32 = 9.;
    /// Ten-pixel dense panel inset.
    pub const INSET: f32 = 10.;
    /// Fourteen-pixel command footer inset.
    pub const FOOTER: f32 = 14.;
    /// Eighteen-pixel section spacing.
    pub const SECTION: f32 = 18.;
    /// Twenty-pixel sidebar content inset.
    pub const PANEL: f32 = 20.;
    /// Twenty-eight-pixel settings pane gutter.
    pub const PANE: f32 = 28.;
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
/// Compatibility radius aliases for shared layout geometry.
pub mod radius {
    /// The corner radius of buttons, tabs, segments and square rail controls.
    pub const CONTROL: f32 = MD;
    /// Tiny non-interactive palette preview tiles.
    pub const PREVIEW: f32 = 3.;
    /// Existing appearance preview selection outline.
    pub const APPEARANCE: f32 = 10.;
    /// Native settings window outline (not a control).
    pub const WINDOW: f32 = 14.;
    /// Small controls and swatches.
    pub const SM: f32 = 4.;
    /// Standard controls.
    pub const MD: f32 = 6.;
    /// Panels.
    pub const LG: f32 = 8.;
    /// Grouped cards and dialogs.
    pub const XL: f32 = 12.;

    /// Radius sharing a center with an outer corner. `inset` includes the
    /// container's border and padding; deep insets become square corners.
    pub fn concentric(outer: f32, inset: f32) -> f32 {
        (outer - inset).max(0.)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::tokens::{metrics, space};

        #[test]
        fn inset_corners_share_centers_and_never_go_negative() {
            assert_eq!(concentric(XL, metrics::EDGE + space::SM), 5.);
            assert_eq!(concentric(LG, metrics::EDGE + space::XS), 3.);
            assert_eq!(concentric(XL, metrics::EDGE), 11.);
            assert_eq!(concentric(XL, XL), 0.);
            assert_eq!(concentric(XL, XL + space::SM), 0.);
        }
    }
}
/// Shared font families and compatibility typography aliases.
pub mod type_size {
    /// ASCII skeleton paragraph line height, matching upstream's 1.3em.
    pub const ASCII_LINE_EM: f32 = 1.3;
    /// Dither skeleton glyph tracking, matching upstream's -.02em.
    pub const ASCII_DITHER_TRACKING_EM: f32 = -0.02;
    /// Paragraph skeleton row gap, matching upstream's .55em.
    pub const ASCII_GAP_EM: f32 = 0.55;
    /// Axis badge inside a rail button.
    pub const BADGE: f32 = 9.;
    /// Document title in the toolbar.
    pub const DOCUMENT: f32 = 15.;
    /// Settings window title.
    pub const WINDOW_TITLE: f32 = 16.;
    /// Confirmation and help headings.
    pub const DIALOG_TITLE: f32 = 18.;
    /// Dense explanatory copy line height.
    pub const LINE_DENSE: f32 = 18.;
    /// Comment message line height.
    pub const LINE_COMMENT: f32 = 21.;
    /// Long-form help line height.
    pub const LINE_PROSE: f32 = 26.;
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
/// Compatibility 24/32/40 control sizes; new controls use `ControlHeight`.
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
/// Shared application geometry, independent of control presentation.
pub mod metrics {
    /// Diagnostics window width, accommodating explicit consent actions.
    pub const REPORT_WIDTH: f32 = 880.0;
    /// Diagnostics window height; report previews scroll within this surface.
    pub const REPORT_HEIGHT: f32 = 760.0;
    /// Split Welcome window and its native editor launch geometry.
    pub mod welcome {
        /// Default Welcome window width.
        pub const WIDTH: f32 = 760.;
        /// Default Welcome window height.
        pub const HEIGHT: f32 = 620.;
        /// Width of the Welcome action pane.
        pub const SIDEBAR: f32 = 244.;
        /// Clearance for native window chrome above the brand.
        pub const SIDEBAR_TOP: f32 = 76.;
        /// Welcome brand artwork side.
        pub const MARK: f32 = 56.;
        /// Welcome product name line height.
        pub const NAME_LINE: f32 = 29.;
        /// Welcome content top inset.
        pub const MAIN_TOP: f32 = 22.;
        /// Welcome sidebar action height.
        pub const ACTION: f32 = 34.;
        /// Recent project row height.
        pub const ROW: f32 = 60.;
        /// Recent project artwork width.
        pub const THUMB_WIDTH: f32 = 60.;
        /// Recent project artwork height.
        pub const THUMB_HEIGHT: f32 = 46.;
        /// Artwork height in the empty-history grid.
        pub const SAMPLE_LARGE: f32 = 196.;
        /// Artwork height in the compact bottom sample strip.
        pub const SAMPLE_SMALL: f32 = 63.;
        /// Initial native editor width.
        pub const EDITOR_WIDTH: f32 = 1440.;
        /// Initial native editor height.
        pub const EDITOR_HEIGHT: f32 = 900.;
        /// Minimum native editor width.
        pub const EDITOR_MIN_WIDTH: f32 = 1024.;
        /// Editor native traffic-light horizontal inset.
        pub const EDITOR_TRAFFIC_LEFT: f32 = 20.;

        /// Glyph side inside Welcome shortcut keycaps.
        pub const KEY_GLYPH: f32 = 8.;
    }

    /// Small glyph inside a keycap or checkbox.
    pub const ICON_TINY: f32 = 10.;
    /// Compact checkbox side.
    pub const ICON_DISCLOSURE: f32 = 14.;
    /// Welcome action glyph.
    pub const ICON_CHECK: f32 = 16.;

    /// Existing recent-project preview image edge.
    pub const RECENT_PREVIEW: f32 = 64.;
    /// Reserved recent-project preview column, including its horizontal gutter.
    pub const RECENT_PREVIEW_COLUMN: f32 = 80.;
    /// Compact centered placeholder while the first viewport frame is rendering.
    pub const SCENE_LOADING_WIDTH: f32 = 240.;
    /// Preserve the mirror tooltip anchor: four pixels of width change plus the new border.
    pub const MIRROR_HINT_X: f32 = 5.;
    /// First mirror tooltip's vertical correction after its button grows from 34 to 36.
    pub const MIRROR_HINT_Y: f32 = -1.;
    /// Additional vertical anchor correction per mirror row, preserving tooltip placement.
    pub const MIRROR_HINT_ROW: f32 = -2.;
    /// Native application menu bar height; macOS uses the system menu.
    pub const MENUBAR_HEIGHT: f32 = if cfg!(target_os = "macos") { 0. } else { 32. };
    /// Square tool and mirror rail button side.
    pub const RAIL_BUTTON: f32 = 36.;
    /// Editor tool rail width.
    pub const RAIL_WIDTH: f32 = 56.;
    /// Editor sidebar width.
    pub const SIDEBAR_WIDTH: f32 = 352.;
    /// Editor sidebar horizontal gutter.
    pub const PANEL_GUTTER: f32 = 24.;
    /// Discussion and history panel content inset.
    pub const PANEL_PADDING: f32 = 20.;
    /// Floating panel content inset.
    pub const POPOVER_PADDING: f32 = 6.;
    /// Compact menu inset.
    pub const MENU_PADDING: f32 = 4.;
    /// Compact menu width.
    pub const MENU_WIDTH: f32 = 224.;
    /// Editor application menu width.
    pub const APP_MENU_WIDTH: f32 = 310.;
    /// Sidebar tab height.
    pub const TAB_HEIGHT: f32 = 32.;
    /// Selected tab underline thickness.
    pub const TAB_UNDERLINE: f32 = 2.;
    /// Selected tab underline horizontal inset.
    pub const TAB_UNDERLINE_INSET: f32 = 2.;
    /// Dense search and layer action glyph.
    pub const ICON_DENSE: f32 = 15.;
    /// Inline shortcut keycap side.
    pub const KEYCAP: f32 = 22.;
    /// Settings shortcut well minimum width.
    pub const SHORTCUT_WIDTH: f32 = 96.;
    /// Settings shortcut well height.
    pub const SHORTCUT_HEIGHT: f32 = 30.;
    /// Save confirmation action minimum width.
    pub const ACTION_MIN_WIDTH: f32 = 80.;
    /// Command shortcut or unavailable-reason maximum width.
    pub const ACTION_HINT_WIDTH: f32 = 240.;
    /// Command palette height.
    pub const COMMAND_HEIGHT: f32 = 440.;
    /// Command palette search row height.
    pub const COMMAND_SEARCH_HEIGHT: f32 = 52.;
    /// Save confirmation and acknowledgements width.
    pub const CONFIRM_WIDTH: f32 = 480.;
    /// Keyboard guide width.
    pub const GUIDE_WIDTH: f32 = 720.;
    /// Compact palette chooser width.
    pub const PALETTE_CHOOSER_WIDTH: f32 = 328.;
    /// Palette choice list maximum height.
    pub const PALETTE_CHOOSER_HEIGHT: f32 = 360.;
    /// Palette manager navigation width.
    pub const PALETTE_SIDEBAR_WIDTH: f32 = 236.;
    /// Palette manager swatch grid height.
    pub const PALETTE_GRID_HEIGHT: f32 = 118.;
    /// Color square and hue slider column width.
    pub const COLOR_EDITOR_WIDTH: f32 = 216.;
    /// Palette manager dialog width.
    pub const PALETTE_DIALOG_WIDTH: f32 = 860.;
    /// Palette manager dialog height.
    pub const PALETTE_DIALOG_HEIGHT: f32 = 780.;
    /// Palette preview thumbnail width.
    pub const THUMBNAIL_WIDTH: f32 = 44.;
    /// Two rows of sidebar swatches.
    pub const PALETTE_SCROLL_STEP: f32 = SWATCH_PITCH * 2.;
    /// Sidebar swatch size plus gap.
    pub const SWATCH_PITCH: f32 = crate::ControlHeight::Default.px() + crate::Space::S3.px();
    /// Settings window initial width.
    pub const SETTINGS_WIDTH: f32 = 880.;
    /// Settings window initial height.
    pub const SETTINGS_HEIGHT: f32 = 640.;
    /// Settings minimum window width.
    pub const SETTINGS_MIN_WIDTH: f32 = 760.;
    /// Settings minimum window height.
    pub const SETTINGS_MIN_HEIGHT: f32 = 520.;
    /// Settings titlebar height.
    pub const SETTINGS_TITLEBAR: f32 = 62.;
    /// macOS settings titlebar leading inset.
    pub const SETTINGS_TRAFFIC_INSET: f32 = 92.;
    /// macOS traffic light vertical position.
    pub const TRAFFIC_TOP: f32 = 23.;
    /// macOS editor titlebar leading inset.
    pub const TOOLBAR_TRAFFIC_INSET: f32 = 104.;
    /// Appearance preview height.
    pub const APPEARANCE_PREVIEW_HEIGHT: f32 = 90.;
    /// Appearance preview body height.
    pub const APPEARANCE_BODY_HEIGHT: f32 = 70.;
    /// Native agent connection popover width.
    pub const AGENT_POPOVER_WIDTH: f32 = 460.;
    /// Agent popover top anchor.
    pub const AGENT_POPOVER_TOP: f32 = 100.;
    /// Recent history height below agent invitation.
    pub const AGENT_HISTORY_HEIGHT: f32 = 156.;
    /// Agent feed minimum height.
    pub const AGENT_FEED_MIN_HEIGHT: f32 = 310.;
    /// Agent event scroll minimum height.
    pub const AGENT_EVENT_MIN_HEIGHT: f32 = 64.;
    /// Captured view card height.
    pub const CAPTURE_HEIGHT: f32 = 100.;
    /// Captured view image height.
    pub const CAPTURE_IMAGE_HEIGHT: f32 = 76.;
    /// Comment list panel minimum height.
    pub const COMMENTS_MIN_HEIGHT: f32 = 120.;
    /// Thread body minimum height.
    pub const THREAD_MIN_HEIGHT: f32 = 60.;
    /// History step list minimum height.
    pub const HISTORY_LIST_MIN_HEIGHT: f32 = 70.;
    /// Named versions list maximum height.
    pub const VERSIONS_MAX_HEIGHT: f32 = 110.;
    /// History scrubber hit area height.
    pub const TIMELINE_HEIGHT: f32 = 26.;
    /// Existing compact playback action width.
    pub const PLAYBACK_WIDTH: f32 = 34.;
    /// Compact layers panel minimum height.
    pub const LAYERS_COMPACT_MIN_HEIGHT: f32 = 180.;
    /// Full layer list maximum height.
    pub const LAYERS_SCROLL_HEIGHT: f32 = 192.;
    /// Compact layer list maximum height.
    pub const LAYERS_COMPACT_SCROLL_HEIGHT: f32 = 128.;
    /// Recent history panel minimum height.
    pub const HISTORY_MIN_HEIGHT: f32 = 204.;
    /// Full document title width.
    pub const DOCUMENT_WIDTH: f32 = 340.;
    /// Compact document title width.
    pub const DOCUMENT_COMPACT_WIDTH: f32 = 260.;
    /// Palette chooser top anchor.
    pub const PALETTE_CHOOSER_TOP: f32 = 112.;

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
