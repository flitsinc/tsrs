use tsrs_core::CompilerOptions;

use tsrs_tsoptions::commandlineoption::CompilerOptionsValue;
use tsrs_tsoptions::parsinghelpers::{
    for_each_compiler_options_field, parse_compiler_options_worker,
};

#[test]
fn cached_typed_lists_and_paths_survive_option_parsing() {
    let mut options = CompilerOptions::default();
    parse_compiler_options_worker(
        "types",
        &CompilerOptionsValue::StringArray(vec!["node".to_string()]),
        &mut options,
    );
    parse_compiler_options_worker(
        "paths",
        &CompilerOptionsValue::Object(
            [("x".to_string(), CompilerOptionsValue::StringArray(vec!["./x".to_string()]))]
                .into_iter()
                .collect(),
        ),
        &mut options,
    );
    assert_eq!(options.types, Some(vec!["node".to_string()]));
    assert_eq!(options.paths.as_ref().unwrap().get("x"), Some(&Some(vec!["./x".to_string()])));
    parse_compiler_options_worker(
        "types",
        &CompilerOptionsValue::StringArray(Vec::new()),
        &mut options,
    );
    assert_eq!(options.types, Some(Vec::new()));
}

#[test]
fn test_parse_compiler_option_no_missing_fields() {
    let mut missing_keys: Vec<&str> = Vec::new();
    macro_rules! check {
        ($($field:ident: $json:literal,)*) => {
            $(
                let mut co = CompilerOptions::default();
                // Go passes the field's zero value; a float is accepted by every value parser.
                if !parse_compiler_options_worker($json, &CompilerOptionsValue::Float(0.0), &mut co) {
                    missing_keys.push($json);
                }
            )*
        };
    }
    for_each_compiler_options_field!(check);
    assert!(
        missing_keys.is_empty(),
        "The following keys are missing entries in the ParseCompilerOptions switch statement:\n{missing_keys:?}"
    );
}
