/// Creates an [`Argument`] or [`OptionalArgument`] with a concise syntax.
/// 
/// # Syntax
/// 
/// ```text
/// arg!(required: "name", Type)
/// arg!(optional: "long", Type = default value)
/// arg!(optional: "long", "short", Type = default value)
/// ```
///
/// # Examples
///
/// Required argument:
/// ```rust
/// use argtiny::{ArgumentParser, arg};
///
/// let parser = ArgumentParser::new()
///     .add_arg(arg!(required: "input", Text));
/// ```
///
/// Optional argument without short form:
/// ```rust
/// use argtiny::{ArgumentParser, arg};
///
/// let parser = ArgumentParser::new()
///     .add_arg(arg!(optional: "verbose", Boolean = false));
/// ```
///
/// Optional argument with short form:
/// ```rust
/// use argtiny::{ArgumentParser, arg};
///
/// let parser = ArgumentParser::new()
///     .add_arg(arg!(optional: "count", "c", Integer = 1));
/// ```
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
