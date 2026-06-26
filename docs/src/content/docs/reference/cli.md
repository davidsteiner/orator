---
title: CLI reference
description: Command-line options for the Orator CLI.
---

## `orator`

Reads an OpenAPI 3.1 spec and writes generated Rust code (axum) into an output directory.

### Usage

```bash
orator <SPEC> --output <DIR>
```

### Arguments & options

| Argument / Flag | Description |
|------|-------------|
| `<SPEC>` | Path to the OpenAPI spec file (YAML or JSON). Positional. |
| `--output`, `-o` | Directory to write generated code into. Required. |
| `--no-header-params` | Disable header parameter extraction. |
| `--no-cookie-params` | Disable cookie parameter extraction. |

### Example

```bash
orator openapi.yaml --output src/api/generated
```