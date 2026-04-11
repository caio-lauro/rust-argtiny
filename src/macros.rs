#[macro_export]
macro_rules! arg {
    (required: $name: literal, $type: ident) => {
        $crate::Argument::new($name, $crate::ArgumentType::$type)
    };
    (optional: $name: literal, $type: ident = $default:expr) => {{
        use $crate::{ArgumentType, OptionalArgument, ParsedValue};
        OptionalArgument::new(
            $name,
            None,
            ArgumentType::$type,
            ParsedValue::$type($default),
        )
    }};
    (optional: $name: literal, $short: literal, $type: ident = $default:expr) => {{
        use $crate::{ArgumentType, OptionalArgument, ParsedValue};
        OptionalArgument::new(
            $name,
            Some($short),
            ArgumentType::$type,
            ParsedValue::$type($default),
        )
    }};
}
