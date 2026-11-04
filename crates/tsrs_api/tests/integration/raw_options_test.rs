use tsrs_core::json;

use tsrs_api::raw_options::compiler_options_from_raw;

#[test]
fn resolves_api_path_options_against_server_cwd() {
    let input = json::unmarshal(
        r#"{"outDir":"dist","rootDirs":["src","${configDir}/generated"],"tsBuildInfoFile":"cache/build.tsbuildinfo","sourceRoot":"sources\\mapped","configFilePath":"","pathsBasePath":"config","NoImplicitAny":true}"#,
    )
    .unwrap();
    let options = compiler_options_from_raw(&input, "/project").unwrap();
    assert_eq!(options.out_dir, "/project/dist");
    assert_eq!(options.root_dirs.unwrap(), ["/project/src", "/project/generated"]);
    assert_eq!(options.ts_build_info_file, "/project/cache/build.tsbuildinfo");
    assert_eq!(options.source_root, "sources/mapped");
    assert_eq!(options.config_file_path, "");
    assert_eq!(options.paths_base_path, "/project/config");
    assert_eq!(options.no_implicit_any, tsrs_core::Tristate::Unknown);
}

#[test]
fn raw_values_follow_finalize_instead_of_typed_struct_decoding() {
    // Recorded through exact e2bc's native API; RawCompilerOptions.UnmarshalJSONFrom stores any
    // JSON value and Finalize converts it through ParseCompilerOptions.
    let input = json::unmarshal(
        r#"{"target":1e1,"jsx":12345,"strict":"yes","checkers":1.5,"maxNodeModuleJsDepth":1e0,"types":["a",1,null,"b"],"rootDirs":["src",false,"generated","","${CONFIGDIR}/literal"],"paths":{"*a":["first",null,"second"],"*":5},"NoImplicitAny":true}"#,
    )
    .unwrap();
    let options = compiler_options_from_raw(&input, "/project").unwrap();
    assert_eq!(options.target.value(), 10);
    assert_eq!(options.jsx.value(), 12345);
    assert_eq!(options.strict, tsrs_core::Tristate::False);
    assert_eq!(options.checkers, Some(1));
    assert_eq!(options.max_node_module_js_depth, Some(1));
    assert_eq!(options.types.unwrap(), ["a", "b"]);
    assert_eq!(
        options.root_dirs.unwrap(),
        ["/project/src", "/project/generated", "/project", "/project/${CONFIGDIR}/literal"]
    );
    let paths = options.paths.unwrap();
    assert_eq!(paths.get("*a").unwrap().as_ref().unwrap(), &["first", "second"]);
    assert!(paths.get("*").unwrap().is_none());
    assert_eq!(options.no_implicit_any, tsrs_core::Tristate::Unknown);
}

#[test]
fn raw_numbers_use_float64_before_native_integer_conversion() {
    let input = json::unmarshal(
        r#"{"checkers":9223372036854775807,"maxNodeModuleJsDepth":9223372036854775808,"target":9007199254740993}"#,
    )
    .unwrap();
    let options = compiler_options_from_raw(&input, "/project").unwrap();
    // Both positive literals round to 2^63 before Go converts them. x86 returns MIN outside range;
    // ARM saturates. These are native Go results, not an exact JSON-integer preservation contract.
    let expected = if cfg!(target_arch = "x86_64") { i64::MIN } else { i64::MAX };
    assert_eq!(options.checkers, Some(expected));
    assert_eq!(options.max_node_module_js_depth, Some(expected));
    assert_eq!(
        options.target.value(),
        if cfg!(target_arch = "x86_64") { i32::MIN } else { i32::MAX }
    );
}
