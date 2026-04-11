//! A simple, lightweight argument parser library for Rust.
//!
//! Support for required positional arguments and optional arguments with
//! long (`--name`) and short (`-n`) forms. Optional arguments always
//! have a default value.
//!
//! # Examples
//!
//! Using the [`arg`] macro to initialize arguments for ArgumentParser (recommended):
//! ```rust
//! use argtiny::{ArgumentParser, ArgumentType::*, arg};
//!
//! let parser = ArgumentParser::new()
//!     .add_arg(arg!(required: "input", Text))
//!     .add_arg(arg!(required: "output", Text))
//!     .add_arg(arg!(optional: "verbose", "v", Boolean = false))
//!     .add_arg(arg!(optional: "count", Integer = 1));
//!
//! # let _ = parser;
//! ```
//!
//! Using constructors directly:
//! ```rust
//! use argtiny::{Argument, ArgumentParser, ArgumentType::*, OptionalArgument, ParsedValue};
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
//! # let _ = parser;
//! ```
//!
//! Pass [`std::env::args`] directly to [`ArgumentParser::parse`] in a real program,
//! or use an iterator of [`String`] for testing.
//!
//! Parsing and getting values:
//! ```rust
//! use argtiny::{ArgumentParser, arg, ArgumentType::*, ParseError};
//!
//! let parsed = ArgumentParser::new()
//!     .add_arg(arg!(required: "input", Text))
//!     .add_arg(arg!(required: "output", Text))
//!     .add_arg(arg!(optional: "verbose", "v", Boolean = false))
//!     .add_arg(arg!(optional: "count", Integer = 1))
//!     .parse(["program", "input.txt", "output.txt", "--count", "42"].map(String::from))?;
//!
//! let input: String = parsed.get_as("input");
//! let output: String = parsed.get_as("output");
//!
//! assert_eq!(input, "input.txt");
//! assert_eq!(output, "output.txt");
//!
//! assert_eq!(parsed.get_as::<bool>("verbose"), false);
//! assert_eq!(parsed.get_as::<i64>("count"), 42);
//! # Ok::<(), ParseError>(())
//! ```

mod args;
mod macro_types;
mod macros;
mod parser;

pub use args::{Argument, OptionalArgument, ParsedArgs};
#[doc(hidden)]
pub use macro_types::FromParsedValue;
pub use macro_types::{ArgumentType, ArgumentType::*, ParsedValue};
pub use parser::{ArgumentParser, ParseError};
