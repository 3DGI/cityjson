# Ecosystem Overview

This page is the short map of the publishable CityJSON Rust stack.

## Core Pieces

- `cityjson-rs`: semantic CityJSON model and invariants
- `serde_cityjson`: JSON and JSONL boundary
- `cityjson_lib`: publishable Rust facade
- `cjindex`: indexed dataset queries
- `cjfake`: synthetic data generation
- `cityjson-benchmarks`: shared benchmark corpus and workload contract

## Dependency Shape

```text
cjfake
  -> cityjson_lib
  -> { serde_cityjson }
  -> cityjson-rs

cjindex
  -> cityjson_lib
  -> { serde_cityjson }
  -> cityjson-rs
```

## Start Here

- use `cityjson_lib` if you want the normal Rust entry point
- use `cityjson-rs` if you want the semantic model directly
- use `serde_cityjson` if you need JSON or JSONL control
- use `cjindex`, `cjfake`, or `cityjson-benchmarks` for tooling, fixtures, or benchmark work

Transport experiments are kept outside the publishable core line.
