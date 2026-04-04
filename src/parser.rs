use std::collections::HashMap;

use crate::args::{ArgumentTrait, ParseError, ParsedArgs};
use crate::macro_types::{ArgumentType::*, ParsedValue};

struct OptionalEntry {
    arg: Box<dyn ArgumentTrait>,
    seen: bool,
}

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

    /// Adds an argument of either type: `Argument` or `OptionalArgument`
    pub fn add_arg(mut self, arg: impl ArgumentTrait + 'static) -> Self {
        if arg.is_required() {
            if arg.get_argtype() == Boolean {
                panic!("Required argument must not be of type boolean.");
            }
            self.required_args.push(Box::new(arg));
        } else {
            let idx = self.optional_args.len();
            if let Some(short) = arg.get_short_form() {
                self.short_map.insert(short, idx);
            }
            self.long_map.insert(arg.get_name(), idx);

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
        let mut seen_all_required = false;
        let mut parse_options = true;
        while let Some(arg) = it.next() {
            let argument = if let Some(nxt) = current_required.next() {
                Some(nxt)
            } else {
                seen_all_required = true;
                None
            };

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
                    let tp = opt_arg.get_argtype();
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
                    let name = opt_arg.get_name();
                    if self.optional_args[short_arg_idx].seen {
                        return Err(ParseError::DuplicateArgument(name));
                    }

                    let tp = opt_arg.get_argtype();
                    if tp == Boolean {
                        parsed.insert(opt_arg.get_name(), ParsedValue::Boolean(true));
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
            } else if seen_all_required {
                return Err(ParseError::TooManyArguments);
            } else if let Some(argument) = argument {
                let value = match argument.get_argtype() {
                    Text => ParsedValue::Text(arg),
                    Integer => ParsedValue::Integer(match arg.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(ParseError::WrongType {
                                name: argument.get_name(),
                                expected: Integer,
                                given: arg,
                            });
                        }
                    }),
                    Boolean => ParsedValue::Boolean(true),
                };

                parsed.insert(argument.get_name(), value);
            }
        }

        if let Some(argument) = current_required.next() {
            return Err(ParseError::MissingRequired(argument.get_name()));
        }

        for entry in &self.optional_args {
            if !entry.seen {
                let arg = &entry.arg;
                if let Some(default) = arg.get_default_value() {
                    parsed.insert(arg.get_name(), default);
                } else {
                    unreachable!(
                        "All optional arguments require an optional value {} doesn't have one.",
                        arg.get_name()
                    );
                }
            }
        }

        Ok(parsed)
    }
}
