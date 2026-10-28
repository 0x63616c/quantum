use super::*;

#[test]
fn palettes_exact_and_interchange() {
    for preset in [
        Preset::Pico8,
        Preset::Endesga32,
        Preset::Sweetie16,
        Preset::Grayscale,
        Preset::Neutral,
        Preset::Warm,
        Preset::Vivid,
    ] {
        let p = preset.palette();
        assert_eq!(from_hex(&p.to_hex()).unwrap(), p);
        assert_eq!(from_gpl(&p.to_gpl("Test").unwrap()).unwrap(), p);
    }
    assert_eq!(Preset::Endesga32.palette().slots.len(), 32);
    assert_eq!(
        Preset::Pico8.palette().get(13).unwrap().rgba,
        [41, 173, 255, 255]
    );
    assert_eq!(
        Preset::Sweetie16.palette().get(16).unwrap().rgba,
        [51, 60, 87, 255]
    );
    let p=from_gpl("GIMP Palette\nName: Custom\nColumns: 2\n#\n  1   2   3\tMoss green\n  1   2   3\tDuplicate\n").unwrap();
    assert_eq!(p.slots.len(), 2);
    assert_eq!(from_gpl(&p.to_gpl("Custom").unwrap()).unwrap(), p);
    for bad in ["#fff", "xxxxxx", "11223344", ""] {
        assert!(from_hex(bad).is_err());
    }
}

#[test]
fn jasc_pal_round_trips_duplicate_colors_and_exports_display_order() {
    use quantum_plugin_api::{Palette, Swatch};
    for preset in [
        Preset::Pico8,
        Preset::Endesga32,
        Preset::Sweetie16,
        Preset::Grayscale,
    ] {
        let palette = preset.palette();
        assert_eq!(from_pal(&to_pal(&palette).unwrap()).unwrap(), palette);
    }
    let mut palette = Palette::from_colors(vec![
        Swatch::rgb(1, 2, 3),
        Swatch::rgb(4, 5, 6),
        Swatch::rgb(1, 2, 3),
    ])
    .unwrap();
    palette.order = vec![2, 1, 3];
    palette.slots[0].color.name = Some("Named".into());
    let text = export("pal", &palette, "Test").unwrap();
    assert_eq!(text, "JASC-PAL\n0100\n3\n4 5 6\n1 2 3\n1 2 3\n");
    assert_eq!(import("pal", &text).unwrap().to_hex(), palette.to_hex());
    assert_eq!(
        from_pal("JASC-PAL\r\n0100\r\n2\r\n1 2 3\r\n\r\n1 2 3 255\r\n")
            .unwrap()
            .to_hex(),
        "010203\n010203\n"
    );
}

#[test]
fn jasc_pal_rejects_malformed_headers_counts_channels_and_transparency() {
    for text in [
        "",
        "RIFF",
        "JASC-PAL\n0200\n1\n1 2 3",
        "JASC-PAL\n0100\n0",
        "JASC-PAL\n0100\n257",
        "JASC-PAL\n0100\n184467440737095516160",
        "JASC-PAL\n0100\n1",
        "JASC-PAL\n0100\n2\n1 2 3",
        "JASC-PAL\n0100\n1\n1 2 3\n4 5 6",
        "JASC-PAL\n0100\n1\n256 0 0",
        "JASC-PAL\n0100\n1\n-1 2 3",
        "JASC-PAL\n0100\n1\n1 2",
        "JASC-PAL\n0100\n1\n1 2 3 name",
        "JASC-PAL\n0100\n1\n1 2 3 128",
        "JASC-PAL\n0100\n1\n1 2 3 255 0",
        "JASC-PAL\n0100\n1\n1 2 3\0",
    ] {
        assert!(from_pal(text).is_err(), "{text:?}");
    }
    assert!(from_pal(&format!("JASC-PAL\n0100\n1\n{}1 2 3", " ".repeat(65536))).is_err());
    let full = format!("JASC-PAL\n0100\n256\n{}", "1 2 3\n".repeat(256));
    assert_eq!(from_pal(&full).unwrap().slots.len(), 256);
}

#[test]
fn all_formats_are_bounded_and_gpl_retains_names() {
    use quantum_plugin_api::{Palette, PaletteSlot, Swatch};
    for format in FORMATS {
        assert_eq!(super::format(format.id).unwrap(), *format);
        assert!(import(format.id, &" ".repeat(65537)).is_err());
    }
    assert!(import("unknown", "112233").is_err());
    assert!(export("unknown", &Palette::default(), "Test").is_err());
    assert!(from_hex(&"112233\n".repeat(257)).is_err());
    assert!(from_gpl(&format!("GIMP Palette\n{}", "1 2 3\n".repeat(257))).is_err());
    assert!(from_gpl(&format!("GIMP Palette\n1 2 3 {}", "n".repeat(129))).is_err());
    let gpl = "GIMP Palette\r\nName: Imported\r\nColumns: 16\r\n# note\r\n1 2 3\tForest moss\r\n1 2 3\tDuplicate\r\n";
    let palette = import("gpl", gpl).unwrap();
    assert_eq!(palette.get(1).unwrap().name.as_deref(), Some("Forest moss"));
    assert_eq!(
        import("gpl", &export("gpl", &palette, "Test").unwrap()).unwrap(),
        palette
    );
    let invalid = Palette {
        slots: vec![PaletteSlot {
            index: 1,
            color: Swatch::rgb(0, 0, 0),
        }],
        order: vec![2],
    };
    for format in FORMATS {
        assert!(export(format.id, &invalid, "Test").is_err());
    }
}
