---
title: CLI reference
description: Command-line options for the Orator CLI.
---

## `orator generate`

The main (and currently only) command. Reads an OpenAPI 3.1 spec and writes generated Rust code.

### Options

| Flag | Description |
|------|-------------|
| `--input`, `-i` | Path to the OpenAPI spec file (YAML or JSON) |
| `--output`, `-o` | Directory to write generated code into |
| `--framework` | Target framework. Currently only `axum` is supported |

### Example

```bash
orator generate -i api.yaml -o src/generated --framework axum
```