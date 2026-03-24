# FFI and Bindings

`cjlib` is intended to become the stable, user-facing integration layer for more than just Rust.
As FFI crates and language bindings are added, the documentation site needs to describe:

- the Rust facade
- the stable FFI boundary
- binding-specific guidance for higher-level languages

## Why This Lives In MkDocs

Rustdoc is still useful for Rust APIs, but it is not the right home for the whole project documentation.
`cjlib` needs one documentation site that can cover:

- Rust usage
- C ABI or low-level FFI concepts
- higher-level bindings such as Python or other host languages
- shared concepts such as ownership, versioning, error categories, and format support

MkDocs is the better top-level documentation generator for that job because it is language-neutral.

## Documentation Split

The intended split is:

- API design pages define the Rust facade contract
- FFI pages define language-neutral conventions and ABI-level expectations
- future binding pages explain how each target language maps onto the shared FFI concepts

This keeps the long-term docs shape clean:

- shared concepts are written once
- Rust-specific details stay in the Rust-facing sections
- language-specific differences are documented where they belong
