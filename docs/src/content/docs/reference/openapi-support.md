---
title: OpenAPI feature support
description: Which OpenAPI 3.1 features orator supports, has planned, or does not support.
---

Orator targets **OpenAPI 3.1** (JSON Schema 2020-12). OpenAPI 3.0 and 2.0 (Swagger)
are not supported.

This page lists what orator supports today, what is planned, and what is currently
unsupported. It reflects the current release and may lag the latest `main`.

:::tip[Missing something?]
If a feature your use case needs is listed as **Unsupported** (so we can reconsider)
or **Planned** (so we can prioritise it), please
[open an issue](https://github.com/davidsteiner/orator/issues/new). Real use cases
drive what we build next.
:::

## Supported

### Schemas

A subset of JSON Schema 2020-12:

- Objects, including `additionalProperties: false` (deny unknown fields).
- Arrays, and tuples via `prefixItems` (requires `minItems` and `maxItems` both equal
  to the number of `prefixItems`).
- String enums.
- `oneOf`, `anyOf`, and `allOf`.
- `discriminator`, with `mapping`.
- Maps via `additionalProperties`.
- Nullable types, both `["T", "null"]` and a `oneOf` with a null branch.
- Inline objects in component schemas (promoted to named types).
- Recursive types (cycles are detected and boxed automatically).
- `$ref` references.

Supported formats: `date`, `date-time`, `uuid`, `int32`, `int64`, `float` (numbers
default to `f64`).

Known issues with `discriminator`:
[#116](https://github.com/davidsteiner/orator/issues/116) (a discriminator on an
`allOf` base loses union semantics) and
[#115](https://github.com/davidsteiner/orator/issues/115) (a `oneOf`/`anyOf`
discriminator duplicates the tag field in the generated struct).

### Request bodies

- `application/json`
- `text/plain`
- `application/octet-stream`
- `application/x-www-form-urlencoded`
- `multipart/form-data`

### Response bodies

- `application/json`
- `text/plain`
- `application/octet-stream`

### Parameters

- Path, query, header, and cookie parameters.
- Required and optional parameters, with descriptions.
- Header and cookie parameter extraction can be toggled with CLI flags.

### Response headers

Scalar values and arrays of scalars, serialized in OpenAPI `simple` style
(comma-joined). See [Response headers](../response-headers/) for details.

### Operations

- All HTTP methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS, TRACE.
- Per-status responses and the `default` response.
- Tags (the first tag groups an operation into a trait).
- Operation summary and description.

## Planned

These are tracked and intended; follow or comment on the issue to help us prioritise.

- Security scheme types and extractors —
  [#88](https://github.com/davidsteiner/orator/issues/88)
- Request validation from schema constraints (`maxLength`, `pattern`, `minimum`, …) —
  [#94](https://github.com/davidsteiner/orator/issues/94)
- Operation filtering (include/exclude by tag) —
  [#93](https://github.com/davidsteiner/orator/issues/93)
- `#[deprecated]` for deprecated operations, parameters, and schemas —
  [#90](https://github.com/davidsteiner/orator/issues/90)
- IP-address formats (`ipv4`/`ipv6`) —
  [#24](https://github.com/davidsteiner/orator/issues/24)
- Configurable derives on generated types —
  [#38](https://github.com/davidsteiner/orator/issues/38)
- Map and object response headers —
  [#122](https://github.com/davidsteiner/orator/issues/122)
- Generation-time warnings for ignored or unsupported features —
  [#37](https://github.com/davidsteiner/orator/issues/37)

## Unsupported

Each item notes how orator behaves when it encounters the feature: it either fails with
an **error** at generation time, or **silently ignores** it.

| Feature | Behavior |
| --- | --- |
| `servers` | Silently ignored — not applicable to a server-stub generator (orator does not generate clients or base URLs). |
| `webhooks` | Silently ignored. |
| `callbacks` | Silently ignored. |
| Response `links` | Silently ignored. |
| `examples` (schema, parameter, request/response) | Silently ignored. |
| Parameter `style` / `explode` (beyond the implicit `simple` handling) | Silently ignored. |
| `const` and `not` schema keywords | Silently ignored. |
| Schema validation constraints (`maxLength`, `pattern`, `minimum`, …) | Silently ignored (see Planned, [#94](https://github.com/davidsteiner/orator/issues/94)). |
| Response body content types other than JSON, text, or octet-stream | **Silently dropped** — the response variant is generated with no body. |
| Request body content types other than the supported five | **Error.** |
| Object or map response headers | **Error** (see Planned, [#122](https://github.com/davidsteiner/orator/issues/122)). |
| Inline objects used directly in a request or response body | **Error** — use a `$ref` to a named schema instead. |
| `multipart/form-data` with a `$ref` schema | **Error** — inline the schema instead. |
| Missing `operationId` on an operation | **Error** — `operationId` is required. |

## Found a gap?

Orator only generates what it can derive from the spec, and the lists above will grow.
If something your project needs is missing — whether it is **Planned** and you want it
sooner, or **Unsupported** and you think it should be reconsidered — please
[open an issue](https://github.com/davidsteiner/orator/issues/new) describing your use
case. For visibility into features that are silently ignored today, see
[#37](https://github.com/davidsteiner/orator/issues/37).
