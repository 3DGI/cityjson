use std::sync::Arc;
use arrow::array::{ArrayRef, DictionaryArray, FixedSizeListBuilder, Float64Builder, ListBuilder, RecordBatch, StringBuilder, StringDictionaryBuilder, StructBuilder, UInt32Builder};
use arrow::datatypes::{DataType, Field, Fields, Int8Type, Schema};
use cityjson::prelude::{DefaultResourcePool, GeometryTrait, GeometryType, ResourceId32, ResourcePool, StringStorage};
use cityjson::v2_0::Geometry;
use crate::error;

// --- Shared Field Types ---
lazy_static::lazy_static! {
    static ref U32_LIST_ITEM_NON_NULL: Arc<Field> = Arc::new(Field::new("item", DataType::UInt32, false));
    static ref U32_LIST_ITEM_NULLABLE: Arc<Field> = Arc::new(Field::new("item", DataType::UInt32, true)); // For resource indices
    static ref F64_LIST_ITEM_NON_NULL: Arc<Field> = Arc::new(Field::new("item", DataType::Float64, false));
}

pub fn geometries_to_arrow<SS: StringStorage>(
    geometries: &DefaultResourcePool<Geometry<u32, ResourceId32, SS>, ResourceId32>,
) -> error::Result<RecordBatch> {
    // Schema for the geometry
    let schema = geometries_schema();

    // --- Initialize Builders ---
    let mut type_builder = StringDictionaryBuilder::<Int8Type>::new();
    let mut lod_builder = StringDictionaryBuilder::<Int8Type>::new();
    // Boundary Builders
    let mut b_vertices_builder = ListBuilder::new(UInt32Builder::with_capacity(1024)); // todo: use a better estimate from 3DBAG
    let mut b_rings_builder = ListBuilder::new(UInt32Builder::with_capacity(128));
    let mut b_surfaces_builder = ListBuilder::new(UInt32Builder::with_capacity(64));
    let mut b_shells_builder = ListBuilder::new(UInt32Builder::with_capacity(2));
    let mut b_solids_builder = ListBuilder::new(UInt32Builder::with_capacity(2));
    // Semantics Builders
    let mut s_points_builder = ListBuilder::new(UInt32Builder::with_capacity(32));
    let mut s_linestrings_builder = ListBuilder::new(UInt32Builder::with_capacity(32));
    let mut s_surfaces_builder = ListBuilder::new(UInt32Builder::with_capacity(64));
    // Instance Builders
    let mut instance_template_builder = UInt32Builder::new();
    let mut instance_ref_pt_builder = UInt32Builder::new();
    let mut instance_matrix_builder = FixedSizeListBuilder::new(Float64Builder::new(), 16);


    
    for (resource_ref, geometry) in geometries.iter() {
        // Append Type and LoD (using DictionaryBuilder logic - see Arrow examples)
        type_builder.append_value(geometry.type_geometry().to_string()); // Example only, requires proper dictionary handling
        if let Some(lod) = geometry.lod() {
            lod_builder.append_value(lod.to_string());
        } else {
            lod_builder.append_null();
        }
        
        if let Some(boundary) = geometry.boundaries() {
            // Access raw slices using RawVertexView
            let vertices_slice: &[u32] = &*boundary.vertices_raw();
            let rings_slice: &[u32] = &*boundary.rings_raw();
            let surfaces_slice: &[u32] = &*boundary.surfaces_raw();
            let shells_slice: &[u32] = &*boundary.shells_raw();
            let solids_slice: &[u32] = &*boundary.solids_raw();

            // Append the slices to the builders
            b_vertices_builder.values().append_slice(vertices_slice);
            b_vertices_builder.append(true); // Append this list entry (it's not null)

            b_rings_builder.values().append_slice(rings_slice);
            b_rings_builder.append(true);

            b_surfaces_builder.values().append_slice(surfaces_slice);
            b_surfaces_builder.append(true);

            b_shells_builder.values().append_slice(shells_slice);
            b_shells_builder.append(true);

            b_solids_builder.values().append_slice(solids_slice);
            b_solids_builder.append(true);

        } else {
            // Geometry has no boundary (e.g., GeometryInstance might not store it directly)
            b_vertices_builder.append(false); // Append null list entry
            b_rings_builder.append(false);
            b_surfaces_builder.append(false);
            b_shells_builder.append(false);
            b_solids_builder.append(false);
        }

        // Append Instance Data
        instance_template_builder.append_option(geometry.instance_template().map(|rr| rr.index()));
        instance_ref_pt_builder.append_option(geometry.instance_reference_point().map(|vi| vi.value()));
        if let Some(matrix) = geometry.instance_transformation_matrix() {
            instance_matrix_builder.values().append_slice(matrix);
            instance_matrix_builder.append(true);
        } else {
            instance_matrix_builder.append(false);
        }
    }

    // --- Finish Builders and Create RecordBatch ---
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(type_builder.finish()),
        Arc::new(lod_builder.finish()),
        Arc::new(b_vertices_builder.finish()),
        Arc::new(b_rings_builder.finish()),
        Arc::new(b_surfaces_builder.finish()),
        Arc::new(b_shells_builder.finish()),
        Arc::new(b_solids_builder.finish()),
        Arc::new(s_points_builder.finish()),
        Arc::new(s_linestrings_builder.finish()),
        Arc::new(s_surfaces_builder.finish()),
/*        Arc::new(materials_list_builder.finish()),
        Arc::new(textures_themes_builder.finish()),
        Arc::new(textures_rings_builder.finish()),
        Arc::new(texture_vertices_builder.finish()),*/
        Arc::new(instance_template_builder.finish()),
        Arc::new(instance_ref_pt_builder.finish()),
        Arc::new(instance_matrix_builder.finish()),
    ];

    
    RecordBatch::try_new(Arc::new(schema), arrays).map_err(error::Error::from)
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
/*        // No need to duplicate the shell and solid arrays, because the cannot carry
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
        Field::new("texture_vertices", DataType::List(U32_LIST_ITEM_NULLABLE.clone()), true),*/
        Field::new("instance_template", DataType::UInt32, true),
        Field::new("instance_reference_point", DataType::UInt32, true),
        Field::new("instance_transformation_matrix", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, false)), 16), true),
    ])
}
