<div align="center">

# Orator

**Server stub generation from OpenAPI 3.1 specs.**

[![CI](https://github.com/davidsteiner/orator/actions/workflows/ci.yml/badge.svg)](https://github.com/Validus-Risk-Management/hotfix/actions/workflows/ci.yml)
[![codecov](https://codecov.io/github/davidsteiner/orator/graph/badge.svg?token=GM4T49SSDO)](https://codecov.io/github/davidsteiner/orator)
[![crates-badge]](https://crates.io/crates/orator)
[![docs-badge]](https://docs.rs/orator)
[![Crates.io](https://img.shields.io/crates/l/orator)](LICENSE)
[![status](https://img.shields.io/badge/status-experimental-orange)](https://github.com/davidsteiner/orator)

</div>

> [!WARNING]
> This is work in progress — do not use it in production yet.

## Examples

The best way to get a feel for what the crate is capable of
is looking at the tennis club example in `examples/`.

Please refer to the example's README for more detail.

## Features & status

- [x] Code generation of schemas for requests and responses
    - [x] String enums
    - [x] Complex types involving `allOf`, `anyOf` and `oneOf`
    - [x] Optional properties mapped to `Option`
- [x] Code generation for operations
    - [x] Per-tag traits
    - [x] Generic context propagation to handlers for flexibility
- [x] Axum router generation
- [x] Build script integration
- [x] Parameter extraction
    - [x] Path parameters
    - [x] Query parameters
    - [x] Headers
    - [x] Cookie parameter support
- [ ] CLI tool for explicit code generation into your project
- [ ] Default response handling
- [ ] Support for security schemes
- [ ] Support for webhooks

[crates-badge]: https://img.shields.io/crates/v/orator.svg

[docs-badge]: https://docs.rs/orator/badge.svg