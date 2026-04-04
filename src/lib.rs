mod args;
mod macro_types;
mod parser;

pub use args::{Argument, OptionalArgument, ParseError, ParsedArgs};
pub use macro_types::{ArgumentType, ArgumentType::*, ParsedValue};
pub use parser::ArgumentParser;

#[cfg(test)]
mod tests {}
