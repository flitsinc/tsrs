// Port of tsc/internal/project/snapshot_test.go (TestSnapshot). Only the subtests that exercise
// Session.APIUpdate are ported so far.

use rustc_hash::FxHashMap;
use std::sync::Arc;

use tsrs_core::collections::new_set_from_items;
use tsrs_core::context::Context;
use tsrs_core::tspath::Path;
use tsrs_lsproto as lsproto;
use tsrs_vfs::{FS, bundled, vfstest};

use tsrs_project::filechange::{FileChange, FileChangeKind, FileChangeSummary};
use tsrs_project::project::new_synthetic_project_id;
use tsrs_project::session::{Session, SessionInit, SessionOptions, new_session};
use tsrs_project::snapshot::APISnapshotRequest;

// snapshot_test.go:23
fn setup(files: &[(&str, &str)]) -> Arc<Session> {
    let fs: Arc<dyn FS> = Arc::new(bundled::wrap_fs(Arc::new(vfstest::from_map(
        files.iter().map(|(k, v)| (k.to_string(), vfstest::MapFile::from(v.to_string()))),
        false, /*useCaseSensitiveFileNames*/
    ))));
    new_session(&SessionInit {
        background_ctx: Context::background(),
        options: Arc::new(SessionOptions {
            current_directory: "/".to_string(),
            default_library_path: bundled::lib_path(),
            typings_location: "/home/src/Library/Caches/typescript".to_string(),
            position_encoding: lsproto::PositionEncodingKind::UTF8,
            watch_enabled: false,
            logging_enabled: false,
            ..Default::default()
        }),
        fs,
        client: None,
        logger: None,
        npm_executor: None,
        spawner: None,
        content_mapper_logger: None,
        parse_cache: None,
        content_mapped_parse_cache: None,
    })
}

#[test]
fn close_projects_retains_the_default_project_of_api_opened_files() {
    let session = setup(&[
        ("/src/tsconfig.json", r#"{"compilerOptions":{"noLib":true},"files":["main.ts"]}"#),
        ("/src/main.ts", "export const value = 1;"),
    ]);
    let ctx = Context::background();
    let config = Path::from("/src/tsconfig.json");
    let file = Path::from("/src/main.ts");
    let snapshot = session
        .api_update(
            &ctx,
            &FileChangeSummary::default(),
            Some(Arc::new(APISnapshotRequest {
                open_projects: Some(new_set_from_items([config.0.clone()])),
                open_files: Some(FxHashMap::from_iter([(file.clone(), file.0.clone())])),
                ..Default::default()
            })),
        )
        .unwrap();
    let project = snapshot.project_collection.configured_project(&config).unwrap();
    let program = project.get_program().unwrap();
    snapshot.deref();
    let snapshot = session
        .api_update(
            &ctx,
            &FileChangeSummary::default(),
            Some(Arc::new(APISnapshotRequest {
                close_projects: Some(new_set_from_items([config.clone()])),
                ..Default::default()
            })),
        )
        .unwrap();
    let retained = snapshot.project_collection.configured_project(&config).unwrap();
    assert!(std::ptr::eq(program, retained.get_program().unwrap()));
    snapshot.deref();
    let snapshot = session
        .api_update(
            &ctx,
            &FileChangeSummary::default(),
            Some(Arc::new(APISnapshotRequest {
                close_files: Some(new_set_from_items([file])),
                ..Default::default()
            })),
        )
        .unwrap();
    assert!(snapshot.project_collection.configured_project(&config).is_none());
    snapshot.deref();
    session.close();
}

// snapshot_test.go:154
#[test]
fn failed_api_update_preserves_flushed_host_changes() {
    let session = setup(&[("/a.ts", "export const a = 1;")]);

    let base_snapshot = session.snapshot();
    session.pending_file_changes.lock().unwrap().push(FileChange {
        kind: FileChangeKind::WatchChange,
        uri: lsproto::DocumentUri("file:///a.ts".to_string()),
        ..Default::default()
    });
    let failed_snapshot = session.api_update(
        &Context::background(),
        &FileChangeSummary::default(),
        Some(Arc::new(APISnapshotRequest {
            remove_programs: Some(new_set_from_items([new_synthetic_project_id(1)])),
            ..Default::default()
        })),
    );

    let err = failed_snapshot.err().expect("APIUpdate should fail");
    assert!(err.message.contains("synthetic program not found for removal"), "{}", err.message);
    assert!(!Arc::ptr_eq(&session.snapshot(), &base_snapshot));
    session.close();
}
