# JSON API

This document defines the intended public shape of `cityjson_lib::json`.

`cityjson_lib::json` is not a second JSON implementation.
It is the explicit boundary layer over `serde_cityjson` for callers that need
more than the default document-oriented `CityModel::from_*` path.

## Why A `json` Module Exists

These concerns belong in a format-qualified module:

- probing the root kind and version
- explicit document parsing
- explicit feature parsing
- feature-stream reading
- feature-stream writing
- document and feature serialization
- future raw or staged JSON access

They should not be folded into `CityModel` itself.

## Stable Surface

The intended surface is:

```rust
pub mod json {
    pub enum RootKind {
        CityJSON,
        CityJSONFeature,
    }

    pub struct Probe {
        /* private fields */
    }

    impl Probe {
        pub fn kind(&self) -> RootKind;
        pub fn version(&self) -> Option<crate::CityJSONVersion>;
    }

    pub fn probe(bytes: &[u8]) -> crate::Result<Probe>;

    pub fn from_slice(bytes: &[u8]) -> crate::Result<crate::CityModel>;
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<crate::CityModel>;
    pub fn from_feature_slice(bytes: &[u8]) -> crate::Result<crate::CityModel>;

    pub fn read_feature_stream<R>(
        reader: R,
    ) -> crate::Result<impl Iterator<Item = crate::Result<crate::CityModel>>>
    where
        R: std::io::BufRead;

    pub fn write_feature_stream<I, W>(writer: W, models: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = crate::CityModel>,
        W: std::io::Write;

    pub fn to_vec(model: &crate::CityModel) -> crate::Result<Vec<u8>>;
    pub fn to_string(model: &crate::CityModel) -> crate::Result<String>;
    pub fn to_writer(
        writer: &mut impl std::io::Write,
        model: &crate::CityModel,
    ) -> crate::Result<()>;
    pub fn to_feature_string(model: &crate::CityModel) -> crate::Result<String>;
    pub fn to_feature_writer(
        writer: &mut impl std::io::Write,
        model: &crate::CityModel,
    ) -> crate::Result<()>;

    pub mod staged {
        pub struct FeatureObjectFragment<'a> {
            pub id: &'a str,
            pub object: &'a serde_json::value::RawValue,
        }

        pub struct FeatureAssembly<'a> {
            pub id: &'a str,
            pub cityobjects: &'a [FeatureObjectFragment<'a>],
            pub vertices: &'a [[i64; 3]],
        }

        pub fn from_feature_slice_with_base(
            feature_bytes: &[u8],
            base_document_bytes: &[u8],
        ) -> crate::Result<crate::CityModel>;

        pub fn from_feature_file_with_base(
            path: impl AsRef<std::path::Path>,
            base_document_bytes: &[u8],
        ) -> crate::Result<crate::CityModel>;

        pub fn from_feature_assembly_with_base(
            assembly: FeatureAssembly<'_>,
            base_document_bytes: &[u8],
        ) -> crate::Result<crate::CityModel>;
    }
}
```

## Relationship To `CityModel`

The intended relationship is:

- `CityModel::from_slice` is a convenience alias for `json::from_slice`
- `CityModel::from_file` is a convenience alias for `json::from_file`
- serialization remains explicit and format-qualified

The module should stay function-oriented.
It does not need a second public serde model, ad hoc extension sniffing, or a
document-oriented `from_stream` alias.

## Probe Once

The probing surface should stay narrow:

- `probe(bytes)` is enough
- avoid `probe_file`
- avoid `probe_reader`

One small `Probe` object is simpler than a scattered set of helpers and avoids
reparsing the same header multiple times.

## `CityJSONFeature` Is A Boundary Form

`CityJSONFeature` belongs in the JSON boundary, not in the semantic API.

The returned semantic unit is still:

- one `crate::CityModel`
- or a stream of `crate::CityModel` values

That is the same rule used by the rest of the architecture: semantic model
inside, wire-format distinctions at the boundary.

## `json::staged`

Advanced reconstruction paths belong under `json::staged`.

That namespace is for callers such as indexers and staged readers that already
own raw feature bytes, cached base-document metadata, or preassembled feature
fragments and need to rehydrate a `CityModel` without pretending that those
transport-specific pieces are the normal API.

## Leave Room For Raw And Staged APIs

If lower-level JSON access becomes necessary, it should appear explicitly in
this module.
Examples include raw documents, staged readers, or other specialized parsing
paths.

Those are valid advanced tools, but they should not distort the default owned
`CityModel` path.
