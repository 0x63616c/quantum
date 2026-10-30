use crate::from_hex;
use voxelite_plugin_api::Palette;

/// Exact built-in palettes. Originals are produced by value; library copies are editable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    /// Lexaloffle's standard sixteen colors.
    Pico8,
    /// ENDESGA's thirty-two colors in published order.
    Endesga32,
    /// GrafxKid's Sweetie 16.
    Sweetie16,
    /// Sixteen evenly spaced sRGB grays, black through white.
    Grayscale,
    /// Original Voxelite stone and slate tones.
    Neutral,
    /// Original Voxelite earth and terracotta tones.
    Warm,
    /// Original Voxelite saturated garden colors.
    Vivid,
}
impl Preset {
    /// Construct exact opaque source colors without assigning slot zero.
    pub fn palette(self) -> Palette {
        let hex = match self {
            Self::Pico8 => {
                "000000 1d2b53 7e2553 008751 ab5236 5f574f c2c3c7 fff1e8 ff004d ffa300 ffec27 00e436 29adff 83769c ff77a8 ffccaa"
            }
            Self::Endesga32 => {
                "be4a2f d77643 ead4aa e4a672 b86f50 733e39 3e2731 a22633 e43b44 f77622 feae34 fee761 63c74d 3e8948 265c42 193c3e 124e89 0099db 2ce8f5 ffffff c0cbdc 8b9bb4 5a6988 3a4466 262b44 181425 ff0044 68386c b55088 f6757a e8b796 c28569"
            }
            Self::Sweetie16 => {
                "1a1c2c 5d275d b13e53 ef7d57 ffcd75 a7f070 38b764 257179 29366f 3b5dc9 41a6f6 73eff7 f4f4f4 94b0c2 566c86 333c57"
            }
            Self::Grayscale => {
                "000000 111111 222222 333333 444444 555555 666666 777777 888888 999999 aaaaaa bbbbbb cccccc dddddd eeeeee ffffff"
            }
            Self::Neutral => "202833 35404c 4f5d69 76838b a2adae c9ceca e8e6da f8f4e7",
            Self::Warm => "352933 593d3b 815344 ab6c4b d78d5b eab878 f1d5a2 f9ebcc",
            Self::Vivid => "263747 326d66 55a36b a5c76b ebd27e e99964 c66264 815477",
        };
        from_hex(&hex.replace(' ', "\n")).expect("validated built-in palette")
    }
}
