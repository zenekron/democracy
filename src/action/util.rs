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

            fn register() -> Vec<serenity::all::CreateCommand> {
                let mut res = Vec::<_>::new();

                $(
                    res.append(&mut $var::register());
                )+

                res
            }
        }

        impl<'a> std::convert::TryFrom<&'a serenity::model::prelude::Interaction> for $name {
            type Error = crate::action::ParseActionError;

            fn try_from(value: &'a serenity::model::prelude::Interaction) -> Result<Self, Self::Error> {
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
        use serenity::all::CommandDataOptionValue;

        if let CommandDataOptionValue::$kind(val) = $resolved {
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
