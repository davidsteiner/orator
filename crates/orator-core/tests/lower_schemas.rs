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

#[test]
fn oneof_nullable() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_oneof_nullable.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn dollar_prefix() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_dollar_prefix.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn datetime() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_datetime.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn uuid() {
    let types = load_and_lower(include_str!("../../../tests/fixtures/schemas_uuid.yaml"));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn recursive_types() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_recursive.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn prefix_items() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_prefix_items.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn inline_objects() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_inline_objects.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn oneof_mapping() {
    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_oneof_mapping.yaml"
    ));
    insta::assert_debug_snapshot!(types);
}

#[test]
fn oneof_mapping_assertions() {
    use orator_core::ir::{EnumDef, TypeDefKind, TypeRef};

    let types = load_and_lower(include_str!(
        "../../../tests/fixtures/schemas_oneof_mapping.yaml"
    ));

    let enum_of = |name: &str| -> EnumDef {
        let td = types.iter().find(|t| t.name == name).expect("type missing");
        match &td.kind {
            TypeDefKind::Enum(e) => e.clone(),
            other => panic!("{name} is not an enum: {other:?}"),
        }
    };

    // Many-to-one: four variants, one per mapping key, all but one wrap PolyominoBody.
    let puzzle = enum_of("Puzzle");
    let puzzle_mapping_values: Vec<_> = puzzle
        .variants
        .iter()
        .map(|v| v.mapping_value.as_deref().unwrap_or("<none>"))
        .collect();
    assert_eq!(
        puzzle_mapping_values,
        vec!["hexa-blocks", "quatro-blocks", "quatro-legacy", "trio-mino"],
        "Puzzle variants should follow BTreeMap key order from the mapping",
    );
    let puzzle_targets: Vec<_> = puzzle
        .variants
        .iter()
        .map(|v| match &v.type_ref {
            TypeRef::Named(n) => n.as_str(),
            other => panic!("unexpected type_ref: {other:?}"),
        })
        .collect();
    assert_eq!(
        puzzle_targets,
        vec![
            "PolyominoBody",
            "PolyominoBody",
            "RegionBalanceBody",
            "PolyominoBody"
        ],
    );

    // One-to-one with renames: both keys differ from schema names.
    let pet = enum_of("Pet");
    let pet_pairs: Vec<_> = pet
        .variants
        .iter()
        .map(|v| {
            let TypeRef::Named(n) = &v.type_ref else {
                panic!()
            };
            (v.mapping_value.as_deref().unwrap_or("<none>"), n.as_str())
        })
        .collect();
    assert_eq!(pet_pairs, vec![("kitty", "Cat"), ("puppy", "Dog")],);

    // Partial mapping: Circle has a mapped tag, Square falls back to schema name.
    let shape = enum_of("Shape");
    let shape_pairs: Vec<_> = shape
        .variants
        .iter()
        .map(|v| {
            let TypeRef::Named(n) = &v.type_ref else {
                panic!()
            };
            (v.mapping_value.as_deref(), n.as_str())
        })
        .collect();
    assert_eq!(
        shape_pairs,
        vec![(Some("round-thing"), "Circle"), (None, "Square")],
        "mapped branches come first (BTreeMap order); unmapped branches preserve oneOf order",
    );

    // Unused mapping key (target is not in the oneOf list) is silently dropped;
    // the matching oneOf entry for Apple falls back to schema-name behaviour.
    let fruit = enum_of("Fruit");
    let fruit_pairs: Vec<_> = fruit
        .variants
        .iter()
        .map(|v| {
            let TypeRef::Named(n) = &v.type_ref else {
                panic!()
            };
            (v.mapping_value.as_deref(), n.as_str())
        })
        .collect();
    assert_eq!(
        fruit_pairs,
        vec![(Some("red-apple"), "Apple"), (None, "Banana")],
    );

    // Collision: both keys should appear in the IR even though they PascalCase
    // to the same identifier; deduping is a codegen concern.
    let paint = enum_of("Paint");
    let paint_values: Vec<_> = paint
        .variants
        .iter()
        .map(|v| v.mapping_value.as_deref().unwrap_or("<none>"))
        .collect();
    assert_eq!(
        paint_values,
        vec!["Red", "red"],
        "BTreeMap order is uppercase before lowercase"
    );
}
