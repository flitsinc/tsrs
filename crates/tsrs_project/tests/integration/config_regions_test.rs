use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};

use tsrs_core::arena::Region;
use tsrs_core::collections::new_set_from_items;
use tsrs_core::context::Context;
use tsrs_core::tspath::Path;
use tsrs_lsproto as lsproto;
use tsrs_project::FileChangeSummary;
use tsrs_project::snapshot::APISnapshotRequest;
use tsrs_vfs::{FS, bundled, vfstest};

use crate::projecttestutil::setup;

const ROOT: &str = r#"{"extends":"./base.json","files":["index.ts"]}"#;
const INDEX: &str = "export const value: number = undefined;";
const LOOSE: &str =
    r#"{"compilerOptions":{"noLib":true,"strictNullChecks":false,"unknownOption":true}}"#;
const STRICT: &str =
    r#"{"compilerOptions":{"noLib":true,"strictNullChecks":true,"unknownOption":true}}"#;

fn on_freed(addr: usize) -> Arc<AtomicBool> {
    let region = Region::containing(addr).expect("config generation has an owner");
    let freed = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&freed);
    region.on_free(Box::new(move || observed.store(true, Ordering::SeqCst)));
    freed
}

fn changed(session: &tsrs_project::Session, path: &str, kind: lsproto::FileChangeType) {
    session.did_change_watched_files(
        &Context::background(),
        &[lsproto::FileEvent { uri: lsproto::DocumentUri(format!("file://{path}")), type_: kind }],
    );
}

#[test]
fn replaced_config_and_extends_generations_live_until_old_language_service_is_dropped() {
    let (session, utils) =
        setup(&[("/src/tsconfig.json", ROOT), ("/src/base.json", LOOSE), ("/src/index.ts", INDEX)]);
    let ctx = Context::background();
    let uri = lsproto::DocumentUri("file:///src/index.ts".to_string());
    let old = session.get_language_service(&ctx, &uri).unwrap();
    let old_program = old.get_program();
    let root_freed = on_freed(old_program.command_line().addr());
    let old_errors = old_program.get_config_file_parsing_diagnostics();
    let error = old_errors.iter().find(|error| error.code() == 5023).unwrap();
    let extended_freed = on_freed(error.addr());
    assert_eq!(
        old_program
            .get_semantic_diagnostics(&ctx, old_program.get_source_file("/src/index.ts"))
            .len(),
        0
    );

    utils.fs().write_file("/src/base.json", STRICT).unwrap();
    changed(&session, "/src/base.json", lsproto::FileChangeType::CHANGED);
    let new = session.get_language_service(&ctx, &uri).unwrap();
    let new_program = new.get_program();
    let new_freed = on_freed(new_program.command_line().addr());
    assert_eq!(
        new_program
            .get_semantic_diagnostics(&ctx, new_program.get_source_file("/src/index.ts"))
            .len(),
        1
    );
    assert!(!root_freed.load(Ordering::SeqCst));
    assert!(!extended_freed.load(Ordering::SeqCst));
    assert_eq!(error.file().unwrap().text(), LOOSE);
    assert!(!old_program.options().strict_null_checks.is_true());
    assert!(new_program.options().strict_null_checks.is_true());

    session.close();
    assert!(!new_freed.load(Ordering::SeqCst));
    drop(new);
    assert!(new_freed.load(Ordering::SeqCst));
    drop(old);
    assert!(root_freed.load(Ordering::SeqCst));
    assert!(extended_freed.load(Ordering::SeqCst));
}

#[test]
fn filename_reloads_release_intermediate_generations_and_keep_original_config_data() {
    let (session, utils) = setup(&[
        ("/src/tsconfig.json", r#"{"compilerOptions":{"noLib":true},"include":["*.ts"]}"#),
        ("/src/index.ts", "export const value = 1;"),
    ]);
    let ctx = Context::background();
    let uri = lsproto::DocumentUri("file:///src/index.ts".to_string());
    let first = session.get_language_service(&ctx, &uri).unwrap();
    let data_freed = on_freed(first.get_program().command_line().addr());
    drop(first);
    let mut previous: Option<Arc<AtomicBool>> = None;
    for round in 0..8 {
        let path = format!("/src/added{round}.ts");
        utils.fs().write_file(&path, "export const added = 1;").unwrap();
        changed(&session, &path, lsproto::FileChangeType::CREATED);
        let ls = session.get_language_service(&ctx, &uri).unwrap();
        session.wait_for_background_tasks();
        let command_line = ls.get_program().command_line();
        assert_eq!(command_line.file_names().len(), round + 2);
        assert!(command_line.compiler_options().unwrap().no_lib.is_true());
        if let Some(previous) = previous.take() {
            assert!(
                previous.load(Ordering::SeqCst),
                "intermediate filename generation must be freed"
            );
        }
        previous = Some(on_freed(command_line.addr()));
        assert!(!data_freed.load(Ordering::SeqCst));
    }
    session.close();
    assert!(previous.unwrap().load(Ordering::SeqCst));
    assert!(data_freed.load(Ordering::SeqCst));
}

#[test]
fn concurrent_snapshot_clones_retain_their_own_extended_config_generations() {
    let (session, _) = setup(&[]);
    let host = Arc::clone(session.snapshot_host());
    let base = session.snapshot();
    let barrier = Barrier::new(4);
    let snapshots = std::thread::scope(|scope| {
        let mut tasks = Vec::new();
        for worker in 0..4 {
            let (host, base, barrier) = (&host, &base, &barrier);
            tasks.push(scope.spawn(move || {
                let mut snapshots = Vec::new();
                barrier.wait();
                for round in 0..12 {
                    let strict = (worker + round) % 2 == 0;
                    let fs: Arc<dyn FS> = Arc::new(bundled::wrap_fs(vfstest::from_map(
                        [
                            ("/src/tsconfig.json", ROOT),
                            ("/src/base.json", if strict { STRICT } else { LOOSE }),
                            ("/src/index.ts", INDEX),
                        ],
                        false,
                    )));
                    let snapshot = host
                        .clone_snapshot(
                            &Context::background(),
                            base,
                            FileChangeSummary::default(),
                            Some(Arc::new(APISnapshotRequest {
                                file_system: Some(fs),
                                replace_file_system: true,
                                open_projects: Some(new_set_from_items([
                                    "/src/tsconfig.json".to_string()
                                ])),
                                ensure_all_programs: true,
                                ..Default::default()
                            })),
                        )
                        .map_err(|(_, error)| error)
                        .unwrap();
                    snapshots.push((snapshot, strict));
                }
                snapshots
            }));
        }
        tasks.into_iter().flat_map(|task| task.join().unwrap()).collect::<Vec<_>>()
    });
    for (snapshot, strict) in snapshots {
        let project = snapshot
            .project_collection
            .configured_project(&Path::from("/src/tsconfig.json"))
            .unwrap();
        let program = project.program.unwrap();
        assert_eq!(program.options().strict_null_checks.is_true(), strict);
        let errors = program.get_config_file_parsing_diagnostics();
        let error = errors.iter().find(|error| error.code() == 5023).unwrap();
        assert_eq!(error.file().unwrap().text(), if strict { STRICT } else { LOOSE });
        snapshot.deref();
    }
    session.close();
}
