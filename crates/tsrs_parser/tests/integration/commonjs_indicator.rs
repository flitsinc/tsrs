use tsrs_ast::{ExternalModuleIndicatorOptions, SourceFileParseOptions};
use tsrs_core::{ScriptKind, tspath::Path};
use tsrs_parser::parse_source_file;

#[test]
fn common_js_module_indicator_is_computed_before_binding() {
    for (file_name, source, expected) in [
        ("/a.js", "const x = require(\"x\");", true),
        ("/a.js", "module.exports = {};", true),
        ("/a.js", "exports.x = 1;", true),
        ("/a.js", "module.exports.x = 1;", true),
        ("/a.js", "Object.defineProperty(exports, \"x\", { value: 1 });", true),
        ("/a.js", "function f() { exports.x = 1; }", true),
        ("/a.js", "var x = 1; foo(x);", false),
        ("/a.js", "/** @type {{ [require(\"x\")]: number }} */ var x;", true),
        ("/a.js", "/** @param {number} [x=require(\"x\")] */ function f(x) {}", false),
        (
            "/a.js",
            "/**\n * @type {{ [require('x')]: () => void }}\n * @type {() => void}\n */\nfunction f() {}",
            false,
        ),
        (
            "/a.js",
            "/**\n * @type {{ [require('x')]: number }}\n * @type {number}\n */\nthis.x = 1;",
            false,
        ),
        (
            "/a.js",
            "/**\n * @template {{ [require('x')]: number }} T\n * @typedef\n */\nfunction f() {}",
            false,
        ),
        ("/a.js", "exports.x = 1; export {};", false),
        ("/a.js", "export const y = require(\"x\");", false),
        ("/a.js", "/** @satisfies {any} */ module.exports = exports;", true),
        ("/a.js", "await / import.meta / 1;\nawait / require('x') / 1;", false),
        ("/a.js", "await / 1 ; export {}; / ; require('x');", true),
        ("/a.cjs", "exports.x = 1;", true),
        ("/a.cjs", "await / 1 ; export {}; / ; require('x');", true),
        ("/a.mjs", "await /require(\"x\")/;", false),
        ("/a.ts", "exports.x = 1; const y = require(\"x\");", false),
    ] {
        let opts = SourceFileParseOptions {
            file_name: file_name.to_owned(),
            path: Path::from(file_name),
            external_module_indicator_options: ExternalModuleIndicatorOptions {
                force: file_name.ends_with(".cjs") || file_name.ends_with(".mjs"),
                ..Default::default()
            },
        };
        let kind = if file_name.ends_with(".ts") { ScriptKind::TS } else { ScriptKind::JS };
        let file = parse_source_file(opts, source, kind);
        assert_eq!(
            file.common_js_module_indicator.get().is_some(),
            expected,
            "{file_name}: {source}"
        );
    }
}
