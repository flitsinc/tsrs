use crate::common::*;
use tsrs_api::{ErrorKind, Handler};
use tsrs_core::json::Value;

fn snapshot(response: &Value) -> u64 {
    let Value::Number(handle) = get(response, "snapshot") else { panic!() };
    *handle as u64
}

#[test]
fn snapshot_filesystem_composes_retained_entries_and_survives_source_release() {
    for source_kind in ["full", "layer"] {
        for extra_kind in ["full", "layer"] {
            let dir = TempDir::new("snapshot_filesystem");
            let host = dir.write("host.ts", "host");
            let session = session(&dir.dir(), false);
            let source = call(
                &session,
                "createSnapshot",
                &format!(
                    r#"{{"fileSystem":{{"kind":"{source_kind}","files":{{"/source.ts":"source","/shared.ts":"source","/removed/nested.ts":"removed"}}}}}}"#
                ),
            );
            let source_id = snapshot(&source);
            let target = call(
                &session,
                "createSnapshot",
                r#"{"fileSystem":{"kind":"layer","files":{"/target.ts":"target","/shared.ts":"target"}}}"#,
            );
            let target_id = snapshot(&target);
            let combined = call(
                &session,
                "updateSnapshot",
                &format!(
                    r#"{{"snapshot":{target_id},"changes":{{"fileSystem":{{"kind":"snapshot","snapshot":{source_id},"fileSystem":{{"kind":"{extra_kind}","files":{{"/extra.ts":"extra","/shared.ts":"extra"}},"symlinks":{{"/alias.ts":{{"target":"/shared.ts"}}}},"removedPaths":["/removed"]}}}}}}}}"#
                ),
            );
            call(&session, "release", &format!(r#"{{"snapshot":{source_id}}}"#));
            let data = session.snapshot_data(snapshot(&combined)).unwrap();
            for file in ["/shared.ts", "/extra.ts", "/alias.ts"] {
                assert_eq!(data.snapshot.read_file(file).as_deref(), Some("extra"));
            }
            assert_eq!(data.snapshot.read_file("/source.ts").is_some(), extra_kind == "layer");
            assert_eq!(
                data.snapshot.read_file("/target.ts").is_some(),
                source_kind == "layer" && extra_kind == "layer"
            );
            assert_eq!(
                data.snapshot.read_file(&host).is_some(),
                source_kind == "layer" && extra_kind == "layer"
            );
            assert!(data.snapshot.read_file("/removed/nested.ts").is_none());
            drop(data);
            let original = session.snapshot_data(target_id).unwrap();
            assert_eq!(original.snapshot.read_file("/shared.ts").as_deref(), Some("target"));
            drop(original);
            session.close();
        }
    }
}

#[test]
fn snapshot_filesystem_rejects_invalid_descriptors_without_releasing_its_source() {
    let dir = TempDir::new("snapshot_filesystem_invalid");
    let session = session(&dir.dir(), false);
    let source = call(
        &session,
        "createSnapshot",
        r#"{"fileSystem":{"kind":"layer","files":{"/source.ts":"source"}}}"#,
    );
    let source_id = snapshot(&source);
    for descriptor in [
        r#"{"kind":"snapshot"}"#.to_string(),
        r#"{"kind":"snapshot","snapshot":999999}"#.to_string(),
        format!(r#"{{"kind":"snapshot","snapshot":{source_id},"files":{{"/extra.ts":"extra"}}}}"#),
        format!(r#"{{"kind":"layer","snapshot":{source_id}}}"#),
        r#"{"kind":"full","fileSystem":{"kind":"layer"}}"#.to_string(),
        format!(
            r#"{{"kind":"snapshot","snapshot":{source_id},"fileSystem":{{"kind":"snapshot","snapshot":{source_id}}}}}"#
        ),
    ] {
        let error = session
            .handle_request(
                "createSnapshot",
                format!(r#"{{"fileSystem":{descriptor}}}"#).as_bytes(),
            )
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::ClientError, "{descriptor}: {error}");
        assert!(session.snapshot_data(source_id).is_ok());
    }
    let error = session.handle_request("createSnapshot", format!(r#"{{"fileSystem":{{"kind":"snapshot","snapshot":{source_id},"fileSystem":{{"kind":"layer","files":{{"/x":true}}}}}}}}"#).as_bytes()).unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidRequest);
    session.close();
}
