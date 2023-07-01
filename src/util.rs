use std::fmt::{Display, Write};

const PROGRESSBAR_EMPTY_CHAR: char = '░';
const PROGRESSBAR_FULL_CHAR: char = '█';

pub struct ProgressBar(pub f32);

impl Display for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bar_length: u32 = 10;
        let nblocks = (self.0.min(1.0) * (bar_length as f32)) as u32;

        for _ in 0..nblocks {
            f.write_char(PROGRESSBAR_FULL_CHAR)?;
        }

        for _ in nblocks..bar_length {
            f.write_char(PROGRESSBAR_EMPTY_CHAR)?;
        }

        Ok(())
    }
}

pub mod colors {
    use serenity::utils::Color;

    // https://www.colorhexa.com/77dd77
    pub static PASTEL_GREEN: Color = Color::new(0x77dd77);
    pub static PASTEL_RED: Color = Color::new(0xff6961);
}

pub mod emojis {
    pub static LARGE_GREEN_CIRCLE: char = '\u{1F7E2}'; // https://emojipedia.org/large-green-circle/
    pub static LARGE_RED_CIRCLE: char = '\u{1F534}'; // https://emojipedia.org/large-red-circle/
    pub static LARGE_YELLOW_CIRCLE: char = '\u{1F7E1}'; // https://emojipedia.org/large-yellow-circle/
}
