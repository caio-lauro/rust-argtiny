use std::{collections::HashMap, fmt::Display};

use crate::macro_types::{ArgumentType, FromParsedValue, ParsedValue};

mod private {
    pub trait Sealed {}
}

#[doc(hidden)]
pub trait ArgumentTrait: private::Sealed {
    fn is_required(&self) -> bool;
    fn name(&self) -> String;
    fn short_form(&self) -> Option<String>;
    fn argtype(&self) -> ArgumentType;
    fn default_value(&self) -> Option<ParsedValue>;
}

/// Type of argument to be used in case you want a required argument.
#[derive(Debug, Clone)]
pub struct Argument<'a> {
    name: &'a str,
    argtype: ArgumentType,
}

impl private::Sealed for Argument<'_> {}

impl<'a> ArgumentTrait for Argument<'a> {
    fn is_required(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn short_form(&self) -> Option<String> {
        None
    }

    fn argtype(&self) -> ArgumentType {
        self.argtype
    }

    fn default_value(&self) -> Option<ParsedValue> {
        None
    }
}

impl<'a> Argument<'a> {
    pub fn new(name: &'a str, argtype: ArgumentType) -> Argument<'a> {
        assert!(
            !name.starts_with('-'),
            "required argument name must not start with '-', got: {name:?}"
        );
        Argument { name, argtype }
    }
}

/// Type of argument to be used in case you want an optional argument. \
/// The *name* of the argument will be considered as the long form of the argument.
///
/// Don't use hyphens when specifying the long and short forms of the argument. \
/// Always use a long form, as it is considered the *name* of the argument. \
/// The short form is optional.
#[derive(Debug, Clone)]
pub struct OptionalArgument<'a> {
    long: &'a str,
    short: Option<&'a str>,
    argtype: ArgumentType,
    default: ParsedValue,
}

impl private::Sealed for OptionalArgument<'_> {}

impl<'a> ArgumentTrait for OptionalArgument<'a> {
    fn is_required(&self) -> bool {
        false
    }

    fn name(&self) -> String {
        self.long.to_string()
    }

    fn short_form(&self) -> Option<String> {
        self.short.map(|s| s.to_string())
    }

    fn argtype(&self) -> ArgumentType {
        self.argtype
    }

    fn default_value(&self) -> Option<ParsedValue> {
        Some(self.default.clone())
    }
}

impl<'a> OptionalArgument<'a> {
    /// Creates an `OptionalArgument` from `long` and `short` forms as `&str`,
    /// an `ArgumentType` and a `default` value. \
    /// Use the same type for both `argtype` and `default`.
    ///
    /// Don't use hyphens when specifying the long and short forms of the argument. \
    /// Always use a long form, as it is considered the *name* of the argument. \
    /// The short form is optional.
    pub fn new(
        long: &'a str,
        short: Option<&'a str>,
        argtype: ArgumentType,
        default: ParsedValue,
    ) -> OptionalArgument<'a> {
        assert!(
            default.matches(&argtype),
            "argtype and default must be of the same type, got {argtype:?} and {default:?}"
        );

        assert!(!long.is_empty(), "long type of argument must be used");
        assert!(
            !long.starts_with("-"),
            "long form must not start with '-', got: {long:?}"
        );

        if let Some(s) = short {
            assert!(
                !s.is_empty(),
                "If entered, short type of argument must not be empty"
            );
            assert!(
                !s.starts_with("-"),
                "short form must not start with '-', got: {long:?}"
            );
        }

        OptionalArgument {
            long,
            short,
            argtype,
            default,
        }
    }
}

/// Stores the values of the arguments given to `parse`. \
/// For required arguments, stores in the order given. \
/// For optional arguments, if given, tries to store the next
/// available `String` as its value, converted to its type. \
/// If not given, stores the default value.
#[derive(Debug)]
pub struct ParsedArgs {
    values: HashMap<String, ParsedValue>,
}

impl ParsedArgs {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, name: String, value: ParsedValue) {
        self.values.insert(name, value);
    }

    /// Gets the value of a given argument by `name` as a reference to ParsedValue. \
    /// Panics in case the argument doesn't exist.
    pub fn get(&self, name: &str) -> &ParsedValue {
        self.values
            .get(name)
            .unwrap_or_else(|| panic!("Argument {name} not found"))
    }

    /// Gets the value of a given argument by `name` directly, in case you don't want
    /// to use pattern matching. \
    /// Using the internal method `get`, panics in case the argument doesn't
    /// exist.
    pub fn get_as<T: FromParsedValue>(&self, name: &str) -> T {
        T::from_parsed(self.get(name), name)
    }
}

#[derive(Debug)]
/// Enum for errors when parsing.
pub enum ParseError {
    /// Required argument not given
    MissingRequired(String),
    /// Missing value for argument
    MissingValue(String),
    /// Different types given for argtype and default or unexpected value for parsing
    WrongType {
        name: String,
        expected: ArgumentType,
        given: String,
    },
    /// Unknown argument, not added to ArgumentParsed
    UnknownArgument(String),
    /// Argument already seen once
    DuplicateArgument(String),
    /// Number of positional arguments exceeds number of  required arguments
    TooManyArguments,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::MissingRequired(name) => write!(f, "missing required argument: {name}"),
            ParseError::MissingValue(name) => {
                write!(f, "argument '{name}' requires a value but none was given")
            }
            ParseError::WrongType {
                name,
                expected,
                given,
            } => write!(
                f,
                "argument '{name}' expects value of type {expected:?} given: {given}"
            ),
            ParseError::UnknownArgument(name) => write!(f, "unknown argument: {name}"),
            ParseError::DuplicateArgument(name) => {
                write!(f, "argument '{name}' was given more than once")
            }
            ParseError::TooManyArguments => write!(f, "too many positional arguments"),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macro_types::ArgumentType::*;
    use crate::macro_types::*;

    #[test]
    fn argument_from_valid_name() {
        Argument::new("argument", Text);
    }

    #[test]
    #[should_panic]
    fn argument_from_name_with_dash_panics() {
        Argument::new("--argument", Text);
    }

    #[test]
    #[should_panic]
    fn argument_from_name_with_single_dash_panics() {
        Argument::new("-a", Text);
    }

    #[test]
    fn optional_argument_valid() {
        OptionalArgument::new(
            "argument",
            Some("a"),
            Text,
            ParsedValue::Text(String::new()),
        );
    }

    #[test]
    fn optional_argument_without_short_form() {
        OptionalArgument::new("argument", None, Text, ParsedValue::Text(String::new()));
    }

    #[test]
    #[should_panic]
    fn optional_argument_mismatched_default_panics() {
        OptionalArgument::new("argument", Some("a"), Boolean, ParsedValue::Integer(0));
    }

    #[test]
    #[should_panic]
    fn optional_argument_with_dash_on_long_form_panics() {
        OptionalArgument::new(
            "--argument",
            Some("a"),
            Text,
            ParsedValue::Text(String::new()),
        );
    }

    #[test]
    #[should_panic]
    fn optional_argument_with_empty_long_form_panics() {
        OptionalArgument::new("", None, Text, ParsedValue::Text(String::new()));
    }

    #[test]
    #[should_panic]
    fn optional_argument_with_dash_on_short_form_panics() {
        OptionalArgument::new(
            "argument",
            Some("-a"),
            Text,
            ParsedValue::Text(String::new()),
        );
    }

    #[test]
    #[should_panic]
    fn optional_argument_with_empty_short_form_panics() {
        OptionalArgument::new("argument", Some(""), Text, ParsedValue::Text(String::new()));
    }

    #[test]
    fn optional_argument_get_short() {
        let opt_arg =
            OptionalArgument::new("verbose", Some("v"), Boolean, ParsedValue::Boolean(false));
        assert_eq!(opt_arg.short_form(), Some("v".to_string()));
    }
}
