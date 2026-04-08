use argtiny::{Argument, ArgumentParser, ArgumentType::*, OptionalArgument, ParsedValue};

fn main() {
    let parser = ArgumentParser::new()
        .add_arg(Argument::new("input", Text))
        .add_arg(Argument::new("output", Text))
        .add_arg(OptionalArgument::new(
            "verbose",
            Some("v"),
            Boolean,
            ParsedValue::Boolean(false),
        ))
        .add_arg(OptionalArgument::new(
            "count",
            Some("c"),
            Integer,
            ParsedValue::Integer(1),
        ));

    let parsed = parser.parse(std::env::args()).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    let input: String = parsed.get_as("input");
    let output: String = parsed.get_as("output");
    let verbose: bool = parsed.get_as("verbose");
    let count: i64 = parsed.get_as("count");

    if verbose {
        println!("input:  {input}");
        println!("output: {output}");
        println!("count:  {count}");
    }
}