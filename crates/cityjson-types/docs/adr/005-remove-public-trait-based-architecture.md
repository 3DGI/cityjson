# Remove the Public Trait-Based Architecture

## Status

Accepted

## Related Commits

- `81b55f0` Generalize Geometry to trait
- `806f6b8` Add blanket trait as container for associated types
- `785f8a9` Refactor GeometryTrait and CityObjectTrait to use macros
- `cef9fff` Remove cityobject and geometry traits
- `d54b8dc` Refactor CityModel traits to use macros
- `30c8d97` Remove CityModel and CityModelType traits
- `44ed078` Remove leftover traits module

## Context

The crate used to model much of the public API through traits shared across
multiple `CityJSON` versions.

That looked flexible, but in practice it made the code harder to work with:

- core domain concepts were split across traits, blanket traits, and concrete
  wrappers
- simple operations often required jumping through several abstraction layers
- associated-type plumbing made signatures harder to read than the underlying
  model required
- version-specific wrappers still ended up carrying most of the real behavior

This was more architecture than the crate needed.

## Decision

The public trait-based architecture was removed.

The crate now exposes concrete domain types directly, with inherent methods on
the actual stored types instead of a trait matrix describing them.

Traits are still acceptable for small internal contracts where they solve a
local problem, but they are no longer used as the main public architecture for
the CityJSON object model.

## Consequences

Good:

- the public API is easier to read and navigate
- signatures reflect the actual data model more directly
- fewer abstraction layers separate callers from the concrete types they use
- refactors are easier because behavior lives with the types that implement it

Trade-offs:

- there is less generic reuse across versions at the type-system level
- some internal duplication is preferable to rebuilding a large trait hierarchy
