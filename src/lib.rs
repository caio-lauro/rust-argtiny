//! A simple, lightweight argument parser library for Rust.
//!
//! Support for required positional arguments and optional arguments with
//! long (`--name`) and short (`-n`) forms. Optional arguments always
//! have a default value.
//!
//! # Example
//!
//! ```rust
//! use argparser::{Argument, ArgumentParser, ArgumentType::*, OptionalArgument, ParsedValue};
//!
//! let parser = ArgumentParser::new()
//!     .add_arg(Argument::new("input", Text))
//!     .add_arg(Argument::new("output", Text))
//!     .add_arg(OptionalArgument::new(
//!         "verbose",
//!         Some("v"),
//!         Boolean,
//!         ParsedValue::Boolean(false),
//!     ))
//!     .add_arg(OptionalArgument::new(
//!         "count",
//!         None,
//!         Integer,
//!         ParsedValue::Integer(1),
//!     ));
//!
//! let parsed = parser.parse(
//!     [
//!         "program",
//!         "input.txt",
//!         "output.txt",
//!         "--count",
//!         "42",
//!     ]
//!     .map(String::from),
//! )?;
//!
//! assert_eq!(parsed.get_as::<String>("input"), "input.txt");
//! assert_eq!(parsed.get_as::<String>("output"), "output.txt");
//! assert_eq!(parsed.get_as::<bool>("verbose"), false);
//! assert_eq!(parsed.get_as::<i64>("count"), 42);
//! # Ok::<(), argparser::ParseError>(())
//! ```

mod args;
mod macro_types;
mod parser;

pub use args::{Argument, OptionalArgument, ParseError, ParsedArgs};
pub use macro_types::{ArgumentType, ArgumentType::*, FromParsedValue, ParsedValue};
pub use parser::ArgumentParser;
