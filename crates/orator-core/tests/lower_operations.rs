use orator_core::ir::OperationIr;
use orator_core::lower::lower_operations;

fn load_and_lower_ops(yaml: &str) -> Vec<OperationIr> {
    let spec = oas3::from_yaml(yaml).unwrap();
    lower_operations(&spec).unwrap()
}

#[test]
fn tennis_club_operations() {
    let ops = load_and_lower_ops(include_str!(
        "../../../examples/tennis-club-core/tennis-club.yaml"
    ));
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn text_plain() {
    let ops = load_and_lower_ops(include_str!("../../../tests/fixtures/text_plain.yaml"));
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn octet_stream() {
    let ops = load_and_lower_ops(include_str!("../../../tests/fixtures/octet_stream.yaml"));
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn form_urlencoded() {
    let ops = load_and_lower_ops(include_str!("../../../tests/fixtures/form_urlencoded.yaml"));
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn multipart() {
    let ops = load_and_lower_ops(include_str!("../../../tests/fixtures/multipart.yaml"));
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn mixed_content_types() {
    let ops = load_and_lower_ops(include_str!(
        "../../../tests/fixtures/mixed_content_types.yaml"
    ));
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn unsupported_request_body_media_type() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Test
  version: "0.1.0"
paths:
  /test:
    post:
      operationId: testOp
      requestBody:
        required: true
        content:
          application/xml:
            schema:
              type: string
      responses:
        "200":
          description: OK
"#;
    let spec = oas3::from_yaml(yaml).unwrap();
    let err = lower_operations(&spec).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unsupported media type") && msg.contains("application/xml"),
        "expected UnsupportedRequestBodyMediaType with xml, got: {msg}"
    );
}

#[test]
fn missing_operation_id() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Test
  version: "0.1.0"
paths:
  /test:
    get:
      responses:
        "200":
          description: OK
"#;
    let spec = oas3::from_yaml(yaml).unwrap();
    let err = lower_operations(&spec).unwrap_err();
    assert!(
        err.to_string().contains("missing operationId"),
        "expected MissingOperationId error, got: {err}"
    );
}
