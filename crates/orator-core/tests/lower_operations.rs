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
fn response_headers() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Response Headers Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      responses:
        "200":
          description: A list of items
          headers:
            X-Rate-Limit:
              description: Calls left this window
              required: true
              schema:
                type: integer
                format: int32
            X-Request-ID:
              required: false
              schema:
                type: string
            Content-Type:
              schema:
                type: string
          content:
            application/json:
              schema:
                type: array
                items:
                  type: string
"#;
    let ops = load_and_lower_ops(yaml);
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn array_response_header() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Array Header Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      responses:
        "200":
          description: OK
          headers:
            X-Ids:
              required: true
              schema:
                type: array
                items:
                  type: integer
                  format: int64
            X-Tags:
              required: false
              schema:
                type: array
                items:
                  type: string
"#;
    let ops = load_and_lower_ops(yaml);
    insta::assert_debug_snapshot!(ops);
}

#[test]
fn non_scalar_response_header_errors() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      responses:
        "200":
          description: OK
          headers:
            X-Meta:
              schema:
                type: object
                additionalProperties:
                  type: string
"#;
    let spec = oas3::from_yaml(yaml).unwrap();
    let err = lower_operations(&spec).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("X-Meta") && msg.contains("not yet supported"),
        "expected UnsupportedSchema error mentioning X-Meta, got: {msg}"
    );
}

#[test]
fn array_of_non_scalar_response_header_errors() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      responses:
        "200":
          description: OK
          headers:
            X-Items:
              schema:
                type: array
                items:
                  type: object
                  additionalProperties:
                    type: string
"#;
    let spec = oas3::from_yaml(yaml).unwrap();
    let err = lower_operations(&spec).unwrap_err();
    assert!(
        matches!(err, orator_core::error::Error::UnsupportedSchema { .. }),
        "expected UnsupportedSchema for array-of-object header, got: {err:?}"
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
