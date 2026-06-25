---
title: Response headers
description: How OpenAPI response headers map to generated Rust.
---

Response headers declared on a response in the spec are surfaced on a generated
`…Headers` struct, one per response variant that declares headers, with one field
per header. The handler sets the values it wants returned; the generated
`IntoResponse` impl writes them onto the response.

## Supported value schemas

| Header schema | Generated field | Serialization |
| --- | --- | --- |
| Scalar (string, integer, number, boolean, date, date-time, UUID) | `T` (or `Option<T>` if not required) | the value's text form |
| Array of scalars | `Vec<T>` (or `Option<Vec<T>>`) | OpenAPI `simple` style, comma-joined (`X-Ids: 1,2,3`) |

## Not yet supported

Object and map (`additionalProperties`) header values are rejected at generation
time with a clear error. Support for them is tracked in
[#122](https://github.com/davidsteiner/orator/issues/122).

If you need a structured value in a single header today, declare the header as
`type: string` and format the payload (for example, JSON) in your handler.
