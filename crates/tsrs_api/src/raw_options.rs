// The API accepts path-valued compiler options as strings. Go's
// RawCompilerOptions.Finalize resolves them against the server cwd before
// constructing CompilerOptions; the JSON representation alone does not.

use tsrs_core::CompilerOptions;
use tsrs_core::json::Value;
use tsrs_core::tspath;
use tsrs_tsoptions::gojson::GoJson;
use tsrs_tsoptions::tsconfigparsing::COMMAND_LINE_COMPILER_OPTIONS_MAP;
use tsrs_tsoptions::{CompilerOptionsValue, parse_compiler_options};

use crate::handler::{ApiError, ApiResult};

fn rooted_option(path: &str, cwd: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let path = if path
        .get(.."${configDir}".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("${configDir}"))
    {
        // Go detects this prefix without case sensitivity but substitutes only its exact spelling.
        path.replacen("${configDir}", "./", 1)
    } else {
        path.to_string()
    };
    tspath::get_normalized_absolute_path(&path, cwd)
}

/// Go `RawCompilerOptions.Finalize`, for the path-valued fields accepted by
/// the API. This runs only on API input; parsed tsconfig options already have
/// their own base directory and must not be resolved against the server cwd.
#[expect(
    clippy::too_long_first_doc_paragraph,
    reason = "Public for the integration tests; the comment keeps its upstream wording."
)]
pub fn compiler_options_from_raw(value: &Value, cwd: &str) -> ApiResult<CompilerOptions> {
    let mut options = CompilerOptions::default();
    let fields = match value {
        Value::Null => return Ok(options),
        Value::Object(fields) => fields,
        _ => {
            return Err(ApiError::invalid_request(
                "cannot unmarshal non-object JSON value into RawCompilerOptions",
            ));
        }
    };
    for (key, value) in fields {
        if matches!(value, Value::Null) {
            continue;
        }
        // RawCompilerOptions.Finalize accepts only the exact declared spelling and its four internal
        // path/boolean options. ParseCompilerOptions alone also accepts case-insensitive names.
        if let Some(option) = COMMAND_LINE_COMPILER_OPTIONS_MAP.get(key) {
            if key != option.name {
                continue;
            }
        } else if !matches!(
            key,
            "configFilePath" | "pathsBasePath" | "allowNonTsExtensions" | "suppressOutputPathCheck"
        ) {
            continue;
        }
        // Go floatOrInt32ToFlag accepts any numeric int32, including unnamed enum values. Use the
        // existing enum wire conversion to retain them instead of the config parser's enum-map fallback.
        macro_rules! enum_option {
            ($field:ident, $name:literal) => {{
                let number = match value {
                    Value::Number(n) => *n,
                    Value::Integer(n) => *n as f64,
                    _ => {
                        return Err(ApiError::internal(
                            "panic: interface conversion: compiler option is not float64",
                        ));
                    }
                };
                let converted = Value::Number(go_float_to_int32(number) as f64);
                let (decoded, unknown) = GoJson::from_go_json_keeping_unknown(&converted, key)
                    .map_err(ApiError::invalid_request)?;
                options.$field = decoded;
                if let Some(unknown) = unknown {
                    options.api_unknown_enum_values.push(($name, unknown));
                }
            }};
        }
        match key {
            "target" => enum_option!(target, "target"),
            "module" => enum_option!(module, "module"),
            "moduleResolution" => enum_option!(module_resolution, "moduleResolution"),
            "moduleDetection" => enum_option!(module_detection, "moduleDetection"),
            "jsx" => enum_option!(jsx, "jsx"),
            "newLine" => enum_option!(new_line, "newLine"),
            _ => {
                let parsed = match (key, value) {
                    ("checkers" | "maxNodeModuleJsDepth", Value::Number(n)) => {
                        CompilerOptionsValue::Int(go_float_to_int(*n))
                    }
                    _ => crate::config::json_to_options_value(value),
                };
                parse_compiler_options(key, &parsed, &mut options);
            }
        }
    }

    macro_rules! rooted_fields {
        ($($field:ident),* $(,)?) => {
            $(options.$field = rooted_option(&options.$field, cwd);)*
        };
    }
    rooted_fields!(
        declaration_dir,
        out_dir,
        project,
        root_dir,
        ts_build_info_file,
        base_url,
        out_file,
        config_file_path,
        paths_base_path,
        generate_cpu_profile,
        generate_trace,
        pprof_dir,
    );
    for paths in [&mut options.root_dirs, &mut options.type_roots].into_iter().flatten() {
        for path in paths {
            *path = if path.is_empty() {
                tspath::get_normalized_absolute_path(path, cwd)
            } else {
                rooted_option(path, cwd)
            };
        }
    }
    // Source-map locations are URI-like text, not rooted filesystem paths.
    options.map_root = tspath::normalize_slashes(&options.map_root);
    options.source_root = tspath::normalize_slashes(&options.source_root);
    Ok(options)
}

// Go specifies implementation-dependent results outside the target integer range. Its native x86
// conversion returns the minimum value; ARM's saturating conversion matches Rust's cast. Keep the
// API's raw float64 conversion separate from the strict integer decoder used by core.CompilerOptions.
fn go_float_to_int(value: f64) -> i64 {
    if cfg!(target_arch = "x86_64")
        && !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&value)
    {
        return i64::MIN;
    }
    value as i64
}

fn go_float_to_int32(value: f64) -> i32 {
    if cfg!(target_arch = "x86_64") && !(-2_147_483_648.0..2_147_483_648.0).contains(&value) {
        return i32::MIN;
    }
    value as i32
}
