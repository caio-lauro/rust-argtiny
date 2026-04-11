use std::collections::HashMap;

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
    /// Creates a new required `Argument`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use argtiny::{Argument, ArgumentType::*};
    ///
    /// let arg = Argument::new("input", Text);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `name` starts with `'-'`.
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
/// The short form is optional, using `None` as its value. Otherwise, specify `Some(&str)`.
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
    /// Creates a new `OptionalArgument`.
    ///
    /// # Example
    /// ```rust
    /// use argtiny::{OptionalArgument, ArgumentType::*, ParsedValue};
    ///
    /// let arg = OptionalArgument::new("count", Some("c"), Integer, ParsedValue::Integer(42));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `long` is empty or starts with `'-'`
    /// - `short` is `Some("")` or starts with `'-'`
    /// - `default` type does not match with `argtype`
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

/// Stores the values of the arguments given to [`ArgumentParser::parse`](crate::ArgumentParser). \
/// For required arguments, stores in the order given. \
/// For optional arguments, if given, tries to store the next
/// available [`String`] as its value, converted to its type. \
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

    /// Gets the value of a given argument by `name` as a reference to [`ParsedValue`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use argtiny::{Argument, ArgumentParser, ArgumentType::*, ParsedValue};
    ///
    /// let parsed = ArgumentParser::new()
    ///     .add_arg(Argument::new("input", Text))
    ///     .parse(["program", "input.txt"].map(String::from))?;
    ///
    /// assert_eq!(parsed.get("input"), &ParsedValue::Text("input.txt".to_string()));
    /// # Ok::<(), argtiny::ParseError>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `name` was not registered as a long form of an argument.
    pub fn get(&self, name: &str) -> &ParsedValue {
        self.values
            .get(name)
            .unwrap_or_else(|| panic!("Argument {name} not found"))
    }

    /// Gets the value of a given argument by `name` directly converted to type `T`. \
    /// Uses `get` internally.
    ///
    /// # Example
    ///
    /// ```rust
    /// use argtiny::{Argument, ArgumentParser, ArgumentType::*, ParsedValue};
    ///
    /// let parsed = ArgumentParser::new()
    ///     .add_arg(Argument::new("input", Text))
    ///     .parse(["program", "input.txt"].map(String::from))?;
    ///
    /// assert_eq!(parsed.get_as::<String>("input"), "input.txt".to_string());
    /// # Ok::<(), argtiny::ParseError>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `name` was not registered as a long form of an argument.
    pub fn get_as<T: FromParsedValue>(&self, name: &str) -> T {
        T::from_parsed(self.get(name), name)
    }
}

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
