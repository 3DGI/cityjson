# Record the Post-`v0.5.0-refactor1` Deserializer Baseline

## Status

Accepted

## Related Commits

- `497421e` bench refactoring and compare to `v0.4.5`

## Context

`v0.5.0-refactor1` completed the migration from the old handwritten parser to
the `cityjson-rs` backend and its normalized model.

That refactor improved architecture, but it also introduced a measurable
deserialization regression. Representative release probes taken immediately
after the refactor showed:

| Dataset | `serde_json::Value` | refactored `from_str_owned` | legacy `v0.4.5` |
| --- | ---: | ---: | ---: |
| `10-356-724.city.json` | `60.2 ms` | `89.0 ms` | `29.1 ms` |
| `30gz1_04.city.json` | `2.47 s` | `4.04 s` | `0.97 s` |

The initial hot path looked roughly like this:

```rust
let root = parse_raw_root(input)?;                  // typed shell + RawValue sections
let cityobjects = from_str(root.cityobjects.get())?; // reparse big subtree

for object in cityobjects {
    let geometry = parse_nested_boundary_vectors(object.geometry)?;
    let draft = GeometryDraft::from_nested_parts(geometry)?;
    let handle = draft.insert_into(&mut model)?;    // validate + analyze + build + validate
    model.add_cityobject(handle)?;
}
```

The regression was not one bug. It was the accumulated cost of:

- reparsing large `RawValue` sections
- materializing nested geometry trees before flattening them again
- routing every geometry through `GeometryDraft`
- converting attributes into a new typed tree before converting them again into
  the backend model
- eagerly transforming and inserting vertices into the normalized backend

## Decision

Treat deserializer performance as a first-class architecture concern for the
post-refactor codebase.

The optimization program after `v0.5.0-refactor1` would follow four rules:

1. keep the normalized `cityjson-rs` model
2. remove redundant parse and conversion passes
3. use explicit handwritten visitors where derive-heavy code hides repeated work
4. measure each step with repeatable release probes and Criterion benches

The main targets were:

- trusted stored-geometry insertion
- streaming `CityObjects` import
- direct flat boundary parsing
- direct attribute deserialization into backend values

## Consequences

Good:

- the optimization work had a documented baseline instead of relying on memory
- each follow-up change could be evaluated against the same datasets
- the team could optimize for the real normalized model rather than falling
  back to the old pre-refactor architecture

Trade-offs:

- the deserializer would become more explicit and less derive-driven
- some handwritten parser code would be preferred over generic convenience
  layers when the benchmark data justified it
