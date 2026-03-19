---
title: Getting started
description: Install Orator and generate your first server stub.
---

## Installation

Install the CLI from crates.io:

```bash
cargo install orator
```

## Generating code

Point Orator at your OpenAPI 3.1 spec:

```bash
orator --output src/generated spec.yaml
```

This produces a set of Rust modules containing your request/response types, operation traits, and (if using axum) a ready-made router.

## Implementing your API

Orator generates a trait per tag in your spec. You bring your own struct and implement the trait on it — that's where your actual logic lives.

```rust
struct MyApi;

impl PetsApi for MyApi {
    type Error = MyError;

    async fn list_pets(
        &self,
        _ctx: Context,
        params: ListPetsParams,
    ) -> Result<ListPetsResponse, Self::Error> {
        // your logic here
        Ok(ListPetsResponse::Ok(vec![]))
    }
}
```

## What's next?

Have a look at the [tennis club example](https://github.com/davidsteiner/orator/tree/main/examples) for a more complete walkthrough.