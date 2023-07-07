#[macro_export(local_inner_macros)]
macro_rules! create_actions {
    ($name:ident, $($var:ident),+) => {
        pub enum $name {
            $(
                $var($var),
            )+
        }

        impl $name {
            pub fn register_all(commands: &mut serenity::builder::CreateApplicationCommands) -> &mut serenity::builder::CreateApplicationCommands {
                $(
                    if let Some(command) = $var::register() {
                        commands.create_application_command(move |cmd| {
                            *cmd = command;
                            cmd
                        });
                    }
                )+

                commands
            }
        }

        #[serenity::async_trait]
        impl Action for $name {
            const ID: &'static str = std::stringify!($name);

            async fn execute(&self, ctx: &serenity::client::Context) -> Result<(), crate::error::Error> {
                match self {
                    $(
                        Self::$var(action) => action.execute(ctx).await,
                    )+
                }
            }

            fn register() -> Option<serenity::builder::CreateApplicationCommand> {
                std::unimplemented!("use `register_all` instead")
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
    ($resolved:expr, $kind:ident, $name:expr) => {{
        use serenity::model::prelude::application_command::CommandDataOptionValue;

        if let Some(CommandDataOptionValue::$kind(val)) = $resolved {
            Ok(val)
        } else {
            Err(ParseActionError::InvalidOptionKind(
                Self::ID,
                $name.into(),
                std::stringify!($kind),
                $resolved.clone(),
            ))
        }
    }};
}
