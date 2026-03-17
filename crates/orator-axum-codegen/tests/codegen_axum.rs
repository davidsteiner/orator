use orator_axum_codegen::codegen::{Config, generate, generate_axum_handlers};
use orator_core::lower::{lower_operations, lower_schemas};

fn generate_axum_from_yaml(yaml: &str, default_tag: &str) -> String {
    generate_axum_from_yaml_with_config(yaml, default_tag, &Config::default())
}

fn generate_axum_from_yaml_with_config(yaml: &str, default_tag: &str, config: &Config) -> String {
    let spec = oas3::from_yaml(yaml).unwrap();
    let ops = lower_operations(&spec).unwrap();
    generate_axum_handlers(&ops, default_tag, config)
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
    insta::assert_snapshot!("module_mod_file", module.mod_file());
}

#[test]
fn header_params_extraction() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Header Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      parameters:
        - name: X-Request-ID
          in: header
          required: true
          schema:
            type: string
        - name: X-Rate-Limit
          in: header
          required: false
          schema:
            type: integer
            format: int32
        - name: X-Trace-Enabled
          in: header
          required: true
          schema:
            type: boolean
      responses:
        "200":
          description: OK
"#;
    let code = generate_axum_from_yaml(yaml, "HeaderTest");
    insta::assert_snapshot!(code);
}

#[test]
fn api_builder_two_tags() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Builder Test
  version: "0.1.0"
paths:
  /users:
    get:
      operationId: listUsers
      tags: [Users]
      responses:
        "200":
          description: OK
  /items:
    get:
      operationId: listItems
      tags: [Items]
      responses:
        "200":
          description: OK
    post:
      operationId: createItem
      tags: [Items]
      responses:
        "201":
          description: Created
"#;
    let code = generate_axum_from_yaml(yaml, "BuilderTest");
    insta::assert_snapshot!(code);
}

#[test]
fn cookie_params_extraction() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Cookie Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      parameters:
        - name: session_id
          in: cookie
          required: true
          schema:
            type: string
        - name: max_results
          in: cookie
          required: false
          schema:
            type: integer
            format: int32
      responses:
        "200":
          description: OK
"#;
    let code = generate_axum_from_yaml(yaml, "CookieTest");
    insta::assert_snapshot!(code);
}

#[test]
fn cookie_params_disabled_by_config() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Cookie Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      parameters:
        - name: session_id
          in: cookie
          required: true
          schema:
            type: string
      responses:
        "200":
          description: OK
"#;
    let config = Config::default().cookie_params(false);
    let code = generate_axum_from_yaml_with_config(yaml, "CookieTest", &config);
    insta::assert_snapshot!(code);
}

#[test]
fn header_params_disabled_by_config() {
    let yaml = r#"
openapi: "3.1.0"
info:
  title: Header Test
  version: "0.1.0"
paths:
  /items:
    get:
      operationId: getItems
      parameters:
        - name: X-Request-ID
          in: header
          required: true
          schema:
            type: string
      responses:
        "200":
          description: OK
"#;
    let config = Config::default().header_params(false);
    let code = generate_axum_from_yaml_with_config(yaml, "HeaderTest", &config);
    insta::assert_snapshot!(code);
}
