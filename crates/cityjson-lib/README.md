# cjlib

`cjlib` is a thin, user-facing facade over [`cityjson-rs`](../cityjson-rs).

It keeps a small set of convenience constructors and version-dispatch logic:

- `CityModel::from_slice`
- `CityModel::from_file`
- `CityModel::from_stream`
- `CityJSONVersion`

The in-memory model comes from `cityjson-rs`. `cjlib::CityModel` is an owned newtype over
`cityjson::v2_0::OwnedCityModel`, and the crate re-exports `cityjson` so callers can drop down to
the underlying API when needed.

Today, the supported import path is:

- `CityJSON` v2.0 document import
- strict `CityJSONFeature` stream aggregation into a v2.0 model

Legacy `CityJSON` v1.0 and v1.1 branches are still recognized at the boundary and remain
`todo!()` intentionally.
