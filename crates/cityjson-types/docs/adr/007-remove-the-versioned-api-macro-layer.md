# Remove the Versioned API Macro Layer

## Status

Accepted

## Related Commits

- `8f6a637` Refactor the Extension trait to use macros
- `8f08302` Refactor the Material and Texture traits to use macros
- `ba4cfde` Refactor the Semantic trait to use macros
- `0398173` Refactor v1.1, v2.0 Metadata to use macros
- `785f8a9` Refactor GeometryTrait and CityObjectTrait to use macros
- `d54b8dc` Refactor CityModel traits to use macros
- `f427358` Add opaque handles and expand macros

## Context

The crate used to implement much of its public API through macros that expanded
the same versioned wrappers across multiple `CityJSON` modules.

That reduced some duplication, but it also created its own problems:

- behavior was defined indirectly instead of where the public types lived
- understanding an API often required reading macro expansions and generated
  patterns
- refactors became harder because structural changes had to fit the macro
  system first
- once legacy versions were no longer first-class, the cross-version macro
  layer mostly preserved architecture that no longer paid for itself

We still wanted small helper macros where they improved local implementation
details, but not as the main way the public API was shaped.

## Decision

The macro-based versioned API layer was removed.

The public API is now written directly around the concrete `v2_0` modules and
types.

Small internal macros may still exist, but they are implementation details.
They do not define the crate as a cross-version generated API surface.

## Consequences

Good:

- public behavior is easier to find in the concrete modules
- module structure matches the user-facing API more closely
- refactors are simpler because code can move without preserving a versioned
  macro framework
- internal helper macros can stay small and local

Trade-offs:

- some repeated code is accepted when it keeps the implementation clearer
- adding another first-class version later would require an explicit new design
  instead of reactivating a generic macro matrix
