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

    //
    // config
    //
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),

    //
    // seaorm
    //
    #[error(transparent)]
    SeaOrmError(#[from] sea_orm::error::DbErr),

    //
    // serenity
    //
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),

    #[error(transparent)]
    UserIdParseError(#[from] UserIdParseError),
}
