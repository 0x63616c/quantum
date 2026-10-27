//! Opaque sRGB ↔ HSB arithmetic. HSB is HSV: brightness is the largest RGB channel.

/// A color in hue degrees and saturation/brightness fractions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hsb {
    /// Hue in 0..=360 degrees. Both endpoints represent red.
    pub h: f32,
    /// Saturation in 0..=1.
    pub s: f32,
    /// Brightness in 0..=1.
    pub b: f32,
}

fn finite_clamp(value: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., max)
    } else {
        0.
    }
}

impl Hsb {
    /// Clamp finite values to the editor's ranges; nonfinite values become zero.
    pub fn new(h: f32, s: f32, b: f32) -> Self {
        Self {
            h: finite_clamp(h, 360.),
            s: finite_clamp(s, 1.),
            b: finite_clamp(b, 1.),
        }
    }

    /// Convert exact sRGB bytes, retaining the last hue for achromatic colors.
    pub fn from_rgb(rgb: [u8; 3], previous_hue: f32) -> Self {
        let [r, g, b] = rgb.map(|v| f32::from(v) / 255.);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let hue = if delta == 0. {
            previous_hue
        } else if max == r {
            60. * ((g - b) / delta).rem_euclid(6.)
        } else if max == g {
            60. * ((b - r) / delta + 2.)
        } else {
            60. * ((r - g) / delta + 4.)
        };
        Self::new(hue, if max == 0. { 0. } else { delta / max }, max)
    }

    /// Convert to nearest sRGB bytes, rounding only at the output boundary.
    pub fn to_rgb(self) -> [u8; 3] {
        let Self { h, s, b } = Self::new(self.h, self.s, self.b);
        let c = b * s;
        let hue = h.rem_euclid(360.) / 60.;
        let x = c * (1. - (hue.rem_euclid(2.) - 1.).abs());
        let rgb = match hue as u8 {
            0 => [c, x, 0.],
            1 => [x, c, 0.],
            2 => [0., c, x],
            3 => [0., x, c],
            4 => [x, 0., c],
            _ => [c, 0., x],
        };
        let minimum = b - c;
        rgb.map(|v| ((v + minimum) * 255.).round().clamp(0., 255.) as u8)
    }

    /// Saturation increases to the right; brightness increases upwards.
    pub fn at_square(self, x: f32, y: f32) -> Self {
        Self::new(self.h, x, 1. - y)
    }
}

/// Canonical six-digit uppercase RGB, including the leading hash.
pub fn rgb_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// Parse exactly six ASCII hexadecimal digits with an optional leading hash.
pub fn parse_rgb_hex(text: &str) -> Option<[u8; 3]> {
    let text = text.trim().strip_prefix('#').unwrap_or(text.trim());
    if text.len() != 6 || !text.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some([
        u8::from_str_radix(&text[0..2], 16).ok()?,
        u8::from_str_radix(&text[2..4], 16).ok()?,
        u8::from_str_radix(&text[4..6], 16).ok()?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn primaries_secondaries_and_wrap() {
        for (h, rgb) in [
            (0., [255, 0, 0]),
            (60., [255, 255, 0]),
            (120., [0, 255, 0]),
            (180., [0, 255, 255]),
            (240., [0, 0, 255]),
            (300., [255, 0, 255]),
            (360., [255, 0, 0]),
        ] {
            assert_eq!(Hsb::new(h, 1., 1.).to_rgb(), rgb);
            assert_eq!(Hsb::from_rgb(rgb, 0.).h, h % 360.);
        }
    }
    #[test]
    fn every_grey_retains_hue_and_round_trips() {
        for byte in 0..=255 {
            for hue in [0., 17.5, 120., 299.9, 360.] {
                let hsb = Hsb::from_rgb([byte; 3], hue);
                assert_eq!(hsb.h, hue);
                assert_eq!(hsb.s, 0.);
                assert_eq!(hsb.to_rgb(), [byte; 3]);
            }
        }
    }
    #[test]
    fn sampled_cube_round_trips_exactly() {
        // Includes both ends, near-greys and the full byte range on each axis.
        for r in (0..=255).step_by(5) {
            for g in (0..=255).step_by(5) {
                for b in (0..=255).step_by(5) {
                    let rgb = [r, g, b];
                    assert_eq!(Hsb::from_rgb(rgb, 37.).to_rgb(), rgb);
                }
            }
        }
    }
    #[test]
    fn clamp_nonfinite_rounding_and_pointer_edges() {
        assert_eq!(Hsb::new(-1., 2., 0.5).to_rgb(), [128, 0, 0]);
        assert_eq!(Hsb::new(210., 0.5, 0.9).to_rgb(), [115, 172, 230]);
        assert_eq!(Hsb::new(f32::NAN, f32::INFINITY, -1.), Hsb::default());
        assert_eq!(Hsb::new(720., 1., 1.).to_rgb(), [255, 0, 0]);
        assert_eq!(
            Hsb::new(42., 0., 0.).at_square(-2., 2.),
            Hsb::new(42., 0., 0.)
        );
        assert_eq!(Hsb::default().at_square(2., -1.), Hsb::new(0., 1., 1.));
    }
    #[test]
    fn hex_is_bounded_ascii_and_canonical() {
        assert_eq!(parse_rgb_hex(" #aB012f "), Some([171, 1, 47]));
        assert_eq!(rgb_hex([171, 1, 47]), "#AB012F");
        for invalid in ["", "#fff", "#12345678", "12345g", "é1234", "##123456"] {
            assert_eq!(parse_rgb_hex(invalid), None);
        }
    }
}
