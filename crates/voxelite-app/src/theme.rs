//! Explicit Concept A theme tokens.
use gpui::{Hsla, rgb};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}
impl Theme {
    pub fn background(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x000000
        } else {
            0xffffff
        })
        .into()
    }
    pub fn text(self) -> Hsla {
        rgb(if self == Self::Dark {
            0xfafafa
        } else {
            0x171717
        })
        .into()
    }
    pub fn muted(self) -> Hsla {
        rgb(if self == Self::Dark {
            0xa3a3a3
        } else {
            0x525252
        })
        .into()
    }
    pub fn line(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x232323
        } else {
            0xe5e5e5
        })
        .into()
    }
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
    pub fn accent(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x93c5fd
        } else {
            0x1d4ed8
        })
        .into()
    }
    pub fn selected(self) -> Hsla {
        rgb(if self == Self::Dark {
            0x142033
        } else {
            0xeff5ff
        })
        .into()
    }
    pub fn danger(self) -> Hsla {
        rgb(if self == Self::Dark {
            0xfca5a5
        } else {
            0xb91c1c
        })
        .into()
    }
    pub fn disabled(self) -> Hsla {
        rgb(0x737373).into()
    }
    pub fn toggle(&mut self) {
        *self = if *self == Self::Dark {
            Self::Light
        } else {
            Self::Dark
        };
    }
}
