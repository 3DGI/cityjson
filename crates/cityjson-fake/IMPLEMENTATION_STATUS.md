# Implementation Status

## Migration from serde_cityjson v1.1 to cityjson-rs v2.0

All core functionality has been ported and validated against the CityJSON v2.0.1 schema.

## What works

### Core generation

- `CityModelBuilder` — deterministic (seeded RNG), fluent builder API
- `vertices()` — initializes `transform` property (required by schema); actual vertices are created on-demand during geometry generation
- `cityobjects()` — generates top-level city objects with valid geometry, semantics, materials, textures, geographical extents, and configurable multiple geometries per object
- `metadata()` — all fields (identifier, reference date, reference system, title, point of contact) use the seeded RNG; fully deterministic
- `materials()` / `textures()` — multi-theme support, attached to geometry surfaces
- `attributes()` — random typed attribute values on city objects

### Geometry types (all 7 standard types)

- MultiPoint
- MultiLineString
- MultiSurface
- CompositeSurface
- Solid
- MultiSolid
- CompositeSolid
- GeometryInstance (via geometry templates)

### City object hierarchy

- Parent-child relationships with `parents`/`children` fields wired correctly
- `CityObjectGroup` members are wired through `children`, `parents`, and `children_roles`
- Only types with valid subtypes (Building, Bridge, Tunnel) generate children
- Child IDs are guaranteed unique (counter suffix appended)
- Top-up loop ensures the configured `min_cityobjects` count is always met

### Geometry counts

- `min/max_members_*` geometry controls are honored for `MultiPoint`, `MultiLineString`, `MultiSurface`, `Solid`, `MultiSolid`, `CompositeSurface`, and `CompositeSolid`
- `min/max_members_cityobject_geometries` controls how many geometries each city object receives

### Type/geometry schema compatibility

Every city object type only receives geometry types allowed by the CityJSON v2.0 schema:

| City object type(s)                                                                                                                       | Allowed geometry types                                                 |
|-------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| Building, BuildingPart, BuildingRoom, BuildingUnit, BuildingStorey, Bridge, BridgePart, BridgeRoom, Tunnel, TunnelPart, TunnelHollowSpace | MultiSurface, CompositeSurface, Solid, CompositeSolid                  |
| LandUse                                                                                                                                   | MultiSurface, CompositeSurface                                         |
| TINRelief                                                                                                                                 | CompositeSurface                                                       |
| PlantCover                                                                                                                                | MultiSurface, CompositeSurface, Solid, MultiSolid, CompositeSolid      |
| Road, Railway, Waterway, TransportSquare                                                                                                  | MultiLineString, MultiSurface, CompositeSurface                        |
| WaterBody                                                                                                                                 | MultiLineString, MultiSurface, CompositeSurface, Solid, CompositeSolid |
| All others                                                                                                                                | Any of the 7 standard types                                            |

### GeometryInstance / templates

- Template geometries always include the required `lod` field
- `GeometryInstance` is only assigned to city object types that support it per schema
- When `use_templates=true` without an explicit `allowed_types_cityobject`, only first-level types that support `GeometryInstance` are selected: `SolitaryVegetationObject`, `CityFurniture`,
  `OtherConstruction`

### First-level vs second-level city objects

- Second-level types (e.g., `BuildingPart`, `BridgeFurniture`, `TunnelInstallation`) require a `parents` field and fail schema validation as top-level objects
- When `allowed_types_cityobject` contains only second-level types, the generator falls back to `Building`
- Second-level types only appear as children of a compatible parent

### Configurability (CJFakeConfig)

Most config fields are wired and respected:

- `CityObjectConfig`: min/max counts, hierarchy toggle, min/max children, allowed types
- `GeometryConfig`: allowed geometry types, allowed LoDs, geometry member counts, min/max geometries per city object
- `VertexConfig`: min/max coordinate values
- `MaterialConfig`: min/max materials, number of themes, per-property generation toggles
- `TextureConfig`: min/max textures, number of themes, max texture vertex budget, allow-none toggle
- `TemplateConfig`: use_templates toggle, min/max templates
- `SemanticConfig`: enabled toggle, allowed semantic surface types
- `MetadataConfig`: all fields optional/toggleable

## Test coverage

| Test suite                            | Tests | Status   |
|---------------------------------------|-------|----------|
| `tests/api.rs`                        | 25    | All pass |
| `tests/validation.rs`                 | 12    | All pass |
| `tests/fuzz.rs` (proptest, 100 cases) | 1     | Passes   |
| `src/` unit tests                     | 7     | All pass |

**Total: 45 direct tests, all passing.**

## Known limitations

- `Extension` city object types are excluded from random generation
- Geographical extent for `GeometryInstance` city objects reflects only the reference point, not the full template bounds
- Texture URI values are random filesystem-style paths, not valid URLs
