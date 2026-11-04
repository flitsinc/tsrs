use tsrs_core::tspath::Path;

use tsrs_execute::build::buildtask::is_content_mapper_virtual_file_path;
use tsrs_incremental::{BuildInfo, BuildInfoRoot};

// buildtask_contentmapper_test.go:11. A build info file entry that names a supplemental output of a root file
// (`<root>.<n>.<ext>`) is skipped when the file is gone, instead of making the build out of date; anything else
// must still be checked.
#[test]
fn test_is_content_mapper_virtual_file_path() {
    let info = BuildInfo {
        file_names: vec!["/src/app.vue".to_string(), "/src/index.ts".to_string()],
        root: vec![BuildInfoRoot { start: 1, end: 2, ..Default::default() }],
        ..Default::default()
    };
    let roots = info.get_build_info_root_info_reader(
        "/",
        &tsrs_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: true,
            ..Default::default()
        },
    );
    let path = Path::new;

    assert!(is_content_mapper_virtual_file_path(&path("/src/app.vue.0.ts"), &roots));
    assert!(is_content_mapper_virtual_file_path(&path("/src/app.vue.12.mts"), &roots));
    assert!(!is_content_mapper_virtual_file_path(&path("/src/app.vue.ts"), &roots));
    assert!(!is_content_mapper_virtual_file_path(&path("/src/app.vue.0.txt"), &roots));
    assert!(!is_content_mapper_virtual_file_path(&path("/src/other.vue.0.ts"), &roots));
}
