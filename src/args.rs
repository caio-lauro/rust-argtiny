use std::{collections::HashMap, fmt::Display};

use crate::macro_types::{
    ArgumentType::{self, *},
    FromParsedValue, ParsedValue,
};

pub trait ArgumentTrait {
    fn is_required(&self) -> bool;
    fn get_name(&self) -> String;
    fn get_short_form(&self) -> Option<String>;
    fn get_argtype(&self) -> ArgumentType;
    fn get_default_value(&self) -> Option<ParsedValue>;
}

/// Type of argument to be used in case you want a required argument.
#[derive(Debug, Clone)]
pub struct Argument<'a> {
    name: &'a str,
    argtype: ArgumentType,
}

impl<'a> ArgumentTrait for Argument<'a> {
    fn is_required(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn get_short_form(&self) -> Option<String> {
        None
    }

    fn get_argtype(&self) -> ArgumentType {
        self.argtype
    }

    fn get_default_value(&self) -> Option<ParsedValue> {
        None
    }
}

impl<'a> Argument<'a> {
    pub fn from(name: &'a str, argtype: ArgumentType) -> Argument<'a> {
        assert!(
            !name.starts_with('-'),
            "required argument name must not start with '-', got: {name:?}"
        );
        Argument { name, argtype }
    }
}

/// Type of argument to be used in case you want an optional argument. \
/// The name of the argument will be considered as the long form of the argument.
///
/// Don't use hyphens when specifying the long and short forms of the argument.
pub struct OptionalArgument<'a> {
    long: &'a str,
    short: &'a str,
    argtype: ArgumentType,
    default: ParsedValue,
}

impl<'a> ArgumentTrait for OptionalArgument<'a> {
    fn is_required(&self) -> bool {
        false
    }

    fn get_name(&self) -> String {
        self.long.to_string()
    }

    fn get_short_form(&self) -> Option<String> {
        Some(self.short.to_string())
    }

    fn get_argtype(&self) -> ArgumentType {
        self.argtype
    }

    fn get_default_value(&self) -> Option<ParsedValue> {
        Some(self.default.clone())
    }
}

impl<'a> OptionalArgument<'a> {
    /// Creates an `OptionalArgument` from `long` and `short` forms as `&str`,
    /// an `ArgumentType` and a `default` value. \
    /// Use the same type for both `argtype` and `default`.
    ///
    /// Don't use hyphens when specifying the long and short forms of the argument.
    pub fn from(
        long: &'a str,
        short: &'a str,
        argtype: ArgumentType,
        default: ParsedValue,
    ) -> OptionalArgument<'a> {
        if !default.matches(&argtype) {
            panic!("Argument type and default value must be of the same type.");
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

    /// Gets the value of a given argument as a reference to ParsedValue. \
    /// Panics in case the argument doesn't exist.
    pub fn get(&self, name: &str) -> &ParsedValue {
        self.values
            .get(name)
            .unwrap_or_else(|| panic!("Argument {name} not found"))
    }

    /// Gets the value of a given argument directly, in case you don't want
    /// to use pattern matching. \
    /// Using the internal method `get`, panics in case the argument doesn't
    /// exist.
    pub fn get_value<T: FromParsedValue>(&self, name: &str) -> T {
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
    /// Different types given for argtype and default
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