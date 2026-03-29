# FFI Documentation Conventions

These conventions keep the FFI documentation small, consistent, and centered on
the shared low-level story.

## Document Shared Concepts Once

Language-neutral concepts belong in the shared core docs:

- model ownership and lifecycle
- version handling
- error categories
- parse and serialize entry points
- collection access and bulk buffer access
- transform and quantization policy

Target pages should link back to the shared core instead of re-explaining it.

## Split By Layer

The documentation layers are:

- Rust facade docs for native Rust users
- shared FFI docs for the low-level cross-language core
- binding docs for target-specific ergonomics

That prevents the Rust docs from turning into mixed Rust/C++/Python/JS prose.

## Prefer Stable Concepts Over Temporary Internals

The shared docs should emphasize durable ideas:

- opaque handles
- ownership transfer
- immutable and mutable access patterns
- bulk operations over chatty crossings
- error and version reporting

Avoid baking temporary crate layouts or one-off implementation experiments into
the long-term binding docs.

## Keep Examples Parallel Where Possible

When examples are added, prefer parallel views of the same workflow:

- Rust facade
- shared low-level FFI core
- target-specific binding

That makes the mapping between layers obvious without multiplying conceptual
surface area.

## Keep The Top Level Language-Neutral

The main docs navigation should stay broad enough to hold:

- Rust guides
- shared FFI concepts
- target-specific binding guides

That is the main reason the site stays MkDocs-driven rather than Rustdoc-only.
