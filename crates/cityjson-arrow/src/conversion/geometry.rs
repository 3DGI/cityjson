use crate::error;
use arrow::array::{
    ArrayRef, DictionaryArray, FixedSizeListBuilder, Float64Builder, ListBuilder, RecordBatch,
    StringBuilder, StringDictionaryBuilder, StructBuilder, UInt32Builder,
};
use arrow::datatypes::{DataType, Field, Fields, Int8Type, Schema};
use cityjson::prelude::{
    DefaultResourcePool, GeometryTrait, GeometryType, ResourceId32, ResourcePool, ResourceRef,
    StringStorage, VertexRef,
};
use cityjson::v2_0::Geometry;
use std::sync::Arc;

// --- Shared Field Types ---
lazy_static::lazy_static! {
    static ref U32_LIST_ITEM_NON_NULL: Arc<Field> = Arc::new(Field::new("item", DataType::UInt32, false));
    static ref U32_LIST_ITEM_NULLABLE: Arc<Field> = Arc::new(Field::new("item", DataType::UInt32, true)); // For resource indices
    static ref F64_LIST_ITEM_NON_NULL: Arc<Field> = Arc::new(Field::new("item", DataType::Float64, false));
}

pub fn geometries_to_arrow<SS: StringStorage>(
    geometries_pool: &DefaultResourcePool<Geometry<u32, ResourceId32, SS>, ResourceId32>,
) -> error::Result<RecordBatch> {
    // Schema for the geometry
    let schema = geometries_schema();
    let num_rows = geometries_pool.len();

    // Special case for empty pools
    if num_rows == 0 {
        return Ok(RecordBatch::new_empty(Arc::new(schema)));
    }

    // --- Initialize Builders ---
    let mut id_builder = UInt32Builder::with_capacity(num_rows);
    let mut type_builder = StringDictionaryBuilder::<Int8Type>::new(); // TODO: calculate capacity
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
    // Material Builder
    let field_material_theme = schema.field_with_name("material_theme")?;
    let mut materials_list_builder = ListBuilder::new(StructBuilder::from_fields(
        vec![field_material_theme.clone()],
        0,
    ));
    // Instance Builders
    let mut instance_template_builder = UInt32Builder::new();
    let mut instance_ref_pt_builder = UInt32Builder::new();
    let mut instance_matrix_builder = FixedSizeListBuilder::new(Float64Builder::new(), 16);

    for (resource_ref, geometry) in geometries_pool.iter() {
        // ResourceId in pool
        id_builder.append_value(resource_ref.index());
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

        // Append Semantics Data
        if let Some(semantics) = geometry.semantics() {
            append_list_option(&mut s_points_builder, &semantics.points());
            append_list_option(&mut s_linestrings_builder, &semantics.linestrings());
            append_list_option(&mut s_surfaces_builder, &semantics.surfaces());
        } else {
            s_points_builder.append(false);
            s_linestrings_builder.append(false);
            s_surfaces_builder.append(false);
        }

        // Append Materials Data
        let num_surfaces_in_boundary = geometry.boundaries().map_or(0, |b| b.surfaces().len());
        if let Some(themed_materials) = geometry.materials() {
            materials_list_builder.append(true);
            let theme_struct_builder = materials_list_builder.values();
            for (theme, material_map) in themed_materials {
                theme_struct_builder.append(true);
                theme_struct_builder
                    .field_builder::<StringBuilder>(0)
                    .unwrap()
                    .append_value(theme.as_ref());
                let surface_list_builder = theme_struct_builder
                    .field_builder::<ListBuilder<UInt32Builder>>(1)
                    .unwrap();
                if num_surfaces_in_boundary > 0 {
                    surface_list_builder.append(true);
                    let value_builder = surface_list_builder.values();
                    for i in 0..num_surfaces_in_boundary {
                        let mat_idx = material_map
                            .surfaces()
                            .get(i)
                            .and_then(|opt_rr| opt_rr.as_ref().map(|rr| rr.index()));
                        value_builder.append_option(mat_idx);
                    }
                } else {
                    surface_list_builder.append(false); // No surfaces, append null list
                }
            }
        } else {
            materials_list_builder.append(false); // No materials for this geometry
        }

        // Append Instance Data
        instance_template_builder.append_option(geometry.instance_template().map(|rr| rr.index()));
        instance_ref_pt_builder
            .append_option(geometry.instance_reference_point().map(|vi| vi.value()));
        if let Some(matrix) = geometry.instance_transformation_matrix() {
            instance_matrix_builder.values().append_slice(matrix);
            instance_matrix_builder.append(true);
        } else {
            instance_matrix_builder.append(false);
        }
    }

    // --- Finish Builders and Create RecordBatch ---
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(id_builder.finish()),
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
        Arc::new(materials_list_builder.finish()),
        /* Arc::new(textures_themes_builder.finish()),
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
        Field::new("id", DataType::UInt32, false),
        Field::new(
            "type_geometry",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            false,
        ),
        Field::new(
            "lod",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            true,
        ),
        Field::new(
            "boundary_vertices",
            DataType::List(U32_LIST_ITEM_NON_NULL.clone()),
            true,
        ),
        Field::new(
            "boundary_rings",
            DataType::List(U32_LIST_ITEM_NON_NULL.clone()),
            true,
        ),
        Field::new(
            "boundary_surfaces",
            DataType::List(U32_LIST_ITEM_NON_NULL.clone()),
            true,
        ),
        Field::new(
            "boundary_shells",
            DataType::List(U32_LIST_ITEM_NON_NULL.clone()),
            true,
        ),
        Field::new(
            "boundary_solids",
            DataType::List(U32_LIST_ITEM_NON_NULL.clone()),
            true,
        ),
        Field::new(
            "semantics_points",
            DataType::List(U32_LIST_ITEM_NON_NULL.clone()),
            true,
        ),
        Field::new(
            "semantics_linestrings",
            DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
            true,
        ),
        Field::new(
            "semantics_surfaces",
            DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
            true,
        ),
        // No need to duplicate the shell and solid arrays, because they cannot carry
        // semantic, material or texture information. We can simply use the boundary
        // arrays for these.
        Field::new(
            "materials",
            DataType::List(Arc::new(Field::new(
                "material_theme",
                DataType::Struct(Fields::from(vec![
                    Field::new("theme", DataType::Utf8, false),
                    // For each surface with a material, store the material reference
                    Field::new(
                        "surfaces",
                        DataType::List(Arc::new(Field::new(
                            "material_ref",
                            DataType::UInt32,
                            true,
                        ))),
                        false,
                    ),
                ])),
                false,
            ))),
            true,
        ),
        /*
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
        Field::new(
            "instance_transformation_matrix",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, false)), 16),
            true,
        ),
    ])
}

// Helper to append Option<Vec<Option<ResourceId32>>> to ListBuilder<UInt32Builder>
fn append_list_option(builder: &mut ListBuilder<UInt32Builder>, data: &[Option<ResourceId32>]) {
    if data.is_empty() {
        builder.append(false);
    } else {
        builder.append(true);
        let values_builder = builder.values();
        values_builder.extend(data.iter().map(|rr| rr.as_ref().map(|v| v.index())));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use cityjson::prelude::*;
    use cityjson::v2_0::{CityModel, Material, Semantic, SemanticType};
    use arrow::array::{Array, DictionaryArray, ListArray, UInt32Array, StringArray, FixedSizeListArray, Float64Array};
    use arrow::datatypes::Int8Type;

    #[test]
    fn test_geometries_to_arrow() {
        // Create a city model to hold our geometries
        let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // --- Create a MultiPoint geometry ---
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);
        builder.add_point(QuantizedCoordinate::new(10, 20, 30));
        builder.add_point(QuantizedCoordinate::new(40, 50, 60));
        builder = builder.with_lod(LoD::LoD1);
        let _geom1_ref = builder.build().expect("Failed to build MultiPoint geometry");

        // --- Create a MultiSurface geometry with semantics and materials ---
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiSurface, BuilderMode::Regular);

        // Add vertices for a square surface
        let p0 = builder.add_point(QuantizedCoordinate::new(0, 0, 0));
        let p1 = builder.add_point(QuantizedCoordinate::new(10, 0, 0));
        let p2 = builder.add_point(QuantizedCoordinate::new(10, 10, 0));
        let p3 = builder.add_point(QuantizedCoordinate::new(0, 10, 0));

        // Create surface with a ring
        let ring1 = builder.add_ring(&[p0, p1, p2, p3, p0]).expect("Failed to add ring");
        let surface1 = builder.start_surface();
        builder.add_surface_outer_ring(ring1).expect("Failed to add ring to surface");

        // Add semantic (RoofSurface type)
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        builder.set_semantic_surface(Some(surface1), roof_semantic).expect("Failed to set semantic");

        // Add material with theme
        let mut roof_material = Material::new("Roof".to_string());
        roof_material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        builder.set_material_surface(Some(surface1), roof_material, "theme1".to_string())
            .expect("Failed to set material");

        builder = builder.with_lod(LoD::LoD2);
        let _geom2_ref = builder.build().expect("Failed to build MultiSurface geometry");

        // --- Create a GeometryInstance ---
        let ref_point = model.add_vertex(QuantizedCoordinate::new(100, 100, 100)).expect("Failed to add vertex");

        // Create a template geometry first
        let mut template_builder = GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Template);
        template_builder.add_template_point(RealWorldCoordinate::new(0.0, 0.0, 0.0));
        template_builder.add_template_point(RealWorldCoordinate::new(1.0, 1.0, 1.0));
        let template_ref = template_builder.build().expect("Failed to build template geometry");

        // Now create the instance referencing the template
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::GeometryInstance, BuilderMode::Regular);
        builder.add_vertex(ref_point);
        builder = builder.with_template(template_ref).expect("Failed to set template");
        builder = builder.with_transformation_matrix([
            2.0, 0.0, 0.0, 0.0,
            0.0, 2.0, 0.0, 0.0,
            0.0, 0.0, 2.0, 0.0,
            10.0, 20.0, 30.0, 1.0
        ]).expect("Failed to set transformation matrix");
        let _geom3_ref = builder.build().expect("Failed to build GeometryInstance");

        // Verify we have all geometries in the model
        assert_eq!(model.geometries().len(), 3, "Expected 3 geometries in the model");

        // Convert geometries to Arrow
        let batch = geometries_to_arrow(model.geometries()).expect("Failed to convert geometries to Arrow");

        // Verify batch structure
        assert_eq!(batch.num_rows(), 3, "Expected 3 rows in the batch");
        assert_eq!(batch.schema().fields().len(), 15, "Expected 15 columns in the schema");

        // Helper function to find a row by geometry type
        let find_row_by_type = |type_name: &str| -> usize {
            let type_array = batch.column(1).as_any().downcast_ref::<DictionaryArray<Int8Type>>()
                .expect("Expected Dictionary array for geometry type");
            let type_values = type_array.values();
            let type_dict = type_values.as_any().downcast_ref::<StringArray>()
                .expect("Expected StringArray for type dictionary");

            for i in 0..batch.num_rows() {
                let key = type_array.keys().value(i);
                let val = type_dict.value(key as usize);
                if val == type_name {
                    return i;
                }
            }
            panic!("Geometry type '{}' not found", type_name);
        };

        // --- Test MultiPoint geometry ---
        let mp_row = find_row_by_type("MultiPoint");

        // Verify LOD
        let lod_array = batch.column(2).as_any().downcast_ref::<DictionaryArray<Int8Type>>()
            .expect("Expected Dictionary array for LOD");
        let lod_values = lod_array.values();
        let lod_dict = lod_values.as_any().downcast_ref::<StringArray>()
            .expect("Expected StringArray for LOD dictionary");
        let key = lod_array.keys().value(mp_row);
        let lod_val = lod_dict.value(key as usize);
        assert_eq!(lod_val, "1", "Expected LOD1 for MultiPoint");

        // Verify vertices
        let vertices_array = batch.column(3).as_any().downcast_ref::<ListArray>()
            .expect("Expected ListArray for boundary vertices");
        let vertices = vertices_array.value(mp_row);
        let vertices_val = vertices.as_any().downcast_ref::<UInt32Array>()
            .expect("Expected UInt32Array for vertex values");
        assert_eq!(vertices_val.len(), 2, "Expected 2 vertices for MultiPoint");

        // --- Test MultiSurface geometry ---
        let ms_row = find_row_by_type("MultiSurface");

        // Verify LOD
        let key = lod_array.keys().value(ms_row);
        let lod_val = lod_dict.value(key as usize);
        assert_eq!(lod_val, "2", "Expected LOD2 for MultiSurface");

        // Verify semantics
        let semantics_array = batch.column(10).as_any().downcast_ref::<ListArray>()
            .expect("Expected ListArray for semantics surfaces");
        assert!(!semantics_array.is_null(ms_row), "Expected non-null semantics for MultiSurface");

        // --- Test GeometryInstance geometry ---
        let gi_row = find_row_by_type("GeometryInstance");

        // Verify template reference
        let template_array = batch.column(12).as_any().downcast_ref::<UInt32Array>()
            .expect("Expected UInt32Array for template references");
        assert!(!template_array.is_null(gi_row), "Expected non-null template reference");
        assert_eq!(template_array.value(gi_row), template_ref.index(), "Expected correct template reference");

        // Verify transformation matrix
        let matrix_array = batch.column(14).as_any().downcast_ref::<FixedSizeListArray>()
            .expect("Expected FixedSizeListArray for transformation matrices");
        assert!(!matrix_array.is_null(gi_row), "Expected non-null transformation matrix");

        let matrix = matrix_array.value(gi_row);
        let matrix_values = matrix.as_any().downcast_ref::<Float64Array>()
            .expect("Expected Float64Array for matrix values");
        assert_eq!(matrix_values.len(), 16, "Expected 16 values in transformation matrix");
        assert_eq!(matrix_values.value(0), 2.0, "Expected scale X = 2.0");
        assert_eq!(matrix_values.value(5), 2.0, "Expected scale Y = 2.0");
        assert_eq!(matrix_values.value(10), 2.0, "Expected scale Z = 2.0");
        assert_eq!(matrix_values.value(12), 10.0, "Expected translation X = 10.0");
        assert_eq!(matrix_values.value(13), 20.0, "Expected translation Y = 20.0");
        assert_eq!(matrix_values.value(14), 30.0, "Expected translation Z = 30.0");
    }
}
