use serenity::model::prelude::application_command::CommandDataOptionValue;

#[derive(Debug, thiserror::Error)]
pub enum ParseActionError {
    #[error("action did not match the given input")]
    MismatchedAction,

    #[error("no matching action found for the given input")]
    NoMatchingActionFound,

    #[error("")]
    InvalidInteractionKind,

    #[error("unknown option `{1}` for command `{0}`")]
    UnknownOption(&'static str, String),

    #[error("no value provided for option `{1}` of command `{0}`")]
    MissingOption(&'static str, &'static str),

    #[error(
        "expected value for option `{1}` of command `{0}` to be a `{2}` but found `{3:?}` instead"
    )]
    InvalidOptionKind(
        &'static str,
        String,
        &'static str,
        Option<CommandDataOptionValue>,
    ),

    #[error("invalid value for option `{1}` of command `{0}`: `{2}`")]
    InvalidOptionValue(&'static str, String, Box<dyn std::error::Error>),
}
