# FFI Documentation Conventions

These conventions keep the future `cjlib` FFI documentation small, consistent, and maintainable.

## Document Shared Concepts Once

Language-neutral concepts should be documented in one place:

- `CityModel` lifecycle and ownership
- version handling
- error categories
- input and output formats
- stream versus whole-document loading

Binding pages should link back to those shared concepts instead of re-explaining them.

## Split By Layer

The intended documentation layers are:

- Rust facade docs for the native Rust API
- FFI docs for the stable cross-language boundary
- binding docs for target-language ergonomics

That prevents the Rust API docs from turning into mixed Rust/C/Python prose.

## Prefer Stable Concepts Over Raw Implementation Details

The FFI docs should describe stable concepts first:

- opaque handles
- ownership transfer
- borrowed versus owned data
- error reporting
- version compatibility

They should avoid baking temporary internal implementation details into the public documentation.

## Keep Examples Parallel Across Languages

When bindings arrive, the same small example should ideally exist in parallel forms:

- Rust
- low-level FFI
- target-language binding

That makes the relationship between the APIs obvious without multiplying conceptual surface area.

## Keep The Site Language-Neutral At The Top

The main MkDocs navigation should stay broad enough to hold:

- Rust guides
- FFI reference and conventions
- language-specific binding guides

That is the main reason the documentation site should stay MkDocs-driven rather than Rustdoc-driven.
