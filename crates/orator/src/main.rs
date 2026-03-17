use std::fs;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use orator_axum_codegen::codegen::{Config, generate};
use orator_core::lower::{lower_operations, lower_schemas};

/// Orator — generate Rust server stubs from OpenAPI 3.1 specs
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the OpenAPI spec file (YAML or JSON)
    spec: PathBuf,

    /// Output directory for generated files
    #[arg(short, long)]
    output: PathBuf,

    /// Disable header parameter extraction
    #[arg(long)]
    no_header_params: bool,

    /// Disable cookie parameter extraction
    #[arg(long)]
    no_cookie_params: bool,
}

fn main() {
    let cli = Cli::parse();

    let yaml = match fs::read_to_string(&cli.spec) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "error: failed to read spec file '{}': {e}",
                cli.spec.display()
            );
            process::exit(1);
        }
    };

    let spec = match oas3::from_yaml(&yaml) {
        Ok(spec) => spec,
        Err(e) => {
            eprintln!("error: failed to parse OpenAPI spec: {e}");
            process::exit(1);
        }
    };

    let types = match lower_schemas(&spec) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: failed to lower schemas: {e}");
            process::exit(1);
        }
    };

    let ops = match lower_operations(&spec) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: failed to lower operations: {e}");
            process::exit(1);
        }
    };

    let config = Config::default()
        .header_params(!cli.no_header_params)
        .cookie_params(!cli.no_cookie_params);

    let module = generate(&types, &ops, &spec.info.title, &config);

    if let Err(e) = module.write_to_dir(&cli.output) {
        eprintln!("error: failed to write generated files: {e}");
        process::exit(1);
    }

    println!("Generated files written to {}", cli.output.display());
}
