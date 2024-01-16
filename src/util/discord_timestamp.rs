use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};

#[allow(dead_code)]
#[derive(Debug)]
pub enum DiscordTimestampStyle {
    FullLong,
    FullShort,
    DateLong,
    DateShort,
    TimeLong,
    TimeShort,
    Relative,
}

#[derive(Debug)]
pub struct DiscordTimestamp(DateTime<Utc>, DiscordTimestampStyle);

impl DiscordTimestamp {
    pub fn new<Tz>(dt: DateTime<Tz>, style: DiscordTimestampStyle) -> Self
    where
        Tz: TimeZone,
    {
        Self(dt.with_timezone(&Utc), style)
    }

    pub fn with_style(&self, style: DiscordTimestampStyle) -> Self {
        Self(self.0, style)
    }
}

impl Display for DiscordTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<t:{}:{}>",
            self.0.timestamp(),
            match self.1 {
                DiscordTimestampStyle::FullLong => 'F',
                DiscordTimestampStyle::FullShort => 'f',
                DiscordTimestampStyle::DateLong => 'D',
                DiscordTimestampStyle::DateShort => 'd',
                DiscordTimestampStyle::TimeLong => 'T',
                DiscordTimestampStyle::TimeShort => 't',
                DiscordTimestampStyle::Relative => 'R',
            }
        )
    }
}

impl Into<String> for DiscordTimestamp {
    fn into(self) -> String {
        self.to_string()
    }
}
