#[test]
fn go_kind_text() {
    assert_eq!(
        tsrs_api::printing::go_kind_text("unexpected node: TypeLiteral"),
        "unexpected node: KindTypeLiteral"
    );
    assert_eq!(
        tsrs_api::printing::go_kind_text("unexpected node: KindTypeLiteral"),
        "unexpected node: KindTypeLiteral"
    );
    assert_eq!(tsrs_api::printing::go_kind_text("index out of range: 3"), "index out of range: 3");
}

#[test]
fn unnameable_type_node_uses_empty_binary_or_json_null() {
    use crate::common::*;
    use tsrs_api::{Handler, Response};
    use tsrs_core::json::{self, Value};

    let dir = TempDir::new("unnameable_type");
    let config = dir.write("tsconfig.json", "{}");
    let file_name = dir.write("main.ts", "export const x = new (class { a = 1; })();\n");
    for binary in [false, true] {
        let session = session(&dir.dir(), binary);
        let created = call(
            &session,
            "createSnapshot",
            &format!(r#"{{"openProjects":[{}]}}"#, quote(&config)),
        );
        let Value::Number(snapshot) = get(&created, "snapshot") else { panic!() };
        let Value::Number(project) = get(&created, "projects.0.handle") else { panic!() };
        let data = session.snapshot_data(*snapshot as u64).unwrap();
        let program = data.get_program(*project as u64).unwrap();
        let file = program.get_source_file(&file_name).unwrap();
        let location = node_request_handle(&session.node_handle_from(file.as_node()).unwrap());
        drop(data);
        let type_ = call(
            &session,
            "getTypeAtPosition",
            &format!(
                r#"{{"snapshot":{snapshot},"project":{project},"file":{},"position":13}}"#,
                quote(&file_name)
            ),
        );
        let payload = format!(
            r#"{{"snapshot":{snapshot},"project":{project},"type":{},"location":{}}}"#,
            json::marshal(get(&type_, "id")).unwrap(),
            quote(&location)
        );
        let response = session.handle_request("typeToTypeNode", payload.as_bytes()).unwrap();
        assert_eq!(
            response,
            if binary { Response::Binary(Vec::new()) } else { Response::Json("null".to_string()) }
        );
        session.close();
    }
}
