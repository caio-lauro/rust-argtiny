use argparser::{
    Argument, ArgumentParser, ArgumentType::*, OptionalArgument, ParseError, ParsedValue,
};

#[test]
fn full_usage() -> Result<(), Box<dyn std::error::Error>> {
    let parser = ArgumentParser::new()
        .add_arg(Argument::from("input", Text))
        .add_arg(Argument::from("output", Text))
        .add_arg(OptionalArgument::from(
            "verbose",
            None,
            Boolean,
            ParsedValue::Boolean(false),
        ))
        .add_arg(Argument::from("port", Integer))
        .add_arg(OptionalArgument::from(
            "count",
            Some("c"),
            Integer,
            ParsedValue::Integer(1),
        ));
    let args = &[
        "program",
        "input.txt",
        "--verbose",
        "output.txt",
        "8080",
        "-c",
        "42",
    ];
    let parsed = parser.parse(args.map(|s| s.to_string()))?;

    assert_eq!(
        parsed.get("input"),
        &ParsedValue::Text("input.txt".to_string())
    );
    assert_eq!(
        parsed.get_value::<String>("output"),
        String::from("output.txt")
    );
    assert_eq!(parsed.get_value::<i64>("port"), 8080);
    assert_eq!(parsed.get_value::<bool>("verbose"), true);
    assert_eq!(parsed.get_value::<i64>("count"), 42);

    Ok(())
}
