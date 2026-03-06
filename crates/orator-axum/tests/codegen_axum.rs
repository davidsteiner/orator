use orator_axum::codegen::{Config, generate, generate_axum_handlers};
use orator_core::lower::{lower_operations, lower_schemas};

fn generate_axum_from_yaml(yaml: &str, default_tag: &str) -> String {
    let spec = oas3::from_yaml(yaml).unwrap();
    let ops = lower_operations(&spec).unwrap();
    generate_axum_handlers(&ops, default_tag, &Config::default())
}

#[test]
fn tennis_club_axum_handlers() {
    let code = generate_axum_from_yaml(
        include_str!("../../../examples/tennis-club/tennis-club.yaml"),
        "TennisClub",
    );
    insta::assert_snapshot!(code);
}

#[test]
fn tennis_club_generated_module() {
    let yaml = include_str!("../../../examples/tennis-club/tennis-club.yaml");
    let spec = oas3::from_yaml(yaml).unwrap();
    let types = lower_schemas(&spec).unwrap();
    let ops = lower_operations(&spec).unwrap();

    let module = generate(&types, &ops, &spec.info.title, &Config::default());

    insta::assert_snapshot!("module_types", module.types);
    insta::assert_snapshot!("module_operations", module.operations);
    insta::assert_snapshot!("module_handlers", module.handlers);
    insta::assert_snapshot!("module_build_rs_entry", module.build_rs_entry());
    insta::assert_snapshot!("module_mod_file", module.mod_file());
}
