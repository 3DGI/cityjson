# Drop Legacy CityJSON Version Support

## Status

Accepted

## Related Commits

- `0cb39ac` Remove v1.0 and v1.1
- `ae9b1e3` Clean up remnants of v1.0 and v1.1

## Context

The crate used to carry API surface and implementation for legacy `CityJSON`
versions in addition to `v2_0`.

That had a cost:

- the crate had to preserve parallel module trees for old versions
- tests and examples had to account for behavior the crate no longer wanted to
  optimize for
- version-specific compatibility logic increased maintenance overhead
- the main public API was less clear because the crate appeared to support
  multiple historical models equally

At the same time, the project direction had shifted toward a focused, modern
`v2_0` API.

## Decision

`cityjson-rs` supports only `CityJSON` `v2_0` as its public in-memory model.

Legacy version support was removed from this crate.

Reading, upgrading, and serializing older versions belong in boundary tooling,
not in the core domain model crate.

## Consequences

Good:

- one clear target data model
- less code and fewer compatibility branches
- simpler docs, tests, and examples
- refactors can optimize for the current API instead of preserving legacy
  structure

Trade-offs:

- callers with older data need an explicit upgrade or conversion step
- legacy compatibility moved out of the core crate instead of being built in
