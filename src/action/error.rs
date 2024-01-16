use serenity::all::CommandDataOptionValue;

#[derive(Debug, thiserror::Error)]
pub enum ParseActionError {
    #[error("action did not match the given input")]
    MismatchedAction,

    #[error("no matching action found for the given input")]
    NoMatchingActionFound,

    #[error(transparent)]
    ParentMessage(#[from] ParseParentMessageError),

    //
    // action
    //
    #[error("invalid action id '{id}'")]
    InvalidActionId {
        action: &'static str,
        id: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("action cannot be performed outside of a guild")]
    NotInAGuild { action: &'static str },

    //
    // options
    //
    #[error("unknown option '{option}'")]
    UnknownOption {
        action: &'static str,
        option: String,
    },

    #[error("missing required option '{option}'")]
    MissingOption {
        action: &'static str,
        option: String,
    },

    #[error("invalid value '{value:?}' for option '{option}': expected a {kind}")]
    InvalidOptionKind {
        action: &'static str,
        option: String,
        kind: &'static str,
        value: CommandDataOptionValue,
    },

    #[error("invalid value '{value}' for '{option}': {source}")]
    InvalidOptionValue {
        action: &'static str,
        option: String,
        value: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    //
    // permissions
    //
    #[error("insufficient permissions")]
    InsufficientPermissions,
}

impl ParseActionError {
    pub fn is_client_error(&self) -> bool {
        match self {
            ParseActionError::MismatchedAction => false,
            ParseActionError::NoMatchingActionFound => true,
            ParseActionError::NotInAGuild { .. } => true,
            ParseActionError::ParentMessage(_) => false,
            ParseActionError::InvalidActionId { .. } => false,
            ParseActionError::UnknownOption { .. } => true,
            ParseActionError::MissingOption { .. } => true,
            ParseActionError::InvalidOptionKind { .. } => true,
            ParseActionError::InvalidOptionValue { .. } => true,
            ParseActionError::InsufficientPermissions => true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseParentMessageError {
    #[error("field '{field}' not found")]
    FieldNotFound { field: String },

    #[error("invalid value '{value}' for '{field}'")]
    InvalidField {
        field: String,
        value: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
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
        assert!(is_send::<ParseActionError>());
        assert!(is_sync::<ParseActionError>());
    }
}
