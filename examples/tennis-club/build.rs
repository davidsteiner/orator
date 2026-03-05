use std::fs;
use std::path::Path;

use orator_core::codegen::{generate_operations, generate_types};
use orator_core::lower::{lower_operations, lower_schemas};

fn main() {
    let spec_path = Path::new("tennis-club.yaml");
    println!("cargo::rerun-if-changed={}", spec_path.display());

    let yaml = fs::read_to_string(spec_path).expect("failed to read OpenAPI spec");
    let spec = oas3::from_yaml(&yaml).expect("failed to parse OpenAPI spec");

    let types = lower_schemas(&spec).expect("failed to lower schemas");
    let types_code = generate_types(&types);

    let ops = lower_operations(&spec).expect("failed to lower operations");
    let ops_code = generate_operations(&ops, &spec.info.title);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out = Path::new(&out_dir);
    fs::write(out.join("types.rs"), types_code).expect("failed to write generated types");
    fs::write(out.join("operations.rs"), ops_code).expect("failed to write generated operations");
}
