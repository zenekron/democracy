pub use self::{discord_timestamp::*, progress_bar::*};

mod discord_timestamp;
mod progress_bar;
pub mod serenity;

pub mod colors {
    #![allow(dead_code)]

    use serenity::model::Color;

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
    pub static CHECK_MARK_BUTTON: &str = "✅";
    pub static CROSS_MARK: &str = "\u{274C}";
    pub static LARGE_GREEN_CIRCLE: &str = "\u{1F7E2}";
    pub static LARGE_RED_CIRCLE: &str = "\u{1F534}";
    pub static LARGE_YELLOW_CIRCLE: &str = "\u{1F7E1}";
    pub static NO_ENTRY: &str = "⛔";
    pub static PROHIBITED: &str = "🚫";
    pub static WARNING: &str = "⚠️";
}
