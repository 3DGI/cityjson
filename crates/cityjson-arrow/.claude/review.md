You are a lead systems engineer. How would you write the most performant encode and decode process for converting between the cityjson-rs data model and the cityarrow schema?

Let me study the current implementation and data models before giving you a concrete answer.
Explore cityarrow codebase deeply

Now let me look at a few critical code sections in detail.
Read 5 files, ran a command

Now let me look at the vertex import path and the geometry import hot path.
Read 2 files

Now I have a thorough understanding of the codebase. Here's my analysis:
Performance Analysis: CityJSON <-> CityArrow Encode/Decode
Current Architecture Summary

The conversion is a ~6800-line module that decomposes OwnedCityModel into ~25 Arrow RecordBatch tables, then reassembles them on decode. The design is correct and well-structured, but leaves significant performance on the table in three areas: allocation patterns, iteration strategy, and attribute serialization.
Encoding Hot Paths and What I'd Change
1. Vertices: Eliminate Multi-Pass Iteration

The current code at convert/mod.rs:2931-2970 iterates vertices four times — once for IDs, once each for x/y/z:

// Current: 4 passes over the same data
Arc::new(UInt64Array::from((0..n).map(|i| i as u64).collect::<Vec<_>>()));
Arc::new(Float64Array::from(verts.iter().map(|v| v.x()).collect::<Vec<_>>()));
Arc::new(Float64Array::from(verts.iter().map(|v| v.y()).collect::<Vec<_>>()));
Arc::new(Float64Array::from(verts.iter().map(|v| v.z()).collect::<Vec<_>>()));

Each pass allocates a Vec, fills it, then hands it to an Arrow array which copies it into its own buffer. That's 8 allocations (4 Vecs + 4 Arrow buffers) and 4 cache-unfriendly passes.

Optimal approach: Single pass with Arrow builders or, better, direct buffer construction:

fn vertices_batch_from_model(schema: &Arc<Schema>, model: &OwnedCityModel) -> Result<RecordBatch> {
    let verts = model.vertices().as_slice();
    let n = verts.len();

    // Pre-allocate flat buffers, one pass
    let mut ids = Vec::with_capacity(n);
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut zs = Vec::with_capacity(n);

    for (i, v) in verts.iter().enumerate() {
        ids.push(i as u64);
        xs.push(v.x());
        ys.push(v.y());
        zs.push(v.z());
    }

    RecordBatch::try_new(schema.clone(), vec![
        Arc::new(UInt64Array::from(ids)),
        Arc::new(Float64Array::from(xs)),
        Arc::new(Float64Array::from(ys)),
        Arc::new(Float64Array::from(zs)),
    ]).map_err(Error::from)
}

For even better performance, use MutableBuffer directly to avoid the double-copy (Vec -> Arrow buffer):

use arrow_buffer::MutableBuffer;

let mut x_buf = MutableBuffer::new(n * 8);
let mut y_buf = MutableBuffer::new(n * 8);
let mut z_buf = MutableBuffer::new(n * 8);
for v in verts {
    x_buf.push(v.x());
    y_buf.push(v.y());
    z_buf.push(v.z());
}
// Convert MutableBuffer -> ScalarBuffer -> Float64Array with zero copy

This halves the allocation count and makes the iteration cache-friendly (sequential reads from the vertex array).
2. Attribute Encoding: Kill the HashMap Clone

The biggest red flag in the codebase is attributes_to_hash_map at line 5003-5010:

fn attributes_to_hash_map(attrs: &OwnedAttributes) -> HashMap<String, OwnedAttributeValue> {
    attrs.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

This is called per city object, per semantic, and per metadata row. It deep-clones every key and value into a new HashMap just to wrap it in OwnedAttributeValue::Map(...) so the generic projected_value_array can consume it. The clone is entirely avoidable.

Optimal approach: Make projected_struct_array_from_attributes work directly with &OwnedAttributes without the HashMap intermediary:

fn projected_struct_array_from_attributes(
    field: &FieldRef,
    spec: &ProjectedStructSpec,
    rows: &[Option<&OwnedAttributes>],
    geometry_id_map: &HashMap<GeometryHandle, u64>,
) -> Result<ArrayRef> {
    // Build each column directly by field-name lookup into OwnedAttributes
    let fields = struct_fields(field)?;
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(spec.fields.len());

    for ProjectedFieldSpec { name, value_spec, .. } in &spec.fields {
        let values: Vec<Option<&OwnedAttributeValue>> = rows
            .iter()
            .map(|row| row.and_then(|attrs| attrs.get(name)))
            .collect();
        columns.push(projected_value_array_ref(
            &field_by_name(&fields, name)?,
            value_spec,
            &values,
            geometry_id_map,
        )?);
    }
    // ... build StructArray from columns
}

This eliminates O(n * avg_attributes) String + Value clones and HashMap constructions. For a model with 100k city objects each having 10 attributes, that's ~1M avoided string clones.
3. Geometry Boundary Export: Reduce Vec-of-Vec Nesting

The geometry boundary tables flatten deeply nested Vec<Vec<Vec<u32>>> boundaries into six list columns. The current approach builds intermediate Vec<Option<Vec<u64>>> for each column, then calls list_u64_array which flattens again. This is two passes with an intermediate allocation.

Optimal approach: Write directly into offset + value buffers in a single pass:

fn build_boundary_columns(
    geometries: &[GeometryBoundaryRow],
) -> Result<(ListArray, ListArray, ListArray, ListArray, ListArray, ListArray)> {
    // Pre-scan for capacity (or estimate)
    let n = geometries.len();
    let mut vi_offsets = Vec::with_capacity(n + 1);
    let mut vi_values: Vec<u64> = Vec::new(); // or estimate capacity
    vi_offsets.push(0i32);

    // Single pass: for each geometry, directly append to flat buffers
    for geom in geometries {
        vi_values.extend(geom.vertex_indices.iter().map(|v| *v as u64));
        vi_offsets.push(vi_values.len() as i32);
        // ... same for other columns
    }
    // Build ListArrays directly from offsets + values
}

4. Encoding the ProjectedValueSpec Tree

projected_value_array (line 5012) dispatches per-type and builds a Vec per column. For deeply nested struct attributes, this recurses and allocates at each level. Consider a column-at-a-time approach where you resolve the full path to leaf arrays and build them bottom-up, avoiding intermediate Vec<Option<&OwnedAttributeValue>> allocations at each struct level.
Decoding Hot Paths and What I'd Change
5. Vertex Import: Batch Add Instead of Per-Row

At line 1834-1857, vertices are added one at a time via model.add_vertex() inside a scalar loop. If cityjson-rs supports bulk insertion or exposes the underlying Vec, this should be a single extend_from_slice or reserve + batch push:

fn import_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    let n = batch.num_rows();
    reserve_model_import(state, CityModelCapacities { vertices: n, .. })?;
    let cols = bind_vertex_columns(batch, "vertex_id")?;

    // Validate monotonic IDs in one pass (SIMD-friendly comparison)
    validate_strictly_increasing(cols.vertex_id)?;

    // Bulk construction: build Vec<RealWorldCoordinate> then extend model
    let coords: Vec<_> = (0..n)
        .map(|i| RealWorldCoordinate::new(cols.x.value(i), cols.y.value(i), cols.z.value(i)))
        .collect();
    state.model.add_vertices(coords)?; // hypothetical batch API
    Ok(())
}

Even without a batch API, removing the per-row ensure_strictly_increasing_u64 call (which does a branch + comparison) and validating monotonicity separately allows the compiler to vectorize the coordinate extraction loop.
6. Geometry Boundary Reconstruction: Avoid Redundant Lookups

In import_boundary_geometries_batch (line 2070), each geometry does a HashMap::get into grouped_rows.boundaries.row_by_id for the boundary row. Since geometry IDs are strictly increasing and the boundary table is sorted, this can be a linear scan or binary search over a sorted array instead of a hash lookup — better cache behavior for sequential access patterns.
7. Deferred Geometry Attachment: Pre-size the Pending Slots

The push_pending_geometry_attachment function (line 2247) uses ensure_slot which calls resize_with — this can trigger multiple reallocations as cityobject_ix values arrive. Since the max cityobject_ix is known from the last row of the geometries batch (line 2085), pre-allocate all slots upfront:

// Already done partially at line 2085, but ensure_slot is still called per-row
// Better: state.pending_geometry_attachments.resize_with(max_ix + 1, Vec::new);
// Then push without ensure_slot in the hot loop

8. Attribute Decoding: Avoid Clone-on-Insert

At line 2341-2342:

for (key, value) in projected_attributes.iter() {
    object.attributes_mut().insert(key.clone(), value.clone());
}

If projected_attributes_from_array returned owned values (which it likely already constructs), use into_iter() + move semantics instead of clone:

for (key, value) in projected_attributes {
    object.attributes_mut().insert(key, value);
}

Structural Optimizations
9. Schema Caching

canonical_schema_set builds all ~25 schemas. Currently called once per encode/decode, which is fine. But field_from_schema (line 4766) does a linear search by name for each column — for batch construction functions that call it repeatedly, cache the FieldRef lookups.
10. Projection Layout Cloning

self.projection.clone() is called multiple times in dispatch_table (lines 1439, 1455, 1554). The ProjectionLayout contains Option<ProjectedStructSpec> trees which can be deep. Pass &self.projection instead of cloning, or wrap in Arc<ProjectionLayout>.
11. RecordBatch Cloning in collect_tables

At the transport layer, collect_tables clones every RecordBatch. Arrow's RecordBatch::clone() is cheap (Arc bumps), but it's still unnecessary if the consumer can take &RecordBatch. Consider making the function return references or Cow.
Summary: Priority-Ordered Optimization Roadmap
Priority	Change	Expected Impact
P0	Eliminate attributes_to_hash_map clone	Removes O(n * attrs) deep clones
P0	Single-pass vertex encoding with MutableBuffer	4x fewer passes, 2x fewer allocations
P1	Move semantics for attribute decoding (remove .clone() in insert)	Eliminates per-attribute cloning on decode
P1	Arc<ProjectionLayout> instead of .clone()	Removes deep struct cloning per table dispatch
P1	Direct offset/value buffer construction for list columns	Eliminates Vec-of-Option-Vec intermediaries
P2	Batch vertex import (validate IDs separately from construction)	Better vectorization, fewer branches
P2	Linear scan for sorted boundary lookups instead of HashMap	Better cache locality on decode
P2	Pre-allocate pending geometry attachment slots	Eliminates per-row resize checks
P3	Bottom-up projected attribute array construction	Fewer recursive allocations for deep structs

The P0 items are pure wins with no API changes needed — they're internal allocation and iteration improvements. P1 items may need minor signature adjustments. P2/P3 are refinements that matter at scale (>100k objects).
1 step

What would you change if you could modify both the cityjson-ra data model and the cityarrow schema in order to make them work together better? Show me alternatives.

Let me dig deeper into the cityjson-rs side to understand exactly where the impedance mismatches live.
Ran 2 agents

Now let me check a few specific friction points I want to quantify.
Read 3 files, ran 3 commands

Now let me look at how boundaries are flattened and reconstructed, which is the core impedance mismatch.
Searched 2 patterns, read 2 files

I now have a complete picture. Here's the analysis:
Co-Design Alternatives: cityjson-rs + cityarrow

The current conversion is ~6800 lines and exists largely because the two data models were designed independently. Here are the structural mismatches and what changes to either side would eliminate them.
Mismatch 1: Boundary Representation (the biggest one)

cityjson-rs: Boundaries are stored as a flat offset-based structure internally. Boundary<u32> has:

    vertices_raw() — flat &[u32] of vertex indices
    rings_raw(), surfaces_raw(), shells_raw(), solids_raw() — offset arrays (CSR-style)

cityarrow: Geometry boundaries are stored as length-based lists:

    vertex_indices: List<u64> — flat indices
    ring_lengths: List<u32>, surface_lengths: List<u32>, etc.

The friction: Encoding calls offsets_to_lengths (line 4110), decoding calls lengths_to_offsets (line 6320). Both allocate a new Vec and do arithmetic on every element. The vertex indices are also widened u32 -> u64 on encode and narrowed u64 -> u32 on decode with fallible casts.
Alternative A: Align cityjson-rs to offset-based Arrow lists

Change the Arrow schema to store offsets instead of lengths, matching cityjson-rs exactly:

// Arrow schema change
geometry_boundaries:
  geometry_id: UInt64
  vertex_indices: List<UInt32>    // was UInt64
  ring_offsets: List<UInt32>      // was ring_lengths
  surface_offsets: List<UInt32>   // was surface_lengths
  shell_offsets: List<UInt32>
  solid_offsets: List<UInt32>

Arrow's ListArray already uses offsets internally. This makes encode/decode a memcpy — the cityjson-rs offset buffer is directly bit-compatible with the Arrow offset buffer. No arithmetic, no allocation.

Also: change vertex indices to UInt32 in Arrow. CityJSON spec caps vertex indices at u32, so the widening to u64 is unnecessary overhead.

Cost: Breaking schema change. Consumers reading boundaries must reconstruct lengths if they want them (trivial subtraction).

Payoff: Boundary encode/decode becomes zero-copy for the index and topology buffers.
Alternative B: Add a raw buffer export to cityjson-rs

Add to Boundary<u32>:

impl Boundary<u32> {
    /// Returns the underlying flat vertex buffer as a byte slice.
    pub fn vertices_as_bytes(&self) -> &[u8] { /* transmute the &[u32] */ }
    
    /// Returns offset arrays as contiguous byte slices.
    pub fn rings_as_bytes(&self) -> &[u8] { ... }
    
    /// Construct from pre-validated raw buffers (unsafe, no topology check).
    pub unsafe fn from_raw_buffers(
        vertices: Vec<u32>,
        rings: Vec<u32>,
        surfaces: Vec<u32>,
        shells: Vec<u32>,
        solids: Vec<u32>,
    ) -> Self { ... }
}

This lets cityarrow build Arrow buffers directly from the byte representation without interpreting individual elements. Combined with Alternative A, the entire geometry boundary encode path becomes:

// Encode: zero-copy buffer transfer
let arrow_buf = Buffer::from_slice_ref(boundary.vertices_raw());

Mismatch 2: Attributes as Dynamic HashMap vs. Projected Struct Columns

cityjson-rs: OwnedAttributes is a HashMap<String, OwnedAttributeValue> (or BTreeMap). Each city object carries its own independent map.

cityarrow: Attributes are projected into typed Arrow struct columns via ProjectedStructSpec. The projection is discovered by scanning all objects to find the union of all attribute keys and their types.

The friction: The projection discovery pass (line 1011: discover_projection_layout) scans the entire model. Then encoding does per-row HashMap lookups against the discovered schema. Decoding reconstructs HashMaps from struct columns. The attributes_to_hash_map clone (line 5003) exists purely because the projection system needs a Map variant to recurse into.
Alternative C: Column-oriented attribute store in cityjson-rs

Replace the per-object HashMap with a column store that lives at the model level:

// In cityjson-rs
pub struct AttributeColumns {
    schema: Vec<(String, AttributeType)>,  // column name + type
    columns: Vec<AttributeColumn>,          // one per schema entry
}

pub enum AttributeColumn {
    Boolean(Vec<Option<bool>>),
    Int64(Vec<Option<i64>>),
    Float64(Vec<Option<f64>>),
    Utf8(Vec<Option<String>>),
    Struct(AttributeColumns),  // nested
    // ...
}

impl OwnedCityModel {
    pub fn cityobject_attribute_columns(&self) -> &AttributeColumns { ... }
}

Now projection discovery is free (the schema is already explicit), and encoding is a direct column-to-column transfer: AttributeColumn::Float64(vec) -> Float64Array::from(vec).

Cost: Large API change to cityjson-rs. Per-object attribute access becomes a column lookup + row index instead of a HashMap get. Reading individual objects gets slightly worse, but bulk operations get dramatically better.

Payoff: Eliminates projection discovery, eliminates per-row HashMap cloning, eliminates recursive projected_value_array dispatch. Attribute encoding becomes O(columns) instead of O(rows * columns).
Alternative D: Typed attribute schema in cityjson-rs (lighter touch)

Keep per-object HashMaps but add a model-level schema declaration:

impl OwnedCityModel {
    /// Declared attribute schema, if known.
    pub fn cityobject_attribute_schema(&self) -> Option<&AttributeSchema> { ... }
    
    /// Set the declared schema (e.g., from a CityJSON metadata extension).
    pub fn set_cityobject_attribute_schema(&mut self, schema: AttributeSchema) { ... }
}

pub struct AttributeSchema {
    pub fields: Vec<(String, AttributeType, bool /* nullable */)>,
}

On decode, cityarrow sets the schema from the ProjectionLayout. On encode, if a schema is already set, skip the discovery scan. Objects still use HashMaps, but the projection is pre-computed.

Cost: Minimal API change. Backward compatible.

Payoff: Eliminates the full-model discovery scan on encode. Doesn't fix the per-row cloning, but removes the O(n) schema inference.
Mismatch 3: Handle-Based Object Graph vs. ID-Based Flat Tables

cityjson-rs: Uses opaque handles (GeometryHandle, SemanticHandle, etc.) backed by a SlotMap/arena. Objects reference geometries by handle. Traversal is pointer-chase.

cityarrow: Uses sequential u64 IDs. Relationships are expressed as foreign keys between tables (e.g., geometries.cityobject_ix -> cityobjects.cityobject_ix).

The friction: Encoding builds five HashMap<Handle, u64> maps (line 1009-1030) to translate handles to sequential IDs. Decoding builds the reverse maps. Every geometry encode/decode does a HashMap lookup.
Alternative E: Dual-indexed storage in cityjson-rs

Expose the dense index alongside the handle:

impl OwnedCityModel {
    /// Returns the dense sequential index for a geometry handle.
    pub fn geometry_index(&self, handle: GeometryHandle) -> Option<u64> { ... }
    
    /// Returns the handle for a dense sequential index.
    pub fn geometry_handle(&self, index: u64) -> Option<GeometryHandle> { ... }
    
    /// Iterate geometries with their dense indices.
    pub fn iter_geometries_indexed(&self) -> impl Iterator<Item = (u64, GeometryHandle, &Geometry)> { ... }
}

If the underlying SlotMap already stores entries densely (which most SlotMap implementations do), the dense index is just the position in the internal Vec. Expose it.

Cost: Requires cityjson-rs to guarantee stable dense indices (no compaction that reorders). Minor API addition.

Payoff: Eliminates all five HashMap<Handle, u64> constructions on encode and the reverse maps on decode. Handle-to-ID translation becomes O(1) field access instead of O(1)-amortized hash lookup. For a model with 500k geometries, that's 500k fewer hash operations per direction.
Alternative F: Use handles as Arrow IDs directly

If cityjson-rs handles are internally u32 or u64 slot keys, expose their raw value:

impl GeometryHandle {
    pub fn as_u64(&self) -> u64 { self.0 as u64 }
    pub fn from_raw(value: u64) -> Self { Self(value as usize) }
}

Use the raw handle value as the Arrow geometry_id directly. No mapping needed at all. The IDs won't be sequential (SlotMap keys include a generation counter), but that's fine — Arrow doesn't require sequential IDs, only uniqueness.

Cost: Arrow IDs become opaque (not 0, 1, 2, ...). Slightly larger ID space. Requires cityjson-rs to expose handle internals.

Payoff: Completely eliminates the ID mapping phase. Encode and decode skip all HashMap construction.
Mismatch 4: Semantic Type as Enum vs. String Column

cityjson-rs: SemanticType is a Rust enum with ~20 variants plus Extension(String).

cityarrow: The semantics table stores semantic_type as Utf8 (string column).

The friction: encode_semantic_type (line 4082) does a 20-arm match to produce strings. parse_semantic_type does the reverse. Every semantic row pays for string allocation and comparison.
Alternative G: Dictionary-encoded semantic types

Change the Arrow schema to use DictionaryArray<UInt8>:

semantics:
  semantic_id: UInt64
  semantic_type: Dictionary<UInt8, Utf8>   // was plain Utf8

Arrow dictionary encoding stores each unique string once and references it by index. For 20 known semantic types, the dictionary has 20 entries and each row is a single byte.

On the cityjson-rs side, add SemanticType::as_u8() and SemanticType::from_u8():

impl SemanticType {
    pub fn discriminant(&self) -> u8 {
        match self {
            Self::RoofSurface => 0,
            Self::GroundSurface => 1,
            // ...
            Self::Extension(_) => 255,
        }
    }
}

Cost: Minor schema change. Dictionary arrays are widely supported.

Payoff: Eliminates per-row string allocation for semantic types. Encode becomes a u8::from() call; decode becomes a table lookup. Same optimization applies to geometry_type, object_type, and lod columns.
Mismatch 5: Vertex Index Width

cityjson-rs: Vertex indices are u32 (via VertexIndex<u32>).

cityarrow: Vertex indices in boundary tables are List<UInt64>.

The friction: Every vertex index is widened on encode (u64::from(u32)) and narrowed on decode with a fallible u32::try_from(u64) (line 6266). For a model with 10M vertex references, that's 10M unnecessary conversions.
Alternative H: Match the index width

Change the Arrow schema's vertex index lists to UInt32:

geometry_boundaries:
  vertex_indices: List<UInt32>    // was List<UInt64>

Cost: Breaking schema change. Limits vertex count to 4B per geometry (not per model). This is fine — no CityJSON model has 4B vertices in a single geometry.

Payoff: Combined with Alternative A (offset-based storage), the boundary encode path becomes a direct buffer copy from cityjson-rs to Arrow with zero per-element processing.
Mismatch 6: UV Coordinates as f32 vs. f64

cityjson-rs: UV coordinates are f32 (UVCoordinate::new(f32, f32)).

cityarrow: The texture_vertices table stores u and v as Float64.

The friction: Encode widens f32 -> f64. Decode narrows with f64_to_f32_preserving_cast (line 1899), which does a lossy-cast check per value.
Alternative I: Store UVs as Float32 in Arrow

texture_vertices:
  uv_id: UInt64
  u: Float32     // was Float64
  v: Float32     // was Float64

Cost: Minor schema change. Arrow fully supports Float32.

Payoff: Eliminates per-UV-coordinate cast and validation. Halves the UV column size in Arrow (4 bytes vs 8 bytes per coordinate).
Compound Effect: What the Best Co-Design Looks Like

If you could change both sides, here's the maximum-impact combination:
Change	Side	Eliminates
A Offset-based boundary lists + UInt32 indices	Arrow	offset<->length conversion, u32<->u64 casts
B Raw buffer export on Boundary	cityjson-rs	per-element iteration in boundary encode
D Model-level attribute schema	cityjson-rs	full-model projection discovery scan
E Dense index exposure on handles	cityjson-rs	5 HashMap constructions per encode/decode
G Dictionary-encoded enum columns	Arrow	per-row string allocation for types
I Float32 UV columns	Arrow	per-UV cast + validation

Together, these changes would reduce encode_parts from its current profile:

Before:
  1. discover_projection_layout  — O(cityobjects * attrs)  ← eliminated by D
  2. build 5 HashMap<Handle,u64> — O(total_entities)        ← eliminated by E  
  3. flatten boundaries           — O(vertex_indices)        ← eliminated by A+B
  4. clone attributes per row     — O(rows * attr_keys)      ← reduced by D
  5. allocate strings for types   — O(geometries+semantics)  ← eliminated by G
  6. widen u32→u64 indices        — O(vertex_indices)        ← eliminated by H
  
After:
  1. Build Arrow buffers directly from cityjson-rs internal buffers
  2. Dictionary-encode enum columns via discriminant lookup
  3. Transfer attribute columns by reference or schema-aware iteration

The decode path sees symmetric gains: no narrowing casts, no HashMap construction for reverse ID mapping, no string parsing for enum types, and attribute columns insert directly into a schema-aware model.

The realistic estimate: these changes combined would bring the conversion from "proportional to model size with constant-factor overhead per element" to "proportional to model size with near-zero per-element overhead" — the conversion becomes dominated by memcpy bandwidth rather than per-element logic.