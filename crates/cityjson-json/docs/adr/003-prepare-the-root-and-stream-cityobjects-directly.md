# Prepare the Root and Stream `CityObjects` Directly

## Status

Accepted

## Related Commits

- `272e938` Rewrite root parser and cityobject import path

## Context

After the trusted geometry path landed, the next large cost was the
`CityObjects` import architecture.

The parser still followed a shell-heavy pattern:

- deserialize the root into a thin shell
- keep large sections as `RawValue`
- parse `CityObjects` again into intermediate typed collections
- import those intermediate values into the final model

That kept the code generic, but it preserved too much staging work on the hot
path.

## Decision

The root parser and `CityObjects` importer were rewritten around an explicit
two-stage import model:

1. pass 1 prepares the root state needed to initialize the model
2. pass 2 streams the `CityObjects` map directly into the initialized model

The accepted shape is:

```rust
struct PreparedRoot<'de> {
    type_name: &'de str,
    version: Option<&'de str>,
    transform: Option<RawTransform>,
    vertices: Vec<[f64; 3]>,
    metadata: Option<RawMetadataSection<'de>>,
    extensions: Option<HashMap<&'de str, RawExtension<'de>>>,
    cityobjects: &'de RawValue,
    appearance: Option<RawAppearanceSection<'de>>,
    geometry_templates: Option<RawGeometryTemplatesSection<'de>>,
    extra: HashMap<&'de str, RawAttribute<'de>>,
}

let prepared = parse_root(input)?;
let mut model = build_model_shell(prepared.header_and_resources())?;
import_cityobjects(prepared.cityobjects, &mut model, &resources)?;
```

Inside the second pass, `CityObjects` are imported entry by entry instead of
first buffering a map of deserialized objects:

```rust
while let Some(id) = map.next_key::<&str>()? {
    let raw_object = map.next_value::<StreamingCityObject<SS>>()?;
    let imported = import_cityobject(id, raw_object, model, resources)?;
    handle_by_id.insert(id, imported.source_handle);
    pending.push(imported);
}

resolve_relations(pending, &handle_by_id, model)?;
```

The important rule is that we still allow one explicit second parse of the raw
`CityObjects` slice, but we do not materialize a second full object graph before
inserting into the model.

## Consequences

Good:

- the root parser has one clear job: prepare model-wide state
- the `CityObjects` importer has one clear job: stream objects into the model
- object relations are resolved after import without a full intermediate model
- field order at the document root no longer drives the import architecture

Trade-offs:

- the parser still scans the raw `CityObjects` slice twice overall
- the code is more explicit than a derive-based root shell

Representative effect:

- the root and `CityObjects` rewrite moved representative release probes from
  about `69.6 ms` / `2.72 s` down to about `44.2 ms` / `1.076 s`
- on Criterion this moved the deserializer materially closer to
  `serde_json::Value` on small files and clearly ahead on the large file
