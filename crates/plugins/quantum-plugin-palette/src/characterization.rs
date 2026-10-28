use super::*;
use quantum_plugin_api::{Palette, PaletteLibrary, Swatch, Uuid};

#[test]
fn exact_presets_in_source_order() {
    for (preset, expected) in [
        (
            Preset::Pico8,
            "000000 1d2b53 7e2553 008751 ab5236 5f574f c2c3c7 fff1e8 ff004d ffa300 ffec27 00e436 29adff 83769c ff77a8 ffccaa",
        ),
        (
            Preset::Endesga32,
            "be4a2f d77643 ead4aa e4a672 b86f50 733e39 3e2731 a22633 e43b44 f77622 feae34 fee761 63c74d 3e8948 265c42 193c3e 124e89 0099db 2ce8f5 ffffff c0cbdc 8b9bb4 5a6988 3a4466 262b44 181425 ff0044 68386c b55088 f6757a e8b796 c28569",
        ),
        (
            Preset::Sweetie16,
            "1a1c2c 5d275d b13e53 ef7d57 ffcd75 a7f070 38b764 257179 29366f 3b5dc9 41a6f6 73eff7 f4f4f4 94b0c2 566c86 333c57",
        ),
        (
            Preset::Grayscale,
            "000000 111111 222222 333333 444444 555555 666666 777777 888888 999999 aaaaaa bbbbbb cccccc dddddd eeeeee ffffff",
        ),
        (
            Preset::Neutral,
            "202833 35404c 4f5d69 76838b a2adae c9ceca e8e6da f8f4e7",
        ),
        (
            Preset::Warm,
            "352933 593d3b 815344 ab6c4b d78d5b eab878 f1d5a2 f9ebcc",
        ),
        (
            Preset::Vivid,
            "263747 326d66 55a36b a5c76b ebd27e e99964 c66264 815477",
        ),
    ] {
        let palette = preset.palette();
        assert_eq!(palette.to_hex(), expected.replace(' ', "\n") + "\n");
        assert_eq!(
            palette.order,
            (1..=palette.slots.len() as u16).collect::<Vec<_>>()
        );
        assert!(
            palette
                .slots
                .iter()
                .all(|s| s.color.rgba[3] == 255 && s.color.name.is_none())
        );
    }
}

#[test]
fn swatch_selection_wraps_and_uses_first_rgba_match_in_display_order() {
    let mut palette = Palette::from_colors(vec![
        Swatch::rgb(1, 2, 3),
        Swatch::rgb(4, 5, 6),
        Swatch::rgb(1, 2, 3),
    ])
    .unwrap();
    palette.slots[2].color.name = Some("Duplicate".into());
    palette.order = vec![3, 2, 1];
    assert_eq!(
        step_swatch(&palette, &Swatch::rgb(1, 2, 3), 1),
        (1, Swatch::rgb(4, 5, 6))
    );
    assert_eq!(
        step_swatch(&palette, &Swatch::rgb(1, 2, 3), -1),
        (2, Swatch::rgb(1, 2, 3))
    );
    assert_eq!(
        step_swatch(&palette, &Swatch::rgb(99, 99, 99), 8),
        (2, Swatch::rgb(1, 2, 3))
    );
    assert_eq!(
        step_swatch(&Palette::default(), &Swatch::rgb(0, 0, 0), -8).0,
        0
    );
}

#[test]
fn library_add_remove_and_order_survive_restart_with_the_same_json() {
    let mut library = library::load(None).unwrap();
    library::create(&mut library, "Studio".into(), Palette::default()).unwrap();
    library.entries[0].id = Uuid::from_u128(1);
    library::set_colors(
        &mut library,
        0,
        vec![
            Swatch::rgb(1, 2, 3),
            Swatch::rgb(4, 5, 6),
            Swatch::rgb(1, 2, 3),
        ],
    )
    .unwrap();
    let mut colors: Vec<_> = library.entries[0]
        .palette
        .slots
        .iter()
        .map(|s| s.color.clone())
        .collect();
    colors.swap(0, 1);
    colors.remove(2);
    library::set_colors(&mut library, 0, colors).unwrap();
    let expected = r#"{
  "entries": [
    {
      "id": "00000000-0000-0000-0000-000000000001",
      "name": "Studio",
      "palette": {
        "slots": [
          {
            "index": 1,
            "rgba": [
              4,
              5,
              6,
              255
            ]
          },
          {
            "index": 2,
            "rgba": [
              1,
              2,
              3,
              255
            ]
          }
        ],
        "order": [
          1,
          2
        ]
      }
    }
  ]
}"#;
    let bytes = library::encode(&library).unwrap();
    assert_eq!(std::str::from_utf8(&bytes).unwrap(), expected);
    let loaded: PaletteLibrary = library::load(Some(&bytes)).unwrap();
    assert_eq!(loaded.entries[0].palette.to_hex(), "040506\n010203\n");
    assert_eq!(loaded.entries[0].id, Uuid::from_u128(1));
}
