use std::str::FromStr;

use ratatui::style::Color;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct ColourTheme {
    #[serde_as(as = "DisplayFromStr")]
    pub keyword: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub ident: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub lit: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub delim: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub types: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub extra: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub background: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub function: Colour,

    #[serde_as(as = "DisplayFromStr")]
    pub comment: Colour,
}

#[derive(Debug, Copy, Clone)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl From<Colour> for Color {
    fn from(val: Colour) -> Self {
        Color::Rgb(val.r, val.g, val.b)
    }
}

impl FromStr for Colour {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('#') {
            let r: u8 = u8::from_str_radix(&s[1..=2], 16)?;
            let g: u8 = u8::from_str_radix(&s[3..=4], 16)?;
            let b: u8 = u8::from_str_radix(&s[5..=6], 16)?;
            Ok(Colour { r, g, b })
        } else {
            let r: u8 = u8::from_str_radix(&s[0..=1], 16)?;
            let g: u8 = u8::from_str_radix(&s[2..=3], 16)?;
            let b: u8 = u8::from_str_radix(&s[4..=5], 16)?;
            Ok(Colour { r, g, b })
        }
    }
}
