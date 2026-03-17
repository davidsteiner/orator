use orator_core::codegen::{Config, generate_operations};
use orator_core::lower::lower_operations;

fn generate_ops_from_yaml(yaml: &str, default_tag: &str) -> String {
    let spec = oas3::from_yaml(yaml).unwrap();
    let ops = lower_operations(&spec).unwrap();
    generate_operations(&ops, default_tag, &Config::default())
}

#[test]
fn tennis_club_operations() {
    let code = generate_ops_from_yaml(
        include_str!("../../../examples/tennis-club-core/tennis-club.yaml"),
        "TennisClub",
    );
    insta::assert_snapshot!(code);
}
