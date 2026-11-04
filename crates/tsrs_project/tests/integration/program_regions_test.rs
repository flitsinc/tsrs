use tsrs_core::context::Context;
use tsrs_lsproto as lsproto;

use crate::projecttestutil::setup;

#[test]
fn watched_file_create_reloads_an_existing_cached_dependency() {
    let files = [
        (
            "/src/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true,"module":"esnext","moduleResolution":"bundler"},"files":["main.ts"]}"#,
        ),
        ("/src/main.ts", "import { value } from './dep'; export const typed: 1 = value;"),
        ("/src/dep.ts", "export const value = 1;"),
    ];
    let (session, utils) = setup(&files);
    let ctx = Context::background();
    let main = lsproto::DocumentUri("file:///src/main.ts".to_string());
    let ls = session.get_language_service(&ctx, &main).unwrap();
    let file = ls.get_program().get_source_file("/src/main.ts").unwrap();
    assert_eq!(
        ls.get_program().get_semantic_diagnostics(&ctx, Some(file)),
        [] as [tsrs_core::P<tsrs_ast::Diagnostic>; 0]
    );
    utils.fs().write_file("/src/dep.ts", "export const value = 2;").unwrap();
    session.did_change_watched_files(
        &ctx,
        &[lsproto::FileEvent {
            uri: lsproto::DocumentUri("file:///src/dep.ts".to_string()),
            type_: lsproto::FileChangeType::CREATED,
        }],
    );
    let ls = session.get_language_service(&ctx, &main).unwrap();
    let program = ls.get_program();
    assert_eq!(program.get_source_file("/src/dep.ts").unwrap().text(), "export const value = 2;");
    let file = program.get_source_file("/src/main.ts").unwrap();
    assert!(
        program
            .get_semantic_diagnostics(&ctx, Some(file))
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322)
    );
    session.close();
}

fn worker_arena_bytes() -> usize {
    tsrs_compiler::worker_pool().broadcast(|_| tsrs_core::ptr::arena_used_bytes()).into_iter().sum()
}

// Isolated because this measures the process-wide compiler pool's permanent arenas.
#[test]
fn watched_file_rebuilds_release_loader_allocations() {
    const CHILD: &str = "TSRS_LOADER_REGIONS_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "program_regions_test::watched_file_rebuilds_release_loader_allocations",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("RAYON_NUM_THREADS", "2")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }

    let source = |round: usize| {
        let mut text = String::new();
        for i in 0..64 {
            let dependency = if i == 0 { round % 2 } else { i };
            writeln!(text, "import {{ value as v{i} }} from './dep{dependency}';").unwrap();
        }
        writeln!(text, "export const result = v0 + {round};").unwrap();
        text
    };
    let mut files = vec![
        (
            "/src/tsconfig.json".to_string(),
            r#"{"compilerOptions":{"noLib":true,"module":"esnext","moduleResolution":"bundler"},"files":["index.ts"]}"#.to_string(),
        ),
        ("/src/index.ts".to_string(), source(0)),
        ("/src/leaf.ts".to_string(), "export const leaf = 1;".to_string()),
    ];
    for i in 0..64 {
        files.push((
            format!("/src/dep{i}.ts"),
            format!("import {{ leaf }} from './leaf'; export const value = leaf + {i};"),
        ));
    }
    let borrowed: Vec<_> =
        files.iter().map(|(path, text)| (path.as_str(), text.as_str())).collect();
    let (session, utils) = setup(&borrowed);
    let ctx = Context::background();
    let uri = lsproto::DocumentUri("file:///src/index.ts".to_string());

    let check = |round| {
        session
            .with_language_service_and_snapshot(&ctx, &uri, |ls, _snapshot| {
                let program = ls.get_program();
                let file = program.get_source_file("/src/index.ts").unwrap();
                assert_eq!(file.text(), source(round));
                assert_eq!(program.get_semantic_diagnostics(&ctx, Some(file)), Vec::new());
                Ok(None)
            })
            .unwrap();
    };
    check(0);
    let mut warm_bytes = 0;
    for round in 1..=30 {
        utils.fs().write_file("/src/index.ts", &source(round)).unwrap();
        session.did_change_watched_files(
            &ctx,
            &[lsproto::FileEvent { uri: uri.clone(), type_: lsproto::FileChangeType::CHANGED }],
        );
        // Alternating an import forces UpdateProgram's full-build fallback, even for a single dirty file.
        check(round);
        if round == 5 {
            warm_bytes = worker_arena_bytes();
        }
    }
    let end_bytes = worker_arena_bytes();
    eprintln!("compiler worker arena bytes: {warm_bytes} -> {end_bytes}");
    assert!(
        end_bytes <= warm_bytes + (64 << 10),
        "full builds must release loader allocations instead of keeping them in worker arenas"
    );
}
use std::fmt::Write;
