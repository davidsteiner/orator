# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/davidsteiner/orator/compare/orator-axum-v0.2.1...orator-axum-v0.3.0) - 2026-03-19

### Added

- support UUID fields ([#75](https://github.com/davidsteiner/orator/pull/75))
- support date and datetime fields ([#74](https://github.com/davidsteiner/orator/pull/74))
- support multipart form data request body schemas ([#73](https://github.com/davidsteiner/orator/pull/73))

## [0.2.1](https://github.com/davidsteiner/orator/compare/orator-axum-v0.2.0...orator-axum-v0.2.1) - 2026-03-19

### Added

- support empty/untyped schemas as serde_json::Value ([#69](https://github.com/davidsteiner/orator/pull/69))

## [0.2.0](https://github.com/davidsteiner/orator/compare/orator-axum-v0.1.2...orator-axum-v0.2.0) - 2026-03-19

### Added

- make ParamRejection better typed ([#57](https://github.com/davidsteiner/orator/pull/57))

### Other

- revised content for documentation landing page ([#56](https://github.com/davidsteiner/orator/pull/56))

## [0.1.2](https://github.com/davidsteiner/orator/compare/orator-axum-v0.1.1...orator-axum-v0.1.2) - 2026-03-18

### Added

- refine default responses to support custom types and status codes ([#50](https://github.com/davidsteiner/orator/pull/50))

## [0.1.1](https://github.com/davidsteiner/orator/compare/orator-axum-v0.1.0...orator-axum-v0.1.1) - 2026-03-17

### Other

- release v0.1.0 ([#47](https://github.com/davidsteiner/orator/pull/47))

## [0.1.0](https://github.com/davidsteiner/orator/compare/orator-axum-v0.0.4...orator-axum-v0.1.0) - 2026-03-17

### Other

- bump version to 0.1.0 ([#48](https://github.com/davidsteiner/orator/pull/48))

## [0.0.3](https://github.com/davidsteiner/orator/compare/orator-axum-v0.0.2...orator-axum-v0.0.3) - 2026-03-17

### Added

- support major non-json content types ([#33](https://github.com/davidsteiner/orator/pull/33))

## [0.0.2](https://github.com/davidsteiner/orator/compare/orator-axum-v0.0.1...orator-axum-v0.0.2) - 2026-03-17

### Added

- remove support for implicitly built modules using build.rs ([#29](https://github.com/davidsteiner/orator/pull/29))
- create CLI to generate files into the users project ([#26](https://github.com/davidsteiner/orator/pull/26))
- opt-in extraction of cookie params ([#25](https://github.com/davidsteiner/orator/pull/25))
- add typesafe builder for full API using typestate pattern ([#20](https://github.com/davidsteiner/orator/pull/20))
- create opt-in extraction of header params ([#18](https://github.com/davidsteiner/orator/pull/18))
- add description to emitted types ([#10](https://github.com/davidsteiner/orator/pull/10))
- implement query params ([#9](https://github.com/davidsteiner/orator/pull/9))
- improve path parameter handling ([#8](https://github.com/davidsteiner/orator/pull/8))
- more interesting types in example app ([#7](https://github.com/davidsteiner/orator/pull/7))
- add code generation for routes in axum ([#5](https://github.com/davidsteiner/orator/pull/5))
