#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown command `{0}`")]
    UnknownCommand(String),

    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
}
