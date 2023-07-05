use std::fmt::{Display, Write};

pub mod serenity;

#[derive(Debug, derive_builder::Builder)]
pub struct ProgressBar {
    value: i64,
    max: i64,

    #[builder(default = "10")]
    length: i64,

    #[builder(default = "'░'")]
    empty_symbol: char,

    #[builder(default = "'█'")]
    full_symbol: char,

    #[builder(default = "false")]
    with_count: bool,

    #[builder(default = "false")]
    with_percentage: bool,
}

impl ProgressBar {
    pub fn builder() -> ProgressBarBuilder {
        ProgressBarBuilder::default()
    }
}

impl Display for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = (self.value as f32) / (self.max as f32);
        let percent = value * 100.0;
        let nblocks = (value * self.length as f32) as _;

        for _ in 0..nblocks {
            f.write_char(self.full_symbol)?;
        }

        for _ in nblocks..self.length {
            f.write_char(self.empty_symbol)?;
        }

        match (self.with_count, self.with_percentage) {
            (false, false) => Ok(()),
            (true, false) => write!(f, " [{}/{}]", self.value, self.max),
            (false, true) => write!(f, " [{:.0}%]", percent),
            (true, true) => write!(f, " [{}/{} · {:.0}%]", self.value, self.max, percent),
        }
    }
}

pub mod colors {
    #![allow(dead_code)]

    use serenity::utils::Color;

    // https://discord.com/branding
    pub static DISCORD_BLURPLE: Color = Color::new(0x5865F2);
    pub static DISCORD_GREEN: Color = Color::new(0x57F287);
    pub static DISCORD_YELLOW: Color = Color::new(0xFEE75C);
    pub static DISCORD_FUCHSIA: Color = Color::new(0xEB459E);
    pub static DISCORD_RED: Color = Color::new(0xED4245);
    pub static DISCORD_WHITE: Color = Color::new(0xFFFFFF);
    pub static DISCORD_BLACK: Color = Color::new(0x000000);
}

pub mod emojis {
    #![allow(dead_code)]

    // https://emojipedia.org/
    pub static CHECK_MARK: &str = "\u{2714}\u{FE0F}";
    pub static CROSS_MARK: &str = "\u{274C}";
    pub static LARGE_GREEN_CIRCLE: &str = "\u{1F7E2}";
    pub static LARGE_RED_CIRCLE: &str = "\u{1F534}";
    pub static LARGE_YELLOW_CIRCLE: &str = "\u{1F7E1}";
}
