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
