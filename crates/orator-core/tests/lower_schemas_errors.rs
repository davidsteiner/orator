use orator_core::lower::lower_schemas;

#[test]
fn prefix_items_without_bounds_errors() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Bad
  version: "1.0.0"
paths: {}
components:
  schemas:
    Loose:
      type: array
      prefixItems:
        - type: integer
        - type: integer
"#;
    let spec = oas3::from_yaml(yaml).unwrap();
    let err = lower_schemas(&spec).expect_err("loose prefixItems must error");
    let msg = err.to_string();
    assert!(
        msg.contains("prefixItems") && msg.contains("minItems") && msg.contains("maxItems"),
        "error message should mention prefixItems and the bounds requirement, got: {msg}"
    );
}

#[test]
fn prefix_items_with_mismatched_bounds_errors() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Bad
  version: "1.0.0"
paths: {}
components:
  schemas:
    Mismatched:
      type: array
      prefixItems:
        - type: integer
        - type: integer
      minItems: 2
      maxItems: 5
"#;
    let spec = oas3::from_yaml(yaml).unwrap();
    let err = lower_schemas(&spec).expect_err("mismatched bounds must error");
    assert!(err.to_string().contains("prefixItems"));
}
