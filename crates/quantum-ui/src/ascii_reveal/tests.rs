// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/quantum-ui/NOTICE

// Goldens evaluated with Node 20 from upstream dist/core.js at f602e20.
// The unmodified textTarget/mediaTarget functions were run with mock layout and
// canvas contexts collecting fillText calls; hash and EASE were exposed from
// the compiled bundle. No Rust implementation was used to generate these values.
use super::{core::*, media::*, presets::*, skeleton::*};

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

#[test]
fn javascript_hash_goldens_include_signed_overflow() {
    for ((a, b, c), bits, value) in [
        ((0, 0, 0), 2_102_831_971, 0.4896037213038653),
        ((1, 2, 3), 2_498_012_504, 0.5816138591617346),
        ((-1, 0, 17), 3_084_957_973, 0.718272750498727),
        ((i32::MAX, i32::MIN, -1), 728_285_183, 0.16956710792146623),
        ((123456, 987654, 29), 4_272_964_558, 0.9948770883493125),
    ] {
        assert_eq!(hash_bits(a, b, c), bits);
        assert_eq!(hash(a, b, c), value);
    }
}

#[test]
fn javascript_float32_bezier_lut_goldens() {
    for (p, expected) in [
        (0., 5.587935669737476e-10),
        (1. / 256., 0.00045641150791198015),
        (0.1, 0.027416251599788666),
        (0.25, 0.13458184897899628),
        (0.5, 0.5),
        (0.75, 0.8654181361198425),
        (0.9, 0.9725837707519531),
        (255. / 256., 0.9995436072349548),
        (1., 1.),
    ] {
        assert_eq!(sweep_ease(p), expected);
    }
    assert_eq!(sweep_ease(0.5 / 256.), f64::from(SWEEP_LUT[1]));
    assert_eq!(ease(0.25), 0.578125);
    assert_eq!(ease_io(0.25), 0.0625);
    assert_eq!(ease_io(0.75), 0.9375);
    assert_eq!(ease(-1.), 0.);
    assert_eq!(ease_io(2.), 1.);
    assert!(clamp(f64::NAN, 0., 1.).is_nan());
}

#[test]
fn javascript_skeleton_base_and_wave_goldens() {
    for (variant, seed, expected) in [
        (
            SkeletonVariant::Dither,
            1,
            "░▒░░░░▒░░░░░▒▒░░░░░░░░░░\n▒░░▒░░░░▒░▒░░▒░▒░▒░░░░░░",
        ),
        (
            SkeletonVariant::Scan,
            97,
            "········-:······:·:·-:  \n······: ·::·:: ·:· -:::-",
        ),
        (
            SkeletonVariant::Braille,
            -7,
            " ⠒⠔⠑⠊⠕⠔⠊⠪⠡⠌⠊⠢⠢⠕⠡⠌⠡⠊⠒⠡⠪⠤⠌\n⠔⠌⠊⠡⠪⠢⠕⠤⠡⠕⠕⠔⠑⠡⠌⠊⠢⠒⠕⠔⠒⠔⠒⠌",
        ),
        (
            SkeletonVariant::Terminal,
            123,
            "0001  010011101101110011\n101010 11011000111001111",
        ),
    ] {
        assert_eq!(skeleton_base(variant, seed, 2, 24), expected);
    }
    assert_eq!(
        glyph_row(1, 32, SkeletonVariant::Scan.spec().base, 0.08, 1.6),
        "·:····-·· ·:::······:······: ·:-"
    );
    let row = SkeletonRow::new(SkeletonVariant::Dither, 1, 0, 0, 24);
    assert_eq!(
        row.wave,
        ["▒▓▓▒█▓▓▓█▒▓▓▓▓██▒▓▓█▒▒▒▒", "▒▓▓▓█▒▓▒█▒▒▓▓▒▒█▓▒▓▒▒▓▓▒"]
    );
    assert_eq!(
        SkeletonRow::new(SkeletonVariant::Scan, 7, 2, 0, 8).base,
        glyph_row(7 * 97 + 2 * 13, 8, "··:·:-", 0.08, 1.6)
    );
}

fn rendered(plan: &Decode, t: f64, direction: Direction) -> String {
    plan.frame(t, direction)
        .map(|cell| {
            if cell.kind == CellKind::Pending {
                "_"
            } else {
                cell.glyph
            }
        })
        .collect()
}

#[test]
fn javascript_decode_frame_goldens() {
    for (preset, frames) in [
        (
            Preset::Decode,
            [
                "_______ __",
                "....___ __",
                "-=+=:._ __",
                "%#+%#*+ =+",
                "Q2ant7U 1=",
                "Quantum 42",
            ],
        ),
        (
            Preset::Dither,
            [
                "_______ __",
                "░░░░___ __",
                "▒▒▓▒░░_ __",
                "█████▓▓ ░▒",
                "Q▒ant░▓ █▒",
                "Quantum 42",
            ],
        ),
        (
            Preset::Terminal,
            [
                "_______ __",
                "████___ __",
                "<};;██_ __",
                "-|{<|;] +}",
                "Q-ant][ -]",
                "Quantum 42",
            ],
        ),
    ] {
        let plan = Decode::new(
            "Quantum 42",
            DecodeOptions {
                preset,
                seed: 7,
                ..Default::default()
            },
        );
        assert_eq!(plan.duration_ms(), 196.56947326660156);
        for (t, expected) in [0., 20., 50., 100., 160., 220.].into_iter().zip(frames) {
            assert_eq!(
                rendered(&plan, t, Direction::Reveal),
                expected,
                "{preset:?} at {t}"
            );
        }
        let early: Vec<_> = plan.frame(20., Direction::Reveal).take(4).collect();
        for cell in early {
            assert_eq!(
                cell.kind,
                if preset == Preset::Terminal {
                    CellKind::Cursor
                } else {
                    CellKind::Scramble
                }
            );
            assert_eq!(
                cell.alpha,
                if preset == Preset::Terminal { 1. } else { 0.5 }
            );
        }
        let kinds: String = plan
            .frame(160., Direction::Reveal)
            .map(|cell| match cell.kind {
                CellKind::Landed => 'L',
                CellKind::Scramble => 'S',
                _ => '?',
            })
            .collect();
        assert_eq!(kinds, "LSLLLSSLSS");
    }
}

#[test]
fn javascript_numeric_and_unicode_goldens() {
    let plan = Decode::new(
        "12,480",
        DecodeOptions {
            seed: 7,
            ..Default::default()
        },
    );
    assert_eq!(plan.duration_ms(), 201.12242126464844);
    for (t, expected) in [
        (0., "______"),
        (20., "398___"),
        (50., "398216"),
        (100., "149397"),
        (160., "16,417"),
        (220., "12,480"),
    ] {
        assert_eq!(rendered(&plan, t, Direction::Reveal), expected);
    }
    let plan = Decode::new(
        "Aé 👩‍💻9",
        DecodeOptions {
            preset: Preset::Blueprint,
            timing: Timing::Cinematic,
            seed: -9,
            ..Default::default()
        },
    );
    assert_eq!(plan.duration_ms(), 475.7097473144531);
    for (t, expected) in [
        (0., "__ __"),
        (50., "+_ __"),
        (100., "·/ /·"),
        (160., "·/ +·"),
        (220., "\\/ ++"),
        (300., "|- +/"),
        (500., "Aé 👩‍💻9"),
    ] {
        assert_eq!(rendered(&plan, t, Direction::Reveal), expected);
    }
    assert_eq!(plan.frame(500., Direction::Reveal).count(), 5);
    assert!(
        plan.frame(100., Direction::Reveal)
            .filter(|c| c.kind == CellKind::Scramble)
            .all(|c| c.alpha == 0.9)
    );
}

#[test]
fn timing_reverse_finish_and_flash_contracts() {
    assert_eq!(Timing::Fast.spread(9), 76.);
    assert_eq!(Timing::Fast.spread(1000), 200.);
    assert_eq!(Timing::Cinematic.spread(9), 207.);
    assert_eq!(Timing::Cinematic.spread(1000), 720.);
    assert_eq!(Timing::Fast.wave_delay(320., 1., false), 100.);
    assert_eq!(Timing::Fast.wave_delay(10000., 2., true), 140.);
    assert_eq!(Timing::Cinematic.wave_delay(11500., 1., false), 10000.);
    let options = DecodeOptions {
        seed: 7,
        flash: true,
        ..Default::default()
    };
    let plan = Decode::new("Quantum 42", options);
    assert!(!plan.is_finished(196.));
    assert!(plan.is_finished(196.56947326660156));
    for t in [0., 20., 50., 100., 160.] {
        assert_eq!(
            rendered(&plan, plan.duration_ms() - t, Direction::Hide),
            rendered(&plan, t, Direction::Reveal)
        );
    }
    assert_eq!(rendered(&plan, 1000., Direction::Hide), "_______ __");
    assert_eq!(
        plan.frame(0., Direction::Hide)
            .filter(|c| c.kind == CellKind::Landed)
            .count(),
        10
    );
    assert!(plan.frame(160., Direction::Reveal).any(|c| c.flash > 0.));
    assert!(plan.frame(1000., Direction::Reveal).all(|c| c.flash == 0.));
    assert!(plan.frame(10., Direction::Hide).all(|c| c.flash == 0.));
    let fast = Decode::new(
        "Quantum 42",
        DecodeOptions {
            speed: 2.,
            ..options
        },
    );
    close(fast.duration_ms(), plan.duration_ms() / 2.);
    assert!(Decode::new(" \n\t", options).is_finished(0.));
    assert!(Decode::new("", options).is_finished(0.));
}

#[test]
fn frames_are_stateless_and_keep_graphemes_and_whitespace() {
    let plan = Decode::new("e\u{301}\t 👩‍💻\n42\u{feff}", DecodeOptions::default());
    let first: Vec<_> = plan.frame(90., Direction::Reveal).collect();
    let _ = plan.frame(1000., Direction::Hide).collect::<Vec<_>>();
    assert_eq!(
        first,
        plan.frame(90., Direction::Reveal).collect::<Vec<_>>()
    );
    assert_eq!(rendered(&plan, 10000., Direction::Reveal), plan.text());
    for c in plan.frame(90., Direction::Reveal) {
        assert!(plan.text().is_char_boundary(c.source.start));
        assert!(plan.text().is_char_boundary(c.source.end));
    }
    let a = Decode::new("ab", DecodeOptions::default());
    let b = Decode::new("a b", DecodeOptions::default());
    assert_eq!(a.duration_ms(), b.duration_ms());
    assert_eq!(
        rendered(&a, 60., Direction::Reveal),
        rendered(&b, 60., Direction::Reveal).replace(' ', "")
    );
}

#[test]
fn skeleton_sweep_geometry_stagger_flicker_and_still_frames() {
    let sweep = Sweep::default();
    assert!(
        sweep
            .strips(100., 0., 0, true)
            .iter()
            .all(|s| s.width == 0.)
    );
    let center = sweep.strips(100., 900., 0, true);
    close(center[0].x, 27.5);
    close(center[0].width, 2.8125);
    close(center[0].alpha, 0.0625);
    close(center[7].alpha, 0.9);
    close(center[8].alpha, 0.9);
    close(center[15].alpha, 0.0625);
    assert_eq!(center, sweep.strips(100., 990., 1, true));
    assert_eq!(
        sweep.strips(100., 0., 1, true),
        sweep.strips(100., 1710., 0, true)
    );
    assert_eq!(flicker_frame(119.), 0);
    assert_eq!(flicker_frame(120.), 1);
    assert_eq!(flicker_frame(240.), 0);
    assert!(
        sweep
            .strips(100., 900., 0, false)
            .iter()
            .all(|s| s.width == 0. && s.alpha == 0.)
    );
    assert_eq!(
        skeleton_base(SkeletonVariant::Scan, 8, 3, 20),
        skeleton_base(SkeletonVariant::Scan, 8, 3, 20)
    );
}

#[test]
fn reduced_motion_is_real_text_fade_and_instant_hide() {
    assert_eq!(reduced_motion_alpha(0., Direction::Reveal), 0.);
    assert_eq!(reduced_motion_alpha(220., Direction::Reveal), 1.);
    // CSS cubic-bezier(0,0,.58,1), not the engine's cubic ease() function.
    assert!((reduced_motion_alpha(110., Direction::Reveal) - 0.6846431874).abs() < 1e-8);
    assert_eq!(reduced_motion_alpha(1000., Direction::Hide), 0.);
    assert_eq!(reduced_motion_alpha(0., Direction::Hide), 0.);
}

fn image(preset: Preset) -> Media {
    let pixels: Vec<[u8; 4]> = (0..24)
        .flat_map(|y| {
            (0..32).map(move |x| {
                [
                    x * 7,
                    y * 9,
                    (x + y) * 4,
                    if (x + y) % 5 == 0 { 0 } else { 255 },
                ]
            })
        })
        .collect();
    Media::new(
        32,
        24,
        &pixels,
        MediaOptions {
            preset,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn javascript_media_goldens_include_sampling_gates_colors_and_crossfade() {
    for (preset, counts, glyphs) in [
        (Preset::Decode, [0, 13, 136, 144, 144, 0], "@@%##"),
        (Preset::Dither, [0, 20, 144, 144, 144, 0], "████▓"),
        (Preset::Blueprint, [0, 11, 105, 144, 144, 0], "###xx"),
    ] {
        let media = image(preset);
        for (t, count) in [25., 100., 250., 350., 400., 450.].into_iter().zip(counts) {
            assert_eq!(
                media.frame(t, Direction::Reveal).cells().count(),
                count,
                "{preset:?} at {t}"
            );
        }
        let frame = media.frame(100., Direction::Reveal);
        assert_eq!(
            frame.cells().take(5).map(|c| c.glyph).collect::<String>(),
            glyphs
        );
        close(
            media.frame(350., Direction::Reveal).glyph_alpha,
            0.6243705564327962,
        );
        close(
            media.frame(400., Direction::Reveal).glyph_alpha,
            0.07804631955409957,
        );
        assert_eq!(media.frame(350., Direction::Reveal).image_alpha, 0.114);
        assert_eq!(media.frame(400., Direction::Reveal).image_alpha, 0.833);
        assert_eq!(media.frame(450., Direction::Reveal).image_alpha, 1.);
        assert_eq!(media.frame(0., Direction::Reveal).image_alpha, 0.);
        assert!(media.is_finished(450.));
    }
    let media = image(Preset::Dither);
    let frame = media.frame(100., Direction::Reveal);
    let first = frame.cells().next().unwrap();
    assert_eq!(first.x, 8.680000305175781);
    assert_eq!(first.y, 2.799999952316284);
    assert_eq!(first.color, Some([49, 0, 28]));
    assert_eq!(first.font_size, 6.);
    let reverse = media.frame(350., Direction::Hide);
    assert_eq!(
        frame.cells().collect::<Vec<_>>(),
        reverse.cells().collect::<Vec<_>>()
    );
    assert_eq!(
        frame.cells().collect::<Vec<_>>(),
        media
            .frame(100., Direction::Reveal)
            .cells()
            .collect::<Vec<_>>()
    );
}

#[test]
fn media_rejects_bad_inputs_and_transparency_uses_backdrop() {
    assert_eq!(
        Media::new(0, 1, &[], MediaOptions::default()).unwrap_err(),
        MediaError::Dimensions
    );
    assert_eq!(
        Media::new(1025, 1, &[], MediaOptions::default()).unwrap_err(),
        MediaError::Dimensions
    );
    assert_eq!(
        Media::new(2, 2, &[], MediaOptions::default()).unwrap_err(),
        MediaError::PixelCount
    );
    assert_eq!(
        Media::new(
            1,
            1,
            &[[0; 4]],
            MediaOptions {
                origin: Some([f64::NAN, 0.]),
                ..Default::default()
            }
        )
        .unwrap_err(),
        MediaError::Origin
    );
    for pixels in [vec![[255; 4]; 64], vec![[0; 4]; 64]] {
        let media = Media::new(8, 8, &pixels, MediaOptions::default()).unwrap();
        assert_eq!(media.frame(100., Direction::Reveal).cells().count(), 0);
    }
}
