use std::fmt::{Display, Write};

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
        let value = (self.value as f32) / (self.max as f32).max(1.0);
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
