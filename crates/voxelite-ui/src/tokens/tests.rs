use super::*;

#[test]
fn pin_every_palette_entry_in_both_themes() {
    let dark = [
        0x0b0b0c, 0x111113, 0x18181b, 0x1f1f23, 0x232326, 0x2e2e33, 0xededef, 0xa1a1aa, 0x8b8b94,
        0x2563eb, 0x93c5fd, 0xf87171, 0xffffff, 0x86efac,
    ];
    let light = [
        0xf5f5f7, 0xffffff, 0xf4f4f5, 0xffffff, 0xe4e4e7, 0xd4d4d8, 0x18181b, 0x52525b, 0x65656f,
        0x2563eb, 0x1d4ed8, 0xb91c1c, 0xffffff, 0x166534,
    ];
    for (theme, expected) in [(Theme::Dark, dark), (Theme::Light, light)] {
        assert_eq!(ColorToken::ALL.map(|c| theme.palette().rgb(c)), expected);
    }
}
fn luminance(rgb: u32) -> f64 {
    let c = [16, 8, 0].map(|shift| {
        let v = f64::from((rgb >> shift) & 255) / 255.;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    });
    c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722
}
fn contrast(a: u32, b: u32) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
#[test]
fn all_body_and_semantic_text_meet_wcag_aa_on_all_surfaces() {
    use ColorToken::*;
    for theme in [Theme::Dark, Theme::Light] {
        let p = theme.palette();
        for bg in [Canvas, Surface, Raised, Overlay] {
            for fg in [
                TextPrimary,
                TextSecondary,
                TextTertiary,
                AccentText,
                Danger,
                Success,
            ] {
                assert!(
                    contrast(p.rgb(fg), p.rgb(bg)) >= 4.5,
                    "{theme:?} {fg:?} on {bg:?}"
                );
            }
        }
        assert!(contrast(p.rgb(OnAccent), p.rgb(Accent)) >= 4.5);
        for overlay in [state::PRIMARY_HOVER, state::PRESSED] {
            let fill = [16, 8, 0].into_iter().fold(0, |rgb, shift| {
                let c = ((p.rgb(Accent) >> shift) & 255) as f32;
                rgb | ((c + (255. - c) * overlay).round() as u32) << shift
            });
            assert!(contrast(p.rgb(OnAccent), fill) >= 4.5);
        }
    }
}
#[test]
fn pin_complete_geometry_and_type_scales() {
    use Space::*;
    assert_eq!(
        [S0, S1, S2, S3, S4, S5, S6, S7, S8].map(Space::px),
        [0., 2., 4., 8., 12., 16., 24., 32., 48.]
    );
    use Radius::*;
    assert_eq!(
        [Xs, Sm, Md, Lg, Full].map(Radius::px),
        [4., 6., 10., 14., 9999.]
    );
    use ControlHeight::*;
    assert_eq!(
        [Dense, Default, Large].map(ControlHeight::px),
        [24., 28., 32.]
    );
    use TextSize::*;
    let sizes = [Caption, Small, Body, Title, Heading, Display, Mono];
    assert_eq!(sizes.map(TextSize::px), [11., 12., 13., 15., 20., 28., 12.]);
    assert_eq!(
        sizes.map(TextSize::line_height),
        [16., 16., 20., 20., 28., 34., 16.]
    );
    assert_eq!(
        sizes.map(|s| s.weight().0),
        [500., 400., 400., 600., 600., 600., 400.]
    );
    assert_eq!(
        [
            geometry::EDGE,
            geometry::ICON,
            geometry::RAIL_ICON,
            geometry::RAIL,
            geometry::MENU_WIDTH,
            geometry::DIALOG_WIDTH
        ],
        [1., 16., 20., 36., 224., 560.]
    );
}
#[test]
fn pin_interaction_contract() {
    assert_eq!(
        [
            state::HOVER,
            state::PRESSED,
            state::SELECTED,
            state::DISABLED,
            state::SCRIM,
            state::PRIMARY_HOVER
        ],
        [0.05, 0.08, 0.16, 0.4, 0.5, 0.06]
    );
    assert_eq!(
        [state::FOCUS_WIDTH, state::FOCUS_OFFSET, state::INDICATOR],
        [2., 1., 2.]
    );
    assert_eq!(state::TOOLTIP_DELAY.as_millis(), 500);
}
