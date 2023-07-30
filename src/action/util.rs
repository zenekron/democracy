#[macro_export(local_inner_macros)]
macro_rules! create_actions {
    ($name:ident, $($var:ident),+) => {
        #[derive(Debug)]
        pub enum $name {
            $(
                $var($var),
            )+
        }

        #[serenity::async_trait]
        impl Action for $name {
            async fn execute(&self, ctx: &serenity::client::Context) -> Result<(), crate::error::Error> {
                match self {
                    $(
                        Self::$var(action) => action.execute(ctx).await,
                    )+
                }
            }

            fn register(commands: &mut serenity::builder::CreateApplicationCommands) -> &mut serenity::builder::CreateApplicationCommands {
                $(
                    $var::register(commands);
                )+

                commands
            }
        }

        impl<'a> std::convert::TryFrom<&'a serenity::model::application::interaction::Interaction> for $name {
            type Error = crate::action::ParseActionError;

            fn try_from(value: &'a serenity::model::application::interaction::Interaction) -> Result<Self, Self::Error> {
                $(
                    let res = $var::try_from(value);
                    if !std::matches!(res, Err(ParseActionError::MismatchedAction)) {
                        return Ok(Self::$var(res?));
                    }
                )+

                Err(ParseActionError::NoMatchingActionFound)
            }
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! resolve_option {
    ($action:expr, $resolved:expr, $kind:ident, $name:expr) => {{
        use serenity::model::prelude::application_command::CommandDataOptionValue;

        if let Some(CommandDataOptionValue::$kind(val)) = $resolved {
            Ok(val)
        } else {
            Err(ParseActionError::InvalidOptionKind {
                action: $action,
                option: $name.into(),
                kind: std::stringify!($kind),
                value: $resolved.clone(),
            })
        }
    }};
}
