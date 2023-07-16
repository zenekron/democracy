use serenity::model::prelude::UserIdParseError;

use crate::{
    action::ParseActionError,
    entities::InvitePollId,
    util::serenity::{GuildId, UserId},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not find an invite poll with id `{0}`")]
    InvitePollNotFound(InvitePollId),

    #[error("value `{0}` is not a valid poll id: {1}")]
    InvitePollIdInvalid(String, Box<dyn std::error::Error + Send + Sync>),

    #[error("could not find a guild with id `{0:?}`")]
    GuildNotFound(GuildId),

    #[error("user '{0}' is already a member")]
    CannotInviteMember(UserId),

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

impl Error {
    pub fn is_client_error(&self) -> bool {
        match self {
            Error::InvitePollNotFound(_) => true,
            Error::InvitePollIdInvalid(_, _) => true,
            Error::GuildNotFound(_) => true,
            Error::CannotInviteMember(_) => true,
            Error::ParseActionError(err) => err.is_client_error(),
            Error::ConfigError(_) => false,
            Error::DatabaseError(_) => false,
            Error::MigrationError(_) => false,
            Error::SerenityError(_) => false,
            Error::UserIdParseError(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_send<T: Send>() -> bool {
        true
    }

    fn is_sync<T: Sync>() -> bool {
        true
    }

    #[test]
    fn test_is_send_and_sync() {
        assert!(is_send::<Error>());
        assert!(is_sync::<Error>());
    }
}
