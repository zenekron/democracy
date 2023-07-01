use serenity::model::prelude::UserIdParseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown command `{0}`")]
    UnknownCommand(String),

    #[error("unknown option `{1}` for command `{0}`")]
    UnknownCommandOption(String, String),

    #[error("missing option `{1}` for command `{0}`")]
    MissingCommandOption(String, String),

    #[error("command `{0}` can only be issued inside a guild")]
    GuildCommandNotInGuild(String),

    #[error("could not extract a valid poll id")]
    InvitePollIdNotFound,

    #[error("value `{0}` is not a valid poll id: {1}")]
    InvitePollIdInvalid(String, Box<dyn std::error::Error>),

    //
    // config
    //
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),

    //
    // sqlx
    //
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error(transparent)]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    //
    // serenity
    //
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),

    #[error(transparent)]
    UserIdParseError(#[from] UserIdParseError),
}
