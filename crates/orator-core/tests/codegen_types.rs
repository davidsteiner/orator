use orator_core::codegen::generate_types;
use orator_core::lower::lower_schemas;

fn generate_from_yaml(yaml: &str) -> String {
    let spec = oas3::from_yaml(yaml).unwrap();
    let types = lower_schemas(&spec).unwrap();
    generate_types(&types)
}

#[test]
fn basic_object() {
    let code = generate_from_yaml(include_str!("../../../tests/fixtures/schemas_basic.yaml"));
    insta::assert_snapshot!(code);
}

#[test]
fn allof_composition() {
    let code = generate_from_yaml(include_str!("../../../tests/fixtures/schemas_allof.yaml"));
    insta::assert_snapshot!(code);
}

#[test]
fn oneof_discriminator() {
    let code = generate_from_yaml(include_str!("../../../tests/fixtures/schemas_oneof.yaml"));
    insta::assert_snapshot!(code);
}

#[test]
fn anyof() {
    let code = generate_from_yaml(include_str!("../../../tests/fixtures/schemas_anyof.yaml"));
    insta::assert_snapshot!(code);
}

#[test]
fn string_enum() {
    let code = generate_from_yaml(include_str!("../../../tests/fixtures/schemas_enum.yaml"));
    insta::assert_snapshot!(code);
}

#[test]
fn nullable_fields() {
    let code = generate_from_yaml(include_str!(
        "../../../tests/fixtures/schemas_nullable.yaml"
    ));
    insta::assert_snapshot!(code);
}

#[test]
fn arrays() {
    let code = generate_from_yaml(include_str!("../../../tests/fixtures/schemas_arrays.yaml"));
    insta::assert_snapshot!(code);
}

#[test]
fn petstore() {
    let code = generate_from_yaml(include_str!("../../../tests/fixtures/petstore.yaml"));
    insta::assert_snapshot!(code);
}

#[test]
fn additional_properties() {
    let code = generate_from_yaml(include_str!(
        "../../../tests/fixtures/schemas_additional_properties.yaml"
    ));
    insta::assert_snapshot!(code);
}

#[test]
fn json_value() {
    let code = generate_from_yaml(include_str!(
        "../../../tests/fixtures/schemas_json_value.yaml"
    ));
    insta::assert_snapshot!(code);
}

#[test]
fn oneof_nullable() {
    let code = generate_from_yaml(include_str!(
        "../../../tests/fixtures/schemas_oneof_nullable.yaml"
    ));
    insta::assert_snapshot!(code);
}

#[test]
fn dollar_prefix() {
    let code = generate_from_yaml(include_str!(
        "../../../tests/fixtures/schemas_dollar_prefix.yaml"
    ));
    insta::assert_snapshot!(code);
}

#[test]
fn datetime() {
    let code = generate_from_yaml(include_str!(
        "../../../tests/fixtures/schemas_datetime.yaml"
    ));
    insta::assert_snapshot!(code);
}
