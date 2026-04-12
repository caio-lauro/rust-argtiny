use argtiny::{ArgumentParser, ParseError, arg};

#[test]
fn arg_macro_required_works() -> Result<(), ParseError> {
    let parsed = ArgumentParser::new()
        .add_arg(arg!(required: "input", Text))
        .parse(["program", "input.txt"].map(String::from))?;
    assert_eq!(parsed.get_as::<String>("input"), "input.txt");
    Ok(())
}

#[test]
fn arg_macro_optional_with_short_works() -> Result<(), ParseError> {
    let parsed = ArgumentParser::new()
        .add_arg(arg!(optional: "verbose", "v", Boolean = false))
        .parse(["program", "--verbose"].map(String::from))?;
    assert_eq!(parsed.get_as::<bool>("verbose"), true);
    Ok(())
}

#[test]
fn arg_macro_optional_without_short_works() -> Result<(), ParseError> {
    let parsed = ArgumentParser::new()
        .add_arg(arg!(optional: "count", Integer = 1))
        .parse(["program", "--count", "42"].map(String::from))?;
    assert_eq!(parsed.get_as::<i64>("count"), 42);
    Ok(())
}

#[test]
fn arg_macro_default_value_used_when_absent() -> Result<(), ParseError> {
    let parsed = ArgumentParser::new()
        .add_arg(arg!(optional: "count", "c", Integer = 42))
        .parse(["program"].map(String::from))?;
    assert_eq!(parsed.get_as::<i64>("count"), 42);
    Ok(())
}
