use serenity::model::prelude::UserIdParseError;

use crate::{action::ParseActionError, entities::InvitePollId, util::serenity::GuildId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not find an invite poll with id `{0}`")]
    InvitePollNotFound(InvitePollId),

    #[error("value `{0}` is not a valid poll id: {1}")]
    InvitePollIdInvalid(String, Box<dyn std::error::Error>),

    #[error("could not find a guild with id `{0:?}`")]
    GuildNotFound(GuildId),

    #[error(transparent)]
    ParseActionError(#[from] ParseActionError),

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
