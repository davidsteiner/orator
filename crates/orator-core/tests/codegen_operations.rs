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

#[test]
fn text_plain_operations() {
    let code = generate_ops_from_yaml(
        include_str!("../../../tests/fixtures/text_plain.yaml"),
        "Default",
    );
    insta::assert_snapshot!(code);
}

#[test]
fn octet_stream_operations() {
    let code = generate_ops_from_yaml(
        include_str!("../../../tests/fixtures/octet_stream.yaml"),
        "Default",
    );
    insta::assert_snapshot!(code);
}

#[test]
fn form_urlencoded_operations() {
    let code = generate_ops_from_yaml(
        include_str!("../../../tests/fixtures/form_urlencoded.yaml"),
        "Default",
    );
    insta::assert_snapshot!(code);
}

#[test]
fn multipart_operations() {
    let code = generate_ops_from_yaml(
        include_str!("../../../tests/fixtures/multipart.yaml"),
        "Default",
    );
    insta::assert_snapshot!(code);
}

#[test]
fn mixed_content_types_operations() {
    let code = generate_ops_from_yaml(
        include_str!("../../../tests/fixtures/mixed_content_types.yaml"),
        "Default",
    );
    insta::assert_snapshot!(code);
}
