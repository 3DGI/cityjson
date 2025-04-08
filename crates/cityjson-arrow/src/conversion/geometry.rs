use std::sync::Arc;
use arrow::array::{DictionaryArray, ListBuilder, RecordBatch, UInt32Builder};
use arrow::datatypes::{DataType, Field, Fields, Int8Type, Schema};
use cityjson::prelude::{GeometryTrait, GeometryType, ResourceId32, StringStorage};
use cityjson::v2_0::Geometry;
use crate::error;

// --- Shared Field Types ---
lazy_static::lazy_static! {
    static ref U32_LIST_ITEM_NON_NULL: Arc<Field> = Arc::new(Field::new("item", DataType::UInt32, false));
    static ref U32_LIST_ITEM_NULLABLE: Arc<Field> = Arc::new(Field::new("item", DataType::UInt32, true)); // For resource indices
    static ref F64_LIST_ITEM_NON_NULL: Arc<Field> = Arc::new(Field::new("item", DataType::Float64, false));
}

pub fn geometry_to_arrow<SS: StringStorage>(
    geometry: Geometry<u32, ResourceId32, SS>,
) -> error::Result<RecordBatch> {
    // Schema for the geometry
    let schema = geometries_schema();

    let mut vertices_builder = ListBuilder::new(UInt32Builder::new());
    let mut rings_builder = ListBuilder::new(UInt32Builder::new());
    let mut surfaces_builder = ListBuilder::new(UInt32Builder::new());
    let mut shells_builder = ListBuilder::new(UInt32Builder::new());
    let mut solids_builder = ListBuilder::new(UInt32Builder::new());

    if let Some(boundary) = geometry.boundaries() {
        // Access raw slices using RawVertexView
        let vertices_slice: &[u32] = &*boundary.vertices_raw();
        let rings_slice: &[u32] = &*boundary.rings_raw();
        let surfaces_slice: &[u32] = &*boundary.surfaces_raw();
        let shells_slice: &[u32] = &*boundary.shells_raw();
        let solids_slice: &[u32] = &*boundary.solids_raw();

        // Append the slices to the builders
        vertices_builder.values().append_slice(vertices_slice);
        vertices_builder.append(true); // Append this list entry (it's not null)

        rings_builder.values().append_slice(rings_slice);
        rings_builder.append(true);

        surfaces_builder.values().append_slice(surfaces_slice);
        surfaces_builder.append(true);

        shells_builder.values().append_slice(shells_slice);
        shells_builder.append(true);

        solids_builder.values().append_slice(solids_slice);
        solids_builder.append(true);

    } else {
        // Geometry has no boundary (e.g., GeometryInstance might not store it directly)
        vertices_builder.append(false); // Append null list entry
        rings_builder.append(false);
        surfaces_builder.append(false);
        shells_builder.append(false);
        solids_builder.append(false);
    }
    
    
    
    RecordBatch::try_new(Arc::new(schema), Vec::new()).map_err(error::Error::from)
}

pub fn geometries_schema() -> Schema {
    Schema::new(vec![
        Field::new("type_geometry", DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)), false),
        Field::new("lod", DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)), true),
        Field::new("boundary_vertices", DataType::List(U32_LIST_ITEM_NON_NULL.clone()), true),
        Field::new("boundary_rings", DataType::List(U32_LIST_ITEM_NON_NULL.clone()), true),
        Field::new("boundary_surfaces", DataType::List(U32_LIST_ITEM_NON_NULL.clone()), true),
        Field::new("boundary_shells", DataType::List(U32_LIST_ITEM_NON_NULL.clone()), true),
        Field::new("boundary_solids", DataType::List(U32_LIST_ITEM_NON_NULL.clone()), true),
        Field::new("semantics_points", DataType::List(U32_LIST_ITEM_NON_NULL.clone()), true),
        Field::new("semantics_linestrings", DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))), true),
        Field::new("semantics_surfaces", DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))), true),
        // No need to duplicate the shell and solid arrays, because the cannot carry
        // semantic, material or texture information. We can simply use the boundary
        // arrays for these.
        Field::new("materials", DataType::List(Arc::new(
            Field::new("material_theme", DataType::Struct(Fields::from(vec![
                Field::new("theme", DataType::Utf8, false),
                // For each surface with a material, store the material reference
                Field::new("surfaces", DataType::List(Arc::new(Field::new("material_ref", DataType::UInt32, true))), false)
            ])), false)
        )), true),
        Field::new("textures_themes", DataType::List(Arc::new(Field::new("theme", DataType::Utf8, false))), true),
        Field::new("textures_rings", DataType::List(Arc::new(
            Field::new("texture_theme", DataType::Struct(Fields::from(vec![
                Field::new("theme_idx", DataType::UInt8, false),
                // For each ring, store the texture reference
                Field::new("rings", DataType::List(Arc::new(Field::new("texture_ref", DataType::UInt32, true))), false),
            ])), false)
        )), true),
        Field::new("textures_vertices", DataType::List(Arc::new(
            Field::new("texture_theme", DataType::Struct(Fields::from(vec![
                Field::new("theme_idx", DataType::UInt8, false),
                // UV coordinate mapping to the vertices
                Field::new("vertices", DataType::List(Arc::new(Field::new("uv_ref", DataType::UInt32, true))), false),
            ])), false)
        )), true),
        Field::new("instance_template", DataType::UInt32, true),
        Field::new("instance_reference_point", DataType::UInt32, true),
        Field::new("instance_transformation_matrix", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, false)), 16), true),
    ])
}
