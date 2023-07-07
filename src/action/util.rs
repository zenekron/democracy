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
