# argparser

A simple, lightweight argument parser library for Rust.

## Features

- Required positional arguments, parsed in order
- Optional arguments with long (`--name`) and short (`-n`) forms
- Boolean flags (no value needed)
- Default values for optional arguments
- `--` to stop option parsing

## Usage

Add to your `Cargo.toml`:
```toml
[dependencies]
argparser = "0.1.0"
```

## Example
```rust
use argparser::{Argument, ArgumentParser, ArgumentType::*, OptionalArgument, ParsedValue};

fn main() {
    let parser = ArgumentParser::new()
        .add_arg(Argument::from("input", Text))
        .add_arg(Argument::from("output", Text))
        .add_arg(OptionalArgument::from(
            "verbose",
            Some("v"),
            Boolean,
            ParsedValue::Boolean(false),
        ))
        .add_arg(OptionalArgument::from(
            "count",
            Some("c"),
            Integer,
            ParsedValue::Integer(1),
        ));

    let parsed = parser.parse(std::env::args()).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    let input = parsed.get_value::<String>("input");
    let output = parsed.get_value::<String>("output");
    let verbose = parsed.get_value::<bool>("verbose");
    let count = parsed.get_value::<i64>("count");

    if verbose {
        println!("input:  {input}");
        println!("output: {output}");
        println!("count:  {count}");
    }
}
```

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.