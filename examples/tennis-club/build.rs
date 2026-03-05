use std::fs;
use std::path::Path;

use orator_core::codegen::generate_types;
use orator_core::lower::lower_schemas;

fn main() {
    let spec_path = Path::new("tennis-club.yaml");
    println!("cargo::rerun-if-changed={}", spec_path.display());

    let yaml = fs::read_to_string(spec_path).expect("failed to read OpenAPI spec");
    let spec = oas3::from_yaml(&yaml).expect("failed to parse OpenAPI spec");
    let types = lower_schemas(&spec).expect("failed to lower schemas");
    let code = generate_types(&types);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("types.rs");
    fs::write(&dest, code).expect("failed to write generated types");
}
