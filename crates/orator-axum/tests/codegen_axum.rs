use orator_axum::codegen::generate_axum_handlers;
use orator_core::lower::lower_operations;

fn generate_axum_from_yaml(yaml: &str, default_tag: &str) -> String {
    let spec = oas3::from_yaml(yaml).unwrap();
    let ops = lower_operations(&spec).unwrap();
    generate_axum_handlers(&ops, default_tag)
}

#[test]
fn tennis_club_axum_handlers() {
    let code = generate_axum_from_yaml(
        include_str!("../../../examples/tennis-club/tennis-club.yaml"),
        "TennisClub",
    );
    insta::assert_snapshot!(code);
}
