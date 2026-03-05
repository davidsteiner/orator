use orator_core::ir::OperationIr;
use orator_core::lower::lower_operations;

fn load_and_lower_ops(yaml: &str) -> Vec<OperationIr> {
    let spec = oas3::from_yaml(yaml).unwrap();
    lower_operations(&spec).unwrap()
}

#[test]
fn tennis_club_operations() {
    let ops = load_and_lower_ops(include_str!(
        "../../../examples/tennis-club/tennis-club.yaml"
    ));
    insta::assert_debug_snapshot!(ops);
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
