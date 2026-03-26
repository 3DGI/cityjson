# Architecture

This document synthesizes the current architecture direction across
`cityjson-rs`, `serde_cityjson`, `cjlib`, and `cjfake`.

The goal is to keep the ecosystem clean, explicit, and maintainable as more
formats land.

## Core Decisions

The architecture is built on five rules:

1. There is exactly one semantic model: `cityjson::v2_0::OwnedCityModel`.
2. There is exactly one semantic interchange unit: a self-contained
   `OwnedCityModel`.
3. `cjlib::CityModel` is the stable owned facade over that one semantic model.
4. JSON, JSONL, Arrow, Parquet, and future transports are format boundaries,
   not semantic model families.
5. Raw, staged, or lazy read paths are explicit advanced APIs and must not
   distort the default owned facade.

## One Semantic Model

The same semantic type should represent all of these cases:

- a full CityJSON document
- a tile or grouped subset
- a single feature-sized package

The difference between those values is only:

- scope
- size
- provenance

It is not a type-level semantic distinction.

That means the ecosystem should not introduce:

- a separate object-package model
- a separate partial-model semantic type
- a format-specific semantic unit in `cjlib`

## Self-contained Does Not Mean Inline

A self-contained model must carry everything needed for independent processing:

- cityobjects
- referenced vertices
- templates and template vertices
- appearance resources
- semantics
- metadata and extras
- enough provenance to merge or trace later

It does not need to inline everything into every geometry.

The preferred compromise is:

- keep pooled storage
- keep indexed references
- scope the pools to the current `OwnedCityModel`

That lets the ecosystem support split, merge, stream, and regroup workflows
without abandoning the storage discipline of the semantic model.

## Layering

The intended dependency and responsibility flow is:

```text
cjfake
  -> cjlib
  -> { serde_cityjson, cityarrow, cityparquet }
  -> cityjson-rs
```

With responsibilities split like this:

- `cityjson-rs`
  - owns the one semantic model
  - owns invariants, validation, and semantic operations
  - owns extraction, localization, remapping, and merge of self-contained
    submodels
- `serde_cityjson`
  - owns the CityJSON JSON and JSONL wire format
  - owns document parsing, feature parsing, stream parsing, and serialization
  - owns staged and raw JSON boundary work
- `cityarrow` / `cityparquet`
  - own Arrow and Parquet boundary representations
  - convert between transport-native representations and `OwnedCityModel`
- `cjlib`
  - owns the stable user-facing facade
  - owns convenience constructors, error shaping, and explicit format modules
  - stays format-neutral at the semantic level
- `cjfake`
  - stays above the facade as a generator and test-data producer
  - emits models and formats by using `cjlib`

## API Consequences For `cjlib`

`cjlib` should expose one semantic wrapper type:

- `cjlib::CityModel`

That wrapper should work for any valid semantic scope:

- whole document
- grouped subset
- single-feature package

The top-level convenience path should stay small and single-model oriented:

- `CityModel::from_slice`
- `CityModel::from_file`

Everything format-specific should live in explicit modules such as:

- `cjlib::json`
- `cjlib::arrow`
- `cjlib::parquet`

Those modules should speak in terms of:

- one `CityModel`
- or a stream of `CityModel` values

They should not surface format-specific semantic types such as:

- `CityJSONFeature`
- Arrow batches as conceptual application objects
- Parquet row groups as semantic units

Those are wire-format details, not the semantic architecture.

## JSON Boundary Direction

The JSON boundary should support both:

- full-document materialization into one `CityModel`
- feature-sized materialization into one `CityModel`
- stream reading as a sequence of `CityModel` values
- strict stream aggregation as an explicit helper when the caller wants to
  rebuild one larger model

That keeps `CityJSONFeature` in its correct place:

- a JSON wire-format construct
- not the conceptual unit of the whole ecosystem

## Raw And Staged Paths

The default `cjlib::CityModel` path should remain:

- owned
- normalized
- stable
- manipulation-friendly

If higher-performance read paths become important, they should be added
explicitly, for example under `cjlib::json`, as raw or staged APIs.

The architecture should not leak borrowed lifetimes or raw-backend concerns
into the default `CityModel` facade.

## What Belongs In `cityjson-rs`

The semantic model crate should eventually own the core operations needed for
package-based processing:

- extract a self-contained submodel
- localize shared resources into a submodel
- merge a self-contained submodel back into a larger model
- assemble a larger model from many smaller models

Those are semantic model concerns.
They should not be reinvented in format crates or hidden as ad hoc logic in
`cjlib`.

## What Belongs In `cjlib::ops`

`cjlib::ops` should stay above the semantic core.
It can provide workflow-oriented helpers, but it should delegate correctness and
merge semantics to `cityjson-rs`.

Good candidates include:

- selection helpers
- LoD filtering
- cleanup workflows
- geometry measurements
- texture-path rewriting
- version-upgrade workflows

## Non-goals

The intended architecture should not:

- reintroduce a second in-memory CityJSON model in `cjlib`
- define a separate semantic package type beside `OwnedCityModel`
- make borrowed-string storage the main architectural axis
- hide format choice behind a generic registry
- force every format to share one universal read representation

The right abstraction is:

- one semantic model
- many format boundaries
- explicit conversions between them
