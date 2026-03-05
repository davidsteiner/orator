use std::fs;
use std::path::Path;

use orator_axum::codegen::generate;
use orator_core::lower::{lower_operations, lower_schemas};

fn main() {
    let spec_path = Path::new("tennis-club.yaml");
    println!("cargo::rerun-if-changed={}", spec_path.display());

    let yaml = fs::read_to_string(spec_path).expect("failed to read OpenAPI spec");
    let spec = oas3::from_yaml(&yaml).expect("failed to parse OpenAPI spec");

    let types = lower_schemas(&spec).expect("failed to lower schemas");
    let ops = lower_operations(&spec).expect("failed to lower operations");

    let module = generate(&types, &ops, &spec.info.title);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out = Path::new(&out_dir);

    let api_dir = out.join("api");
    fs::create_dir_all(&api_dir).expect("failed to create api directory");
    fs::write(api_dir.join("types.rs"), &module.types).expect("failed to write types");
    fs::write(api_dir.join("operations.rs"), &module.operations)
        .expect("failed to write operations");
    fs::write(api_dir.join("handlers.rs"), &module.handlers).expect("failed to write handlers");
    fs::write(out.join("api.rs"), module.build_rs_entry()).expect("failed to write api entry");
}
