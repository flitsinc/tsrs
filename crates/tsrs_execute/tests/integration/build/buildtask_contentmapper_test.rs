use tsrs_core::tspath::Path;

use tsrs_execute::build::buildtask::is_content_mapper_virtual_file_path;
use tsrs_incremental::{BuildInfo, BuildInfoResolvedRoot, BuildInfoRoot};

// buildtask_contentmapper_test.go:11. A build info file entry that names a supplemental output of a root file
// (`<root>.<n>.<ext>`) is skipped when the file is gone, instead of making the build out of date; anything else
// must still be checked.
#[test]
fn test_is_content_mapper_virtual_file_path() {
    let info = BuildInfo {
        root: ["/src/a.vue", "/src/b.ts", "/src/a.vue.0"]
            .into_iter()
            .map(|name| BuildInfoRoot { non_incremental: name.to_string(), ..Default::default() })
            .collect(),
        ..Default::default()
    };
    let roots = info.get_build_info_root_info_reader(
        "/",
        &tsrs_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: true,
            ..Default::default()
        },
    );
    for name in [
        "/src/a.vue.0.ts",
        "/src/a.vue.1.tsx",
        "/src/a.vue.12.mts",
        "/src/a.vue.2.cts",
        "/src/a.vue.0.js",
        "/src/a.vue.0.jsx",
        "/src/a.vue.0.mjs",
        "/src/a.vue.0.cjs",
        "/src/a.vue.0.json",
        "/src/b.ts.0.ts",
        "/src/a.vue.0.1.ts",
        "/src/a.vue.+1.ts",
        "/src/a.vue.-1.ts",
        "/src/a.vue.001.ts",
    ] {
        assert!(is_content_mapper_virtual_file_path(&Path::new(name), &roots), "{name}");
    }
    for name in [
        "/src/a.vue.ts",
        "/src/a.vue.0.txt",
        "/src/a.vue.0.d.ts",
        "/src/a.vue.0.d.mts",
        "/src/a.vue.0.d.cts",
        "/src/a.vue.0",
        "/src/a.vue..ts",
        "/src/a.vue.x.ts",
        "/src/a.vue.99999999999999999999.ts",
        "/src/a.vue.1.0.ts",
        "/src/b.vue.0.ts",
        "/src/a.vuex.0.ts",
        "/src/a.ts",
        "/src/.0.ts",
        "/src/..0.ts",
    ] {
        assert!(!is_content_mapper_virtual_file_path(&Path::new(name), &roots), "{name}");
    }
}

#[test]
fn content_mapper_virtual_file_paths_use_original_roots() {
    let info = BuildInfo {
        file_names: vec!["./src/a.vue".to_string(), "./src/b.vue".to_string()],
        root: vec![BuildInfoRoot { start: 2, ..Default::default() }],
        resolved_root: vec![BuildInfoResolvedRoot { resolved: 2, root: 1 }],
        ..Default::default()
    };
    let roots = info.get_build_info_root_info_reader(
        "/",
        &tsrs_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: true,
            ..Default::default()
        },
    );
    assert!(is_content_mapper_virtual_file_path(&Path::new("/src/a.vue.0.ts"), &roots));
    assert!(!is_content_mapper_virtual_file_path(&Path::new("/src/b.vue.0.ts"), &roots));
}
