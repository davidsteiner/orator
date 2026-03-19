use orator_core::lower::lower_schemas;

fn load_and_lower(yaml: &str) -> Vec<orator_core::ir::TypeDef> {
    let spec = oas3::from_yaml(yaml).unwrap();
    lower_schemas(&spec).unwrap()
}

#[test]
fn basic_object() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/schemas_basic.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn allof_composition() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/schemas_allof.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn oneof_discriminator() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/schemas_oneof.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn anyof() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/schemas_anyof.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn string_enum() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/schemas_enum.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn nullable_fields() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_nullable.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn arrays() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/schemas_arrays.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn petstore() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/petstore.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn additional_properties() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_additional_properties.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn json_value() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_json_value.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}
