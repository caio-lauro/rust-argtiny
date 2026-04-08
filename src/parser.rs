use std::{collections::HashMap, fmt::Display};

use crate::args::{ArgumentTrait, ParsedArgs};
use crate::macro_types::{
    ArgumentType::{self, *},
    ParsedValue,
};

struct OptionalEntry {
    arg: Box<dyn ArgumentTrait>,
    seen: bool,
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

/// Argument parser, to be used with std::env::args or other types that implement
/// `IntoIterator` and have `String` as items.
pub struct ArgumentParser {
    required_args: Vec<Box<dyn ArgumentTrait>>,
    optional_args: Vec<OptionalEntry>,
    short_map: HashMap<String, usize>,
    long_map: HashMap<String, usize>,
}

impl ArgumentParser {
    /// Creates an `ArgumentParser`, which will parse any and all arguments given to it, after the **first** one.
    pub fn new() -> Self {
        ArgumentParser {
            required_args: vec![],
            optional_args: vec![],
            short_map: HashMap::new(),
            long_map: HashMap::new(),
        }
    }

    /// Adds an `Argument` or `OptionalArgument` to the parser.
    ///
    /// # Panics
    ///
    /// Panics if `Argument` is of `Boolean` type.
    pub fn add_arg(mut self, arg: impl ArgumentTrait + 'static) -> Self {
        if arg.is_required() {
            if arg.argtype() == Boolean {
                panic!("Required argument must not be of type boolean.");
            }
            self.required_args.push(Box::new(arg));
        } else {
            let idx = self.optional_args.len();
            if let Some(short) = arg.short_form() {
                self.short_map.insert(short, idx);
            }
            self.long_map.insert(arg.name(), idx);

            self.optional_args.push(OptionalEntry {
                arg: Box::new(arg),
                seen: false,
            });
        }

        self
    }

    /// Parses any and all arguments given **after** the first one. \
    /// `args` must implement `IntoIterator` trait and each item must be of type `String`.
    ///
    /// Parses *required* arguments in the order they were given. \
    /// For *optional* arguments, if they are not seen, their default value is used.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if:
    /// - a required argument is missing ([`ParseError::MissingRequired`])
    /// - a value is missing after an optional argument ([`ParseError::MissingValue`])
    /// - a value cannot be converted to the expected type ([`ParseError::WrongType`])
    /// - an unknown argument is encountered ([`ParseError::UnknownArgument`])
    /// - an argument is given more than once ([`ParseError::DuplicateArgument`])
    /// - more positional arguments given than expected ([`ParseError::TooManyArguments`])
    ///
    /// # Panics
    ///
    /// Panics if `args` yields no items at all. \
    /// When using [`std::env::args`] this will never happen.
    pub fn parse(
        mut self,
        args: impl IntoIterator<Item = String>,
    ) -> Result<ParsedArgs, ParseError> {
        let mut it = args.into_iter();

        if let None = it.next() {
            panic!("No arguments at all");
        }

        let mut parsed = ParsedArgs::new();
        let mut current_required = self.required_args.iter();
        let mut parse_options = true;
        while let Some(arg) = it.next() {
            if parse_options && arg == "--" {
                parse_options = false;
                continue;
            }

            if parse_options && arg.starts_with("--") {
                let name = arg[2..].to_string();
                if let Some(&long_arg_idx) = self.long_map.get(&name) {
                    if self.optional_args[long_arg_idx].seen {
                        return Err(ParseError::DuplicateArgument(name));
                    }

                    let opt_arg = &self.optional_args[long_arg_idx].arg;
                    let tp = opt_arg.argtype();
                    if tp == Boolean {
                        parsed.insert(name, ParsedValue::Boolean(true));
                        self.optional_args[long_arg_idx].seen = true;
                        continue;
                    }

                    let value = if let Some(nxt) = it.next() {
                        nxt
                    } else {
                        return Err(ParseError::MissingValue(name));
                    };

                    match tp {
                        Boolean => (),
                        Text => parsed.insert(name, ParsedValue::Text(value)),
                        Integer => parsed.insert(
                            name.clone(),
                            ParsedValue::Integer(match value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(ParseError::WrongType {
                                        name,
                                        expected: Integer,
                                        given: value,
                                    });
                                }
                            }),
                        ),
                    }
                    self.optional_args[long_arg_idx].seen = true;
                } else {
                    return Err(ParseError::UnknownArgument(name));
                }
            } else if parse_options && arg.starts_with("-") {
                let name = arg[1..].to_string();
                if let Some(&short_arg_idx) = self.short_map.get(&name) {
                    let opt_arg = &self.optional_args[short_arg_idx].arg;
                    let name = opt_arg.name();
                    if self.optional_args[short_arg_idx].seen {
                        return Err(ParseError::DuplicateArgument(name));
                    }

                    let tp = opt_arg.argtype();
                    if tp == Boolean {
                        parsed.insert(opt_arg.name(), ParsedValue::Boolean(true));
                        self.optional_args[short_arg_idx].seen = true;
                        continue;
                    }

                    let value = if let Some(nxt) = it.next() {
                        nxt
                    } else {
                        return Err(ParseError::MissingValue(name));
                    };

                    match tp {
                        Boolean => (),
                        Text => parsed.insert(name, ParsedValue::Text(value)),
                        Integer => parsed.insert(
                            name.clone(),
                            ParsedValue::Integer(match value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(ParseError::WrongType {
                                        name,
                                        expected: Integer,
                                        given: value,
                                    });
                                }
                            }),
                        ),
                    }
                    self.optional_args[short_arg_idx].seen = true;
                } else {
                    return Err(ParseError::UnknownArgument(name));
                }
            } else if let Some(argument) = current_required.next() {
                let value = match argument.argtype() {
                    Text => ParsedValue::Text(arg),
                    Integer => ParsedValue::Integer(match arg.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(ParseError::WrongType {
                                name: argument.name(),
                                expected: Integer,
                                given: arg,
                            });
                        }
                    }),
                    Boolean => ParsedValue::Boolean(true),
                };

                parsed.insert(argument.name(), value);
            } else {
                return Err(ParseError::TooManyArguments);
            }
        }

        if let Some(argument) = current_required.next() {
            return Err(ParseError::MissingRequired(argument.name()));
        }

        for entry in &self.optional_args {
            if !entry.seen {
                let arg = &entry.arg;
                if let Some(default) = arg.default_value() {
                    parsed.insert(arg.name(), default);
                } else {
                    unreachable!(
                        "All optional arguments require an optional value {} doesn't have one.",
                        arg.name()
                    );
                }
            }
        }

        Ok(parsed)
    }
}

impl Default for ArgumentParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{Argument, OptionalArgument};
    use crate::macro_types::*;

    fn make_args(args: &[&str]) -> impl IntoIterator<Item = String> {
        std::iter::once("program".to_string()).chain(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parse_required_text() {
        let parsed = ArgumentParser::new()
            .add_arg(Argument::new("filename", Text))
            .parse(make_args(&["filename.txt"]))
            .unwrap();
        assert_eq!(parsed.get_as::<String>("filename"), "filename.txt");
    }

    #[test]
    fn parse_required_integer() {
        let parsed = ArgumentParser::new()
            .add_arg(Argument::new("port", Integer))
            .parse(make_args(&["8080"]))
            .unwrap();
        assert_eq!(parsed.get_as::<i64>("port"), 8080);
    }

    #[test]
    fn parse_optional_long_form() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "port",
                Some("p"),
                Integer,
                ParsedValue::Integer(8080),
            ))
            .parse(make_args(&["--port", "42"]))
            .unwrap();
        assert_eq!(parsed.get_as::<i64>("port"), 42);
    }

    #[test]
    fn parse_optional_short_form() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "port",
                Some("p"),
                Integer,
                ParsedValue::Integer(8080),
            ))
            .parse(make_args(&["-p", "42"]))
            .unwrap();
        assert_eq!(parsed.get_as::<i64>("port"), 42);
    }

    #[test]
    fn parse_optional_bool_long_form() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "verbose",
                Some("v"),
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&["--verbose"]))
            .unwrap();
        assert_eq!(parsed.get_as::<bool>("verbose"), true);
    }

    #[test]
    fn parse_optional_bool_short_form() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "verbose",
                Some("v"),
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&["-v"]))
            .unwrap();
        assert_eq!(parsed.get_as::<bool>("verbose"), true);
    }

    #[test]
    fn parse_optional_uses_default_value() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "verbose",
                Some("v"),
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&[]))
            .unwrap();
        assert_eq!(parsed.get_as::<bool>("verbose"), false);
    }

    #[test]
    fn parse_optional_without_short_form() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "version",
                None,
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&["--version"]))
            .unwrap();
        assert_eq!(parsed.get_as::<bool>("version"), true);
    }

    #[test]
    fn parse_optional_without_short_form_uses_default_value() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "version",
                None,
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&[]))
            .unwrap();
        assert_eq!(parsed.get_as::<bool>("version"), false);
    }

    #[test]
    fn parse_double_dash_stops_option_parsing() {
        let parsed = ArgumentParser::new()
            .add_arg(Argument::new("filename", Text))
            .parse(make_args(&["--", "--filename.txt"]))
            .unwrap();
        assert_eq!(parsed.get_as::<String>("filename"), "--filename.txt");
    }

    #[test]
    fn parse_double_dash_with_nothing_after() {
        let result = ArgumentParser::new().parse(make_args(&["--"]));
        assert!(result.is_ok());
    }

    #[test]
    fn parse_multiple_required_arguments() {
        let parsed = ArgumentParser::new()
            .add_arg(Argument::new("input", Text))
            .add_arg(Argument::new("output", Text))
            .add_arg(Argument::new("count", Integer))
            .parse(make_args(&["input.txt", "output.txt", "42"]))
            .unwrap();
        assert_eq!(parsed.get_as::<String>("input"), "input.txt");
        assert_eq!(parsed.get_as::<String>("output"), "output.txt");
        assert_eq!(parsed.get_as::<i64>("count"), 42);
    }

    #[test]
    fn parse_multiple_optional_arguments() {
        let parsed = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "verbose",
                Some("v"),
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .add_arg(OptionalArgument::new(
                "port",
                Some("p"),
                Integer,
                ParsedValue::Integer(8080),
            ))
            .add_arg(OptionalArgument::new(
                "version",
                None,
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&["-p", "42", "-v"]))
            .unwrap();
        assert_eq!(parsed.get_as::<bool>("verbose"), true);
        assert_eq!(parsed.get_as::<i64>("port"), 42);
        assert_eq!(parsed.get_as::<bool>("version"), false);
    }

    #[test]
    fn parse_multiple_optional_and_required_arguments() {
        let parsed = ArgumentParser::new()
            .add_arg(Argument::new("input", Text))
            .add_arg(OptionalArgument::new(
                "verbose",
                Some("v"),
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .add_arg(Argument::new("output", Text))
            .parse(make_args(&["input.txt", "-v", "output.txt"]))
            .unwrap();
        assert_eq!(parsed.get_as::<String>("input"), "input.txt");
        assert_eq!(parsed.get_as::<bool>("verbose"), true);
        assert_eq!(parsed.get_as::<String>("output"), "output.txt");
    }

    #[test]
    fn parse_errors_missing_required() {
        let result = ArgumentParser::new()
            .add_arg(Argument::new("filename", Text))
            .parse(make_args(&[]));
        assert!(matches!(result, Err(ParseError::MissingRequired(_))));
    }

    #[test]
    fn parse_errors_missing_value() {
        let result = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "port",
                Some("p"),
                Integer,
                ParsedValue::Integer(8080),
            ))
            .parse(make_args(&["-p"]));
        assert!(matches!(result, Err(ParseError::MissingValue(_))));
    }

    #[test]
    fn parse_errors_wrong_type() {
        let result = ArgumentParser::new()
            .add_arg(Argument::new("count", Integer))
            .parse(make_args(&["definitely_not_integer"]));
        assert!(matches!(result, Err(ParseError::WrongType { .. })));
    }

    #[test]
    fn parse_errors_unknown_argument() {
        let result = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "port",
                Some("p"),
                Integer,
                ParsedValue::Integer(8080),
            ))
            .parse(make_args(&["-c", "42"]));
        assert!(matches!(result, Err(ParseError::UnknownArgument(_))));
    }

    #[test]
    fn parse_errors_duplicate_argument_short_then_long() {
        let result = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "verbose",
                Some("v"),
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&["-v", "--verbose"]));
        assert!(matches!(result, Err(ParseError::DuplicateArgument(_))));
    }

    #[test]
    fn parse_errors_duplicate_argument_long_then_short() {
        let result = ArgumentParser::new()
            .add_arg(OptionalArgument::new(
                "verbose",
                Some("v"),
                Boolean,
                ParsedValue::Boolean(false),
            ))
            .parse(make_args(&["--verbose", "-v"]));
        assert!(matches!(result, Err(ParseError::DuplicateArgument(_))));
    }

    #[test]
    fn parse_errors_too_many_arguments() {
        let result = ArgumentParser::new()
            .add_arg(Argument::new("filename", Text))
            .parse(make_args(&["filename.txt", "extra_argument"]));
        println!("{result:?}");
        assert!(matches!(result, Err(ParseError::TooManyArguments)));
    }

    #[test]
    fn parse_errors_positional_with_no_required_args() {
        let result = ArgumentParser::new().parse(make_args(&["unexpected_argument"]));
        assert!(matches!(result, Err(ParseError::TooManyArguments)));
    }

    #[test]
    #[should_panic]
    fn panics_on_get_unknown_argument() {
        let parsed = ArgumentParser::new()
            .add_arg(Argument::new("input", Text))
            .parse(make_args(&["input.txt"]))
            .unwrap();
        parsed.get("output");
    }

    #[test]
    #[should_panic]
    fn panics_on_required_boolean() {
        ArgumentParser::new().add_arg(Argument::new("verbose", Boolean));
    }
}
