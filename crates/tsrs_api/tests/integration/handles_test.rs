use crate::common::*;
use tsrs_api::{Handler, Response};
use tsrs_core::json::Value;

fn number(value: &Value) -> u64 {
    let Value::Number(value) = value else { panic!("expected numeric handle: {value:?}") };
    *value as u64
}

#[test]
fn numeric_project_handles_identify_lifetimes_within_each_snapshot() {
    let dir = TempDir::new("project_handles");
    let file = dir.write("index.ts", "export const x = 1;");
    let session = session(&dir.dir(), false);
    let params = format!(
        r#"{{"createPrograms":[{{"rootFiles":[{}],"compilerOptions":{{"noLib":true}}}}]}}"#,
        quote(&file)
    );
    let first = call(&session, "createSnapshot", &params);
    let second = call(&session, "createSnapshot", &params);
    assert_eq!(get(&first, "projects.0.id"), get(&second, "projects.0.id"));
    let first_handle = number(get(&first, "projects.0.handle"));
    let second_handle = number(get(&second, "projects.0.handle"));
    assert_ne!(first_handle, second_handle);
    let first_snapshot = number(get(&first, "snapshot"));
    let second_snapshot = number(get(&second, "snapshot"));
    for (snapshot, project) in [(first_snapshot, first_handle), (second_snapshot, second_handle)] {
        let names = call(
            &session,
            "getSourceFileNames",
            &format!(r#"{{"snapshot":{snapshot},"project":{project}}}"#),
        );
        assert!(
            matches!(names, Value::Array(files) if files.contains(&Value::String(file.clone())))
        );
    }
    let error = session
        .handle_request(
            "getSourceFileNames",
            format!(r#"{{"snapshot":{second_snapshot},"project":{first_handle}}}"#).as_bytes(),
        )
        .unwrap_err();
    assert_eq!(error.kind, tsrs_api::ErrorKind::ClientError);
    for project in [quote("#42"), quote(&file)] {
        let error = session
            .handle_request(
                "getSourceFileNames",
                format!(r#"{{"snapshot":{first_snapshot},"project":{project}}}"#).as_bytes(),
            )
            .unwrap_err();
        assert_eq!(error.kind, tsrs_api::ErrorKind::InvalidRequest);
    }
    let update = call(
        &session,
        "updateSnapshot",
        &format!(r#"{{"snapshot":{first_snapshot},"changes":{{}}}}"#),
    );
    let updated_data = session.snapshot_data(number(get(&update, "snapshot"))).unwrap();
    assert!(updated_data.get_program(first_handle).is_ok());
    drop(updated_data);
    session.close();
}

#[test]
fn node_request_handles_reject_a_separately_parsed_file_with_identical_text() {
    let dir = TempDir::new("node_handles");
    let file_name = dir.write("index.ts", "export const x = 1;");
    let session = session(&dir.dir(), false);
    let response = call(
        &session,
        "createSnapshot",
        &format!(
            r#"{{"createPrograms":[{{"rootFiles":[{}],"compilerOptions":{{"noLib":true}}}}]}}"#,
            quote(&file_name)
        ),
    );
    let snapshot = number(get(&response, "snapshot"));
    let project = number(get(&response, "projects.0.handle"));
    let data = session.snapshot_data(snapshot).unwrap();
    let program = data.get_program(project).unwrap();
    let file = program.get_source_file(&file_name).unwrap();
    let node_id = tsrs_ast::get_node_id(file.as_node());
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope
                .spawn(move || assert_eq!(program.get_source_file_by_node_id(node_id), Some(file)));
        }
    });
    let response_handle = session.node_handle_from(file.as_node()).unwrap();
    assert!(response_handle.contains(&format!(".#{}.", node_id.0)));
    let query = |handle: &str| {
        session.handle_request(
            "getTypeAtLocation",
            format!(
                r#"{{"snapshot":{snapshot},"project":{project},"location":{}}}"#,
                quote(handle)
            )
            .as_bytes(),
        )
    };
    assert!(matches!(query(&format!("1.#{}", node_id.0)), Ok(Response::Json(_))));
    let separate_session = crate::common::session(&dir.dir(), false);
    let separate = call(
        &separate_session,
        "createSnapshot",
        &format!(
            r#"{{"createPrograms":[{{"rootFiles":[{}],"compilerOptions":{{"noLib":true}}}}]}}"#,
            quote(&file_name)
        ),
    );
    let separate_data = separate_session.snapshot_data(number(get(&separate, "snapshot"))).unwrap();
    let separate_file = separate_data
        .get_program(number(get(&separate, "projects.0.handle")))
        .unwrap()
        .get_source_file(&file_name)
        .unwrap();
    let separate_id = tsrs_ast::get_node_id(separate_file.as_node()).0;
    assert_ne!(separate_id, node_id.0);
    let error = query(&format!("1.#{separate_id}")).unwrap_err();
    assert_eq!(error.kind, tsrs_api::ErrorKind::ClientError);
    for handle in [
        response_handle,
        "1.#0".to_string(),
        "1.#18446744073709551616".to_string(),
        format!("0.#{}", node_id.0),
    ] {
        assert_eq!(query(&handle).unwrap_err().kind, tsrs_api::ErrorKind::ClientError);
    }
    drop(data);
    drop(separate_data);
    separate_session.close();
    session.close();
}
