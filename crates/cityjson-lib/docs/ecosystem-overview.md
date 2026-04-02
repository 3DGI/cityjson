# Ecosystem Overview

This page is the plain-language map of the current CityJSON Rust ecosystem.
It is meant for the moment when you land on one repository and need to answer
three questions quickly:

- what does this project do?
- how does it relate to the others?
- where should I start for my use case?

The short version is:

- `cityjson-rs` is the semantic model
- `serde_cityjson` is the JSON and JSONL boundary
- `cjlib` is the user-facing Rust facade
- `cityarrow` and `cityparquet` are non-JSON transport layers
- `cjindex` adds indexing and random-access retrieval
- `cjfake` generates synthetic data for tests and fixtures
- `cjlib-benchmarks` keeps the shared benchmark corpus and workload contract

## The Big Picture

These projects are not seven unrelated experiments.
They are one stack with a deliberate split of responsibilities.

The main idea is simple:

- keep one semantic CityJSON model
- keep wire formats and transport formats explicit
- keep higher-level tooling above the model instead of duplicating it
- keep tests and benchmarks shared across the ecosystem

In dependency terms, the ecosystem currently looks like this:

```text
cjfake
  -> cjlib
  -> { serde_cityjson, cityarrow, cityparquet }
  -> cityjson-rs

cjindex
  -> cjlib
  -> { serde_cityjson, cityarrow, cityparquet }
  -> cityjson-rs

cjlib-benchmarks
  -> shared corpus and workload definitions consumed by the other projects
```

That means each layer has a job:

- `cityjson-rs` answers "what is a valid in-memory CityJSON model?"
- `serde_cityjson` answers "how do JSON and JSONL map to that model?"
- `cityarrow` and `cityparquet` answer "how do Arrow and package-file transports map to that model?"
- `cjlib` answers "what is the ergonomic Rust entry point for users?"
- `cjindex` answers "how do I query large CityJSON datasets efficiently?"
- `cjfake` answers "how do I generate controlled synthetic models for tests?"
- `cjlib-benchmarks` answers "how do all of these projects benchmark against the same corpus?"

## Projects At A Glance

| Project | Link | Main job | Typical user |
| --- | --- | --- | --- |
| `cityjson-rs` | <https://github.com/3DGI/cityjson-rs> | Semantic CityJSON model types, invariants, and correctness-critical model operations | Library authors and advanced Rust users |
| `serde_cityjson` | <https://github.com/3DGI/serde_cityjson> | Parsing, probing, and serializing CityJSON JSON and JSONL | Format-boundary code and lower-level integrations |
| `cjlib` | <https://github.com/3DGI/cjlib> | User-facing facade that ties the model and format crates together | Most Rust users who just want to load, write, and work with models |
| `cityarrow` | <https://github.com/3DGI/cityarrow> | Live Arrow IPC transport for the CityJSON model | Users moving CityJSON data into columnar or Arrow-based workflows |
| `cityparquet` | bundled in the `cityarrow` repository | Persistent single-file package boundary built alongside `cityarrow` | Users who need a file-oriented columnar package format |
| `cjindex` | <https://github.com/3DGI/cjindex> | SQLite-backed indexing and random-access retrieval for CityJSON datasets | Users serving or querying larger local corpora |
| `cjfake` | <https://github.com/3DGI/cjfake> | Synthetic CityJSON generation for tests, fixtures, and workload shaping | Test authors and benchmark authors |
| `cjlib-benchmarks` | <https://github.com/3DGI/cjlib-benchmarks> | Shared benchmark corpus, cases, acquisition rules, and generated artifacts | Maintainers comparing performance and correctness across repos |

## What Each Project Owns

### `cityjson-rs`

Use this project when you care about the actual CityJSON model.

It defines the semantic Rust types for CityJSON 2.0 and is the place where
model invariants and correctness-sensitive operations belong.
If something changes what the model means, or whether it is still valid, it
probably belongs here.

You reach for `cityjson-rs` when:

- you want to build or edit CityJSON models directly in Rust
- you are implementing model-level algorithms
- you need the full semantic API rather than a convenience wrapper

### `serde_cityjson`

Use this project when the problem is specifically JSON or JSONL.

It is the wire-format boundary for CityJSON documents, CityJSONFeature values,
and feature streams.
It should own format probing, parsing, and serialization, rather than forcing
those concerns into the semantic model crate.

You reach for `serde_cityjson` when:

- you are implementing JSON-focused I/O
- you need lower-level control over CityJSON or CityJSONSeq handling
- you are writing tooling that sits at the serialization boundary

### `cjlib`

Use this project when you want the normal Rust entry point.

`cjlib` is the integration layer.
It does not try to replace `cityjson-rs`, but it gives users one small,
predictable place to start: `cjlib::CityModel` for the common document path,
plus explicit modules such as `cjlib::json`, `cjlib::arrow`, and
`cjlib::parquet` when the format matters.

You reach for `cjlib` when:

- you want to read or write CityJSON without assembling the stack yourself
- you want one owned wrapper type for day-to-day Rust use
- you want access to format modules without losing the semantic model below

### `cityarrow` and `cityparquet`

Use these when JSON is not the format you want to move around.

`cityarrow` is the live Arrow IPC transport layer for the CityJSON model.
`cityparquet` is the sibling persistent package format in the same repository.
Both are still about moving the same model across a boundary, not inventing a
separate semantic representation.

You reach for them when:

- you need Arrow-based interchange
- you want a columnar transport path
- you need a persistent package-style file format instead of JSON

### `cjindex`

Use this when your problem is no longer "parse one file" but "query a dataset".

`cjindex` builds and reads a persistent SQLite index over several on-disk
CityJSON layouts.
Its job is random access by feature id or bounding box, not semantic modeling
or generic serialization.

You reach for `cjindex` when:

- you have many tiles or features on disk
- you need bbox or id lookups
- you want dataset inspection, indexing, and validation workflows

### `cjfake`

Use this when you need data, not just code.

`cjfake` generates synthetic CityJSON models with controlled structure and
content.
That makes it useful for tests, conformance fixtures, and benchmarks where
real-world data is too messy, too large, or simply missing the case you need.

You reach for `cjfake` when:

- you need reproducible test data
- you want rare CityObject or geometry combinations
- you want to generate many shaped cases from one manifest

### `cjlib-benchmarks`

Use this when you want one shared benchmark story across the ecosystem.

The repository currently named `cjlib-benchmarks` is the shared corpus and
workload contract.
Its job is to stop each crate from inventing a separate benchmark universe.
It keeps benchmark cases, acquisition metadata, invariants, and derived
artifacts in one place so tools can compare against the same inputs.

You reach for it when:

- you want benchmark cases reused across repositories
- you need a pinned acquisition story for larger public datasets
- you want synthetic and real-world workloads described in one common format

## Common Workflows

### I just want to read and write CityJSON in Rust

Start with `cjlib`.

That gives you the small, user-facing API.
Under the hood it still relies on `serde_cityjson` for the JSON boundary and
`cityjson-rs` for the actual model.

### I am implementing a model algorithm

Start with `cityjson-rs`.

If the algorithm changes or depends on model semantics, that is the correct
layer.
`cjlib` can expose convenience wrappers later, but it should not own the
semantic truth.

### I need lower-level JSON or JSONL control

Start with `serde_cityjson`.

That is the crate that should know how CityJSON documents, features, and
feature streams are parsed and serialized.

### I need a non-JSON transport

Start with `cityarrow` or `cityparquet`, or use `cjlib::arrow` /
`cjlib::parquet` if you want the higher-level facade.

### I need to query a large dataset by id or bbox

Start with `cjindex`.

That is the repo that turns files on disk into something queryable and
repeatable.

### I need synthetic data for tests or benchmarks

Start with `cjfake`.

If you also want those cases to become part of the shared benchmark story, tie
them into `cjlib-benchmarks`.

## How The Pieces Reinforce Each Other

The projects work better together than apart:

- `cityjson-rs` gives the whole ecosystem one semantic center
- `serde_cityjson` keeps JSON concerns out of the model crate
- `cityarrow` and `cityparquet` add other boundaries without creating a second model
- `cjlib` gives users one predictable on-ramp
- `cjindex` builds higher-level dataset retrieval on top of the shared stack
- `cjfake` feeds tests and corpus generation with controlled synthetic data
- `cjlib-benchmarks` gives the ecosystem one common benchmark language

That synergy matters because it avoids a common failure mode:
every tool quietly grows its own partial model, its own fixtures, and its own
benchmark data.

This ecosystem is stronger when the projects stay specialized but compatible.

## Where To Start

If you are new to the ecosystem, start here:

1. `cjlib` if you want to use the stack
2. `cityjson-rs` if you want to understand the semantic core
3. `serde_cityjson` if you care about JSON and JSONL details
4. `cjindex`, `cjfake`, and `cjlib-benchmarks` if you are working on tooling,
   datasets, testing, or benchmarking

If you only remember one rule, remember this:

`cityjson-rs` is the model, `serde_cityjson` is the JSON boundary, and
`cjlib` is the user-facing facade that makes the ecosystem practical to use.
