# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/davidsteiner/orator/compare/orator-core-v0.0.2...orator-core-v0.0.3) - 2026-03-17

### Added

- handle additionalProperties flag correctly ([#42](https://github.com/davidsteiner/orator/pull/42))
- support major non-json content types ([#33](https://github.com/davidsteiner/orator/pull/33))

## [0.0.2](https://github.com/davidsteiner/orator/compare/orator-core-v0.0.1...orator-core-v0.0.2) - 2026-03-17

### Added

- remove support for implicitly built modules using build.rs ([#29](https://github.com/davidsteiner/orator/pull/29))
- opt-in extraction of cookie params ([#25](https://github.com/davidsteiner/orator/pull/25))
- add typesafe builder for full API using typestate pattern ([#20](https://github.com/davidsteiner/orator/pull/20))
- create opt-in extraction of header params ([#18](https://github.com/davidsteiner/orator/pull/18))
- add description to emitted types ([#10](https://github.com/davidsteiner/orator/pull/10))
- implement query params ([#9](https://github.com/davidsteiner/orator/pull/9))
- improve path parameter handling ([#8](https://github.com/davidsteiner/orator/pull/8))
- more interesting types in example app ([#7](https://github.com/davidsteiner/orator/pull/7))
- add code generation for routes in axum ([#5](https://github.com/davidsteiner/orator/pull/5))
- implement code generation for operations ([#4](https://github.com/davidsteiner/orator/pull/4))
- add example app ([#3](https://github.com/davidsteiner/orator/pull/3))
- implement code generation from the intermediate representations for object types ([#2](https://github.com/davidsteiner/orator/pull/2))
- map anyOf and oneOf to enums if the type has no other field ([#1](https://github.com/davidsteiner/orator/pull/1))
