# Tennis Club

This is an example application using orator code generation
to build an API.

## Running the application

To run the application, just use `cargo run` or `cargo run -p tennis-club`
from the project's root directory.

You can browse the API endpoints using Scalar hosted at
http://localhost:3000/docs.

## Project structure

Orator code generation is set up in the `build.rs` file
and it's configured to output the API code in `api.rs`
in the output directory of the build.

This code is then included in the `api.rs` module contained
in this crate.
