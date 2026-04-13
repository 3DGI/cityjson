# Deserialize Attributes Directly Into Backend Values

## Status

Accepted

## Related Commits

- `6ceb3eca` Deserilaize attributes directly into backend values

## Context

After the boundary rewrite, the next visible hotspot was attribute handling.

The old attribute path did the same logical work twice:

1. deserialize JSON into `RawAttribute<'de>`
2. walk that tree again to build `AttributeValue<SS>`

At the same time, the backend representation used an extra box per nested
element, which increased allocation pressure for deep arrays and maps.

The effective shape was:

```rust
enum RawAttribute<'de> {
    Null,
    Bool(bool),
    Number(Number),
    String(Cow<'de, str>),
    Array(Vec<RawAttribute<'de>>),
    Object(HashMap<&'de str, RawAttribute<'de>>),
}

let raw: HashMap<&str, RawAttribute<'de>> = map.next_value()?;
let attrs = attribute_map::<SS>(raw, "attributes")?;
```

## Decision

Attributes are now deserialized directly into backend values.

The parser uses `AttributeValueSeed<SS>`, `AttributesSeed<SS>`, and
`OptionalAttributesSeed<SS>` so the `CityObject` visitor can parse attributes
and extra properties straight into `cityjson-rs::AttributeValue<SS>` and
`Attributes<SS>`.

At the same time, the backend attribute tree was simplified from boxed nested
values to direct nested values:

```rust
// Before
AttributeValue::Vec(Vec<Box<AttributeValue<SS>>>)
AttributeValue::Map(HashMap<SS::String, Box<AttributeValue<SS>>>)

// After
AttributeValue::Vec(Vec<AttributeValue<SS>>)
AttributeValue::Map(HashMap<SS::String, AttributeValue<SS>>)
```

The new hot path is:

```rust
while let Some(key) = map.next_key::<&'de str>()? {
    match key {
        "attributes" => {
            attributes = map.next_value_seed(OptionalAttributesSeed::<SS>::new())?;
        }
        _ => {
            let value = map.next_value_seed(AttributeValueSeed::<SS>::new())?;
            extra.insert(SS::store(key), value);
        }
    }
}
```

This preserves borrowed-mode support for unescaped strings while removing the
temporary recursive attribute tree.

## Consequences

Good:

- one full recursive allocation and conversion pass is removed
- nested arrays and maps no longer allocate one box per child
- `CityObject` deserialization now produces final attributes directly

Trade-offs:

- the direct seeds are more explicit than the original generic `RawAttribute`
  path
- borrowed mode still rejects escaped JSON strings because zero-copy borrowing
  cannot represent them safely

Representative effect:

- Criterion on March 27, 2026 moved to about `36.2 ms` for
  `10-356-724.city.json` and about `827 ms` for `30gz1_04.city.json`
- this was another clear step down from the pre-rewrite post-boundary parser
  state and made the large-file benchmark faster than `v0.4.5`

Current benchmark crossover against `serde_json::Value`:

- `30gz1_04.city.json` is now materially faster than `serde_json::Value`
- `10-356-724.city.json` is still slower than `serde_json::Value`

The most plausible explanation is dataset shape, not one remaining regression.

`30gz1_04.city.json` is a geometry-dominant stream:

- about `391 MB`
- `63,343` CityObjects
- `63,343` geometries, so roughly one geometry per object
- no parent/child relations
- mostly `MultiSurface`

That profile matches the current strengths of the importer:

- the boundary parser in `src/de/geometry.rs` flattens large numeric boundary
  arrays directly into `Boundary<u32>`
- geometry construction goes straight into `Geometry::from_stored_parts(...)`
  and `add_geometry_unchecked(...)`
- the generic nested JSON DOM that `serde_json::Value` must allocate is avoided

In other words, on very large regular geometry payloads, the normalized
`cityjson-rs` representation is cheaper than building a full generic JSON tree.

`10-356-724.city.json` is less favorable:

- about `7.6 MB`
- `3,146` CityObjects
- `6,292` geometries, so about two geometries per object
- `1,573` parent relations and `1,573` child relations
- about `24.5` attributes per object
- mostly `Solid`

That means the fixed semantic work in `cityjson-json` matters much more:

- direct attribute deserialization still builds backend `Attributes<SS>`
- relation resolution still runs after object import
- vertex coordinates are still transformed into backend coordinates
- `Solid` boundaries are deeper than `MultiSurface` boundaries

For a small file, `serde_json::Value` can remain faster simply because building
a generic JSON tree is cheap enough, while `cityjson-json` still performs the
extra domain-specific normalization that the backend needs.
