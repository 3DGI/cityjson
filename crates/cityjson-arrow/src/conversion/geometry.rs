use crate::error;
use crate::error::{Error, Result};
use arrow::array::{
    Array, DictionaryArray, FixedSizeListArray, Float64Array, ListArray, StringArray, UInt32Array,
};
use arrow::array::{
    ArrayBuilder, ArrayRef, FixedSizeListBuilder, Float64Builder, ListBuilder, RecordBatch,
    StringBuilder, StringDictionaryBuilder, StructBuilder, UInt32Builder,
};
use arrow::datatypes::{DataType, Field, Fields, Int8Type, Schema};
use cityjson::prelude::{
    Boundary, GeometryType, LoD, MaterialMap, SemanticMap, TextureMap, VertexIndex,
};
use cityjson::prelude::{DefaultResourcePool, ResourceId32, ResourcePool, StringStorage};
use cityjson::v2_0::Geometry;
use std::hash::Hash;
use std::sync::Arc;

// --- Shared Field Types ---
lazy_static::lazy_static! {
    // TODO: the list items should be not-nullable, but it seems tha the PrimitiveBuilder only builds nullable arrays and I'm lazy now to manually set up the builder as for example in metadata.geographical_extent, but with correct offsets
    static ref U32_LIST_ITEM_NON_NULL: Arc<Field> = Arc::new(Field::new("item", DataType::UInt32, true));
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
    let mut materials_list_builder = ListBuilder::new(StructBuilder::from_fields(
        vec![
            Field::new("theme", DataType::Utf8, false),
            // For each surface with a material, store the material reference
            Field::new(
                "surfaces",
                DataType::List(Arc::new(Field::new("material_ref", DataType::UInt32, true))),
                false,
            ),
        ],
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
            let vertices_slice: &[u32] = &boundary.vertices_raw();
            let rings_slice: &[u32] = &boundary.rings_raw();
            let surfaces_slice: &[u32] = &boundary.surfaces_raw();
            let shells_slice: &[u32] = &boundary.shells_raw();
            let solids_slice: &[u32] = &boundary.solids_raw();

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
                // See docs why dyn ArrayBuilder and then downcast: arrow_array::builder::StructBuilder
                let surface_list_builder = theme_struct_builder
                    .field_builder::<ListBuilder<Box<dyn ArrayBuilder>>>(1)
                    .unwrap();
                if num_surfaces_in_boundary > 0 {
                    surface_list_builder.append(true);
                    let value_builder = surface_list_builder
                        .values()
                        .as_any_mut()
                        .downcast_mut::<UInt32Builder>()
                        .unwrap();
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
            // For null lists, still need to append values to maintain array structure
            instance_matrix_builder.values().append_slice(&[0.0; 16]);
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
                "item", /* TODO: would be better to call it "material_theme" but the built-in list builder defaults to "item". Same applies for the nullable field, that should be false, but the built-in builder only does nullable. */
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
                true,
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
        // TODO: the list items should be not-nullable, but it seems tha the PrimitiveBuilder only builds nullable arrays and I'm lazy now to manually set up the builder as for example in metadata.geographical_extent, but with correct offsets
        Field::new(
            "instance_transformation_matrix",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, true)), 16),
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

/// Converts an Arrow RecordBatch containing geometry data into a cityjson-rs geometry resource pool.
///
/// This function extracts geometry objects from the Arrow representation and reconstructs them
/// as cityjson-rs Geometry objects in a resource pool.
///
/// # Parameters
///
/// * `batch` - The Arrow RecordBatch containing geometry data
///
/// # Returns
///
/// A Result containing either the populated geometry pool or an error
pub fn arrow_to_geometries<SS>(
    batch: &RecordBatch,
) -> Result<DefaultResourcePool<Geometry<u32, ResourceId32, SS>, ResourceId32>>
where
    SS: StringStorage + Default,
    SS::String: AsRef<str> + From<String> + Eq + Hash,
{
    // Create a new empty pool
    let mut pool = DefaultResourcePool::new();

    // If the batch is empty, return the empty pool
    if batch.num_rows() == 0 {
        return Ok(pool);
    }

    // Extract basic arrays
    /*    let id_array = batch
            .column_by_name("id")
            .ok_or_else(|| Error::MissingField("id".to_string()))?
            .as_any()
            .downcast_ref::<UInt32Array>()
            .ok_or_else(|| Error::Conversion("Failed to downcast id array".to_string()))?;
    */
    let type_array = batch
        .column_by_name("type_geometry")
        .ok_or_else(|| Error::MissingField("type_geometry".to_string()))?
        .as_any()
        .downcast_ref::<DictionaryArray<Int8Type>>()
        .ok_or_else(|| Error::Conversion("Failed to downcast type_geometry array".to_string()))?;

    let lod_array = batch
        .column_by_name("lod")
        .ok_or_else(|| Error::MissingField("lod".to_string()))?
        .as_any()
        .downcast_ref::<DictionaryArray<Int8Type>>()
        .ok_or_else(|| Error::Conversion("Failed to downcast lod array".to_string()))?;

    // Extract boundary arrays
    let vertices_array = batch
        .column_by_name("boundary_vertices")
        .ok_or_else(|| Error::MissingField("boundary_vertices".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            Error::Conversion("Failed to downcast boundary_vertices array".to_string())
        })?;

    let rings_array = batch
        .column_by_name("boundary_rings")
        .ok_or_else(|| Error::MissingField("boundary_rings".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast boundary_rings array".to_string()))?;

    let surfaces_array = batch
        .column_by_name("boundary_surfaces")
        .ok_or_else(|| Error::MissingField("boundary_surfaces".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            Error::Conversion("Failed to downcast boundary_surfaces array".to_string())
        })?;

    let shells_array = batch
        .column_by_name("boundary_shells")
        .ok_or_else(|| Error::MissingField("boundary_shells".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast boundary_shells array".to_string()))?;

    let solids_array = batch
        .column_by_name("boundary_solids")
        .ok_or_else(|| Error::MissingField("boundary_solids".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast boundary_solids array".to_string()))?;

    // Extract instance-related arrays
    let instance_template_array = batch
        .column_by_name("instance_template")
        .ok_or_else(|| Error::MissingField("instance_template".to_string()))?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| {
            Error::Conversion("Failed to downcast instance_template array".to_string())
        })?;

    let instance_reference_point_array = batch
        .column_by_name("instance_reference_point")
        .ok_or_else(|| Error::MissingField("instance_reference_point".to_string()))?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| {
            Error::Conversion("Failed to downcast instance_reference_point array".to_string())
        })?;

    let instance_transformation_matrix_array = batch
        .column_by_name("instance_transformation_matrix")
        .ok_or_else(|| Error::MissingField("instance_transformation_matrix".to_string()))?
        .as_any()
        .downcast_ref::<FixedSizeListArray>()
        .ok_or_else(|| {
            Error::Conversion("Failed to downcast instance_transformation_matrix array".to_string())
        })?;

    // Process each row in the batch
    for i in 0..batch.num_rows() {
        // Extract geometry type
        let type_id = type_array.keys().value(i);
        let type_values = type_array.values();
        let type_dict = type_values
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Expected StringArray for type dictionary");
        let type_value = type_dict.value(type_id as usize);

        // Parse the geometry type
        let geometry_type = parse_geometry_type(type_value)?;

        // Extract LoD if present
        let lod = if lod_array.is_null(i) {
            None
        } else {
            let lod_id = lod_array.keys().value(i);
            let lod_values = lod_array.values();
            let lod_dict = lod_values
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("Expected StringArray for lod dictionary");
            let lod_value = lod_dict.value(lod_id as usize);

            parse_lod(lod_value)
        };

        // Extract boundary data
        let boundary = if vertices_array.is_null(i) {
            None
        } else {
            // Get the arrays for this geometry
            let vertices_arr = vertices_array.value(i);
            let rings_arr = if rings_array.is_null(i) {
                None
            } else {
                Some(rings_array.value(i))
            };
            let surfaces_arr = if surfaces_array.is_null(i) {
                None
            } else {
                Some(surfaces_array.value(i))
            };
            let shells_arr = if shells_array.is_null(i) {
                None
            } else {
                Some(shells_array.value(i))
            };
            let solids_arr = if solids_array.is_null(i) {
                None
            } else {
                Some(solids_array.value(i))
            };

            // Extract boundary data
            Some(extract_boundary(
                vertices_arr,
                rings_arr,
                surfaces_arr,
                shells_arr,
                solids_arr,
            )?)
        };

        // Extract instance data if this is a GeometryInstance
        let (instance_template, instance_reference_point, instance_transformation_matrix) =
            if geometry_type == GeometryType::GeometryInstance {
                // Template reference
                let template = if instance_template_array.is_null(i) {
                    None
                } else {
                    Some(ResourceId32::new(instance_template_array.value(i), 0))
                };

                // Reference point
                let ref_point = if instance_reference_point_array.is_null(i) {
                    None
                } else {
                    let value = instance_reference_point_array.value(i);
                    Some(VertexIndex::new(value))
                };

                // Transformation matrix
                let matrix = if instance_transformation_matrix_array.is_null(i) {
                    None
                } else {
                    let matrix_list = instance_transformation_matrix_array.value(i);
                    let matrix_values = matrix_list
                        .as_any()
                        .downcast_ref::<Float64Array>()
                        .ok_or_else(|| {
                            Error::Conversion("Failed to downcast matrix values".to_string())
                        })?;

                    let mut matrix = [0.0; 16];
                    for j in 0..16 {
                        matrix[j] = matrix_values.value(j);
                    }

                    Some(matrix)
                };

                (template, ref_point, matrix)
            } else {
                (None, None, None)
            };

        // For the initial implementation, leave semantics, materials, and textures as None
        // These can be added in a subsequent refinement
        let semantics: Option<SemanticMap<u32, ResourceId32>> = None;
        let materials: Option<Vec<(SS::String, MaterialMap<u32, ResourceId32>)>> = None;
        let textures: Option<Vec<(SS::String, TextureMap<u32, ResourceId32>)>> = None;

        // Create and add the Geometry instance to the pool
        let geometry = Geometry::new(
            geometry_type,
            lod,
            boundary,
            semantics,
            materials,
            textures,
            instance_template,
            instance_reference_point,
            instance_transformation_matrix,
        );

        pool.add(geometry);
    }

    Ok(pool)
}

/// Parses a geometry type string into a GeometryType enum
fn parse_geometry_type(value: &str) -> Result<GeometryType> {
    match value {
        "MultiPoint" => Ok(GeometryType::MultiPoint),
        "MultiLineString" => Ok(GeometryType::MultiLineString),
        "MultiSurface" => Ok(GeometryType::MultiSurface),
        "CompositeSurface" => Ok(GeometryType::CompositeSurface),
        "Solid" => Ok(GeometryType::Solid),
        "MultiSolid" => Ok(GeometryType::MultiSolid),
        "CompositeSolid" => Ok(GeometryType::CompositeSolid),
        "GeometryInstance" => Ok(GeometryType::GeometryInstance),
        _ => Err(Error::Conversion(format!(
            "Unknown geometry type: {}",
            value
        ))),
    }
}

/// Parses a LoD string into a LoD enum
fn parse_lod(value: &str) -> Option<LoD> {
    match value {
        "0" => Some(LoD::LoD0),
        "0.0" => Some(LoD::LoD0_0),
        "0.1" => Some(LoD::LoD0_1),
        "0.2" => Some(LoD::LoD0_2),
        "0.3" => Some(LoD::LoD0_3),
        "1" => Some(LoD::LoD1),
        "1.0" => Some(LoD::LoD1_0),
        "1.1" => Some(LoD::LoD1_1),
        "1.2" => Some(LoD::LoD1_2),
        "1.3" => Some(LoD::LoD1_3),
        "2" => Some(LoD::LoD2),
        "2.0" => Some(LoD::LoD2_0),
        "2.1" => Some(LoD::LoD2_1),
        "2.2" => Some(LoD::LoD2_2),
        "2.3" => Some(LoD::LoD2_3),
        "3" => Some(LoD::LoD3),
        "3.0" => Some(LoD::LoD3_0),
        "3.1" => Some(LoD::LoD3_1),
        "3.2" => Some(LoD::LoD3_2),
        "3.3" => Some(LoD::LoD3_3),
        _ => None,
    }
}

/// Extracts boundary data from Arrow arrays and constructs a Boundary object
fn extract_boundary(
    vertices_arr: ArrayRef,
    rings_arr: Option<ArrayRef>,
    surfaces_arr: Option<ArrayRef>,
    shells_arr: Option<ArrayRef>,
    solids_arr: Option<ArrayRef>,
) -> Result<Boundary<u32>> {
    // Extract the raw UInt32 arrays from the ListArrays
    let vertices = vertices_arr
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| Error::Conversion("Failed to downcast vertices values".to_string()))?;

    // Create a new Boundary with appropriate capacity
    let mut boundary = Boundary::with_capacity(
        vertices.len(),
        rings_arr.as_ref().map_or(0, |r| r.len()),
        surfaces_arr.as_ref().map_or(0, |r| r.len()),
        shells_arr.as_ref().map_or(0, |r| r.len()),
        solids_arr.as_ref().map_or(0, |r| r.len()),
    );

    // Convert vertices (requires copying to create VertexIndex objects)
    boundary.set_vertices_from_iter(vertices.iter().map(|v| VertexIndex::new(v.unwrap())));
    // Convert rings if present
    if let Some(arr_ref) = rings_arr {
        let arr_any = arr_ref.as_any();
        let arr = arr_any
            .downcast_ref::<UInt32Array>()
            .ok_or_else(|| Error::Conversion("Failed to downcast rings values".to_string()))?;
        boundary.set_rings_from_iter(arr.iter().map(|r| VertexIndex::new(r.unwrap())));
    }

    // Convert surfaces if present
    if let Some(arr_ref) = surfaces_arr {
        let arr_any = arr_ref.as_any();
        let arr = arr_any
            .downcast_ref::<UInt32Array>()
            .ok_or_else(|| Error::Conversion("Failed to downcast surfaces values".to_string()))?;
        boundary.set_surfaces_from_iter(arr.iter().map(|r| VertexIndex::new(r.unwrap())));
    }

    // Convert shells if present
    if let Some(arr_ref) = shells_arr {
        let arr_any = arr_ref.as_any();
        let arr = arr_any
            .downcast_ref::<UInt32Array>()
            .ok_or_else(|| Error::Conversion("Failed to downcast shells values".to_string()))?;
        boundary.set_shells_from_iter(arr.iter().map(|r| VertexIndex::new(r.unwrap())));
    }

    // Convert solids if present
    if let Some(arr_ref) = solids_arr {
        let arr_any = arr_ref.as_any();
        let arr = arr_any
            .downcast_ref::<UInt32Array>()
            .ok_or_else(|| Error::Conversion("Failed to downcast solids values".to_string()))?;
        boundary.set_solids_from_iter(arr.iter().map(|r| VertexIndex::new(r.unwrap())));
    }

    Ok(boundary)
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use arrow::array::{
        Array, DictionaryArray, FixedSizeListArray, Float64Array, Int8Array, Int32Array, ListArray,
        StringArray, UInt32Array,
    };
    use arrow::buffer::Buffer;
    use arrow::datatypes::Int8Type;
    use cityjson::prelude::*;
    use cityjson::v2_0::{CityModel, Material, Semantic, SemanticType};

    // TODO: This test needs to be updated to work with the new AttributePool-based API
    #[cfg(any())]
    #[test]
    fn test_geometries_to_arrow() {
        // Create a city model to hold our geometries
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // --- Create a MultiPoint geometry ---
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);
        builder.add_point(QuantizedCoordinate::new(10, 20, 30));
        builder.add_point(QuantizedCoordinate::new(40, 50, 60));
        builder = builder.with_lod(LoD::LoD1);
        let _geom1_ref = builder
            .build()
            .expect("Failed to build MultiPoint geometry");

        // --- Create a MultiSurface geometry with semantics and materials ---
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiSurface, BuilderMode::Regular);

        // Add vertices for a square surface
        let p0 = builder.add_point(QuantizedCoordinate::new(0, 0, 0));
        let p1 = builder.add_point(QuantizedCoordinate::new(10, 0, 0));
        let p2 = builder.add_point(QuantizedCoordinate::new(10, 10, 0));
        let p3 = builder.add_point(QuantizedCoordinate::new(0, 10, 0));

        // Create surface with a ring
        let ring1 = builder
            .add_ring(&[p0, p1, p2, p3, p0])
            .expect("Failed to add ring");
        let surface1 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring1)
            .expect("Failed to add ring to surface");

        // Add semantic (RoofSurface type)
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        builder
            .set_semantic_surface(Some(surface1), roof_semantic)
            .expect("Failed to set semantic");

        // Add material with theme
        let mut roof_material = Material::new("Roof".to_string());
        roof_material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        builder
            .set_material_surface(Some(surface1), roof_material, "theme1".to_string())
            .expect("Failed to set material");

        builder = builder.with_lod(LoD::LoD2);
        let _geom2_ref = builder
            .build()
            .expect("Failed to build MultiSurface geometry");

        // --- Create a GeometryInstance ---
        let ref_point = model
            .add_vertex(QuantizedCoordinate::new(100, 100, 100))
            .expect("Failed to add vertex");

        // Create a template geometry first
        let mut template_builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Template);
        template_builder.add_template_point(RealWorldCoordinate::new(0.0, 0.0, 0.0));
        template_builder.add_template_point(RealWorldCoordinate::new(1.0, 1.0, 1.0));
        let template_ref = template_builder
            .build()
            .expect("Failed to build template geometry");

        // Now create the instance referencing the template
        let mut builder = GeometryBuilder::new(
            &mut model,
            GeometryType::GeometryInstance,
            BuilderMode::Regular,
        );
        builder.add_vertex(ref_point);
        builder = builder
            .with_template(template_ref)
            .expect("Failed to set template");
        builder = builder
            .with_transformation_matrix([
                2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 10.0, 20.0, 30.0, 1.0,
            ])
            .expect("Failed to set transformation matrix");
        let _geom3_ref = builder.build().expect("Failed to build GeometryInstance");

        // Verify we have all geometries in the model
        assert_eq!(
            model.iter_geometries().len(),
            3,
            "Expected 3 geometries in the model"
        );

        // Convert geometries to Arrow
        let batch = geometries_to_arrow(model.iter_geometries())
            .expect("Failed to convert geometries to Arrow");

        // Verify batch structure
        assert_eq!(batch.num_rows(), 3, "Expected 3 rows in the batch");
        assert_eq!(
            batch.schema().fields().len(),
            15,
            "Expected 15 columns in the schema"
        );

        // Helper function to find a row by geometry type
        let find_row_by_type = |type_name: &str| -> usize {
            let type_array = batch
                .column(1)
                .as_any()
                .downcast_ref::<DictionaryArray<Int8Type>>()
                .expect("Expected Dictionary array for geometry type");
            let type_values = type_array.values();
            let type_dict = type_values
                .as_any()
                .downcast_ref::<StringArray>()
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
        let lod_array = batch
            .column(2)
            .as_any()
            .downcast_ref::<DictionaryArray<Int8Type>>()
            .expect("Expected Dictionary array for LOD");
        let lod_values = lod_array.values();
        let lod_dict = lod_values
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Expected StringArray for LOD dictionary");
        let key = lod_array.keys().value(mp_row);
        let lod_val = lod_dict.value(key as usize);
        assert_eq!(lod_val, "1", "Expected LOD1 for MultiPoint");

        // Verify vertices
        let vertices_array = batch
            .column(3)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("Expected ListArray for boundary vertices");
        let vertices = vertices_array.value(mp_row);
        let vertices_val = vertices
            .as_any()
            .downcast_ref::<UInt32Array>()
            .expect("Expected UInt32Array for vertex values");
        assert_eq!(vertices_val.len(), 2, "Expected 2 vertices for MultiPoint");

        // --- Test MultiSurface geometry ---
        let ms_row = find_row_by_type("MultiSurface");

        // Verify LOD
        let key = lod_array.keys().value(ms_row);
        let lod_val = lod_dict.value(key as usize);
        assert_eq!(lod_val, "2", "Expected LOD2 for MultiSurface");

        // Verify semantics
        let semantics_array = batch
            .column(10)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("Expected ListArray for semantics surfaces");
        assert!(
            !semantics_array.is_null(ms_row),
            "Expected non-null semantics for MultiSurface"
        );

        // --- Test GeometryInstance geometry ---
        let gi_row = find_row_by_type("GeometryInstance");

        // Verify template reference
        let template_array = batch
            .column(12)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .expect("Expected UInt32Array for template references");
        assert!(
            !template_array.is_null(gi_row),
            "Expected non-null template reference"
        );
        assert_eq!(
            template_array.value(gi_row),
            template_ref.index(),
            "Expected correct template reference"
        );

        // Verify transformation matrix
        let matrix_array = batch
            .column(14)
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .expect("Expected FixedSizeListArray for transformation matrices");
        assert!(
            !matrix_array.is_null(gi_row),
            "Expected non-null transformation matrix"
        );

        let matrix = matrix_array.value(gi_row);
        let matrix_values = matrix
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("Expected Float64Array for matrix values");
        assert_eq!(
            matrix_values.len(),
            16,
            "Expected 16 values in transformation matrix"
        );
        assert_eq!(matrix_values.value(0), 2.0, "Expected scale X = 2.0");
        assert_eq!(matrix_values.value(5), 2.0, "Expected scale Y = 2.0");
        assert_eq!(matrix_values.value(10), 2.0, "Expected scale Z = 2.0");
        assert_eq!(
            matrix_values.value(12),
            10.0,
            "Expected translation X = 10.0"
        );
        assert_eq!(
            matrix_values.value(13),
            20.0,
            "Expected translation Y = 20.0"
        );
        assert_eq!(
            matrix_values.value(14),
            30.0,
            "Expected translation Z = 30.0"
        );
    }

    // TODO: This test needs to be updated to work with the new AttributePool-based API
    #[cfg(any())]
    #[test]
    fn test_arrow_to_geometries() {
        // ----- STEP 1: Create test Arrow RecordBatch -----

        // Define schema fields for the batch
        let fields = vec![
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
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                true,
            ),
            Field::new(
                "boundary_rings",
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                true,
            ),
            Field::new(
                "boundary_surfaces",
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                true,
            ),
            Field::new(
                "boundary_shells",
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                true,
            ),
            Field::new(
                "boundary_solids",
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                true,
            ),
            Field::new(
                "semantics_points",
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
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
            Field::new(
                "materials",
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                true,
            ),
            Field::new("instance_template", DataType::UInt32, true),
            Field::new("instance_reference_point", DataType::UInt32, true),
            Field::new(
                "instance_transformation_matrix",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, true)), 16),
                true,
            ),
        ];

        let schema = Schema::new(fields);

        // Create arrays for three different geometries
        // 1. MultiPoint geometry
        // 2. MultiSurface geometry
        // 3. GeometryInstance

        // ID array
        let id_array = UInt32Array::from(vec![1, 2, 3]);

        // Type array (dictionary encoded)
        let type_keys = Int8Array::from(vec![0, 1, 2]);
        let type_values = StringArray::from(vec!["MultiPoint", "MultiSurface", "GeometryInstance"]);
        let type_array = DictionaryArray::try_new(type_keys, Arc::new(type_values)).unwrap();

        // LoD array (dictionary encoded with null)
        let lod_keys = Int8Array::from(vec![0, 1, 0]);
        let lod_values = StringArray::from(vec!["1", "2"]);
        // Create with validity bitmap to represent the null in position 2
        let validity = Buffer::from_slice_ref(&[0b00000011]);
        let lod_array_data = arrow::array::ArrayData::try_new(
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            3,
            Some(validity),
            0,
            vec![Buffer::from(lod_keys.to_data().buffers()[0].clone())],
            vec![lod_values.to_data()],
        )
        .unwrap();
        let lod_array: DictionaryArray<Int8Type> = DictionaryArray::from(lod_array_data);

        // Boundary vertices array
        // MultiPoint vertices: [10, 20]
        // MultiSurface vertices: [0, 1, 2, 3, 0]
        // GeometryInstance - null (no boundary)
        let _mp_vertices = UInt32Array::from(vec![10, 20]);
        let _ms_vertices = UInt32Array::from(vec![0, 1, 2, 3, 0]);

        let offsets = Int32Array::from(vec![0, 2, 7, 7]);
        let values = UInt32Array::from(vec![10, 20, 0, 1, 2, 3, 0]);

        // Create validity buffer for nulls (third entry is null)
        let list_validity = Buffer::from_slice_ref(&[0b00000011]);

        let vertices_array_data = arrow::array::ArrayData::try_new(
            DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
            3,
            Some(list_validity),
            0,
            vec![Buffer::from(offsets.to_data().buffers()[0].clone())],
            vec![values.to_data()],
        )
        .unwrap();
        let vertices_array = ListArray::from(vertices_array_data);

        // Boundary rings array
        // MultiPoint - null (no rings)
        // MultiSurface - [0] (one ring starting at index 0)
        // GeometryInstance - null (no rings)
        let ring_offsets = Int32Array::from(vec![0, 0, 1, 1]);
        let ring_values = UInt32Array::from(vec![0]);

        let ring_validity = Buffer::from_slice_ref(&[0b00000010]);

        let rings_array_data = arrow::array::ArrayData::try_new(
            DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
            3,
            Some(ring_validity),
            0,
            vec![Buffer::from(ring_offsets.to_data().buffers()[0].clone())],
            vec![ring_values.to_data()],
        )
        .unwrap();
        let rings_array = ListArray::from(rings_array_data);

        // Boundary surfaces array
        // MultiPoint - null (no surfaces)
        // MultiSurface - [0] (one surface starting at ring index 0)
        // GeometryInstance - null (no surfaces)
        let surface_offsets = Int32Array::from(vec![0, 0, 1, 1]);
        let surface_values = UInt32Array::from(vec![0]);

        let surface_validity = Buffer::from_slice_ref(&[0b00000010]);

        let surfaces_array_data = arrow::array::ArrayData::try_new(
            DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
            3,
            Some(surface_validity),
            0,
            vec![Buffer::from(surface_offsets.to_data().buffers()[0].clone())],
            vec![surface_values.to_data()],
        )
        .unwrap();
        let surfaces_array = ListArray::from(surfaces_array_data);

        // Create null arrays for the remaining boundary arrays
        // Shells, solids, semantics, etc.
        let null_offsets = Int32Array::from(vec![0, 0, 0, 0]);
        let null_values = UInt32Array::from(Vec::<u32>::new());

        // All entries null
        let null_validity = Buffer::from_slice_ref(&[0b00000000]);

        let null_list_data = arrow::array::ArrayData::try_new(
            DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
            3,
            Some(null_validity.clone()),
            0,
            vec![Buffer::from(null_offsets.to_data().buffers()[0].clone())],
            vec![null_values.to_data()],
        )
        .unwrap();

        let shells_array = ListArray::from(null_list_data.clone());
        let solids_array = ListArray::from(null_list_data.clone());
        let semantics_points_array = ListArray::from(null_list_data.clone());
        let semantics_linestrings_array = ListArray::from(null_list_data.clone());
        let semantics_surfaces_array = ListArray::from(null_list_data.clone());
        let materials_array = ListArray::from(null_list_data);

        // Instance template array (null for 0,1, value 42 for index 2)
        let template_values = UInt32Array::from(vec![0, 0, 42]);
        let template_validity = Buffer::from_slice_ref(&[0b00000100]);

        let template_array_data = arrow::array::ArrayData::try_new(
            DataType::UInt32,
            3,
            Some(template_validity),
            0,
            vec![Buffer::from(template_values.to_data().buffers()[0].clone())],
            vec![],
        )
        .unwrap();
        let instance_template_array = UInt32Array::from(template_array_data);

        // Instance reference point array (null for 0,1, value 5 for index 2)
        let ref_point_values = UInt32Array::from(vec![0, 0, 5]);
        let ref_point_validity = Buffer::from_slice_ref(&[0b00000100]);

        let ref_point_array_data = arrow::array::ArrayData::try_new(
            DataType::UInt32,
            3,
            Some(ref_point_validity),
            0,
            vec![Buffer::from(
                ref_point_values.to_data().buffers()[0].clone(),
            )],
            vec![],
        )
        .unwrap();
        let instance_reference_point_array = UInt32Array::from(ref_point_array_data);

        // Instance transformation matrix array
        // Create a scale + translation matrix for the third geometry
        let identity_matrix = vec![
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let scale_matrix = vec![
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 10.0, 20.0, 30.0, 1.0,
        ];

        // Create a single array with all matrices (2 null, 1 valid)
        let all_matrices = [identity_matrix.clone(), identity_matrix, scale_matrix].concat();
        let matrix_values = Float64Array::from(all_matrices);

        let matrix_validity = Buffer::from_slice_ref(&[0b00000100]);

        // Create fixed size list array for matrices
        let matrix_array_data = arrow::array::ArrayData::try_new(
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, true)), 16),
            3,
            Some(matrix_validity),
            0,
            vec![],
            vec![matrix_values.to_data()],
        )
        .unwrap();

        let matrix_array = FixedSizeListArray::from(matrix_array_data);

        // Combine all arrays into a RecordBatch
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(id_array),
            Arc::new(type_array),
            Arc::new(lod_array),
            Arc::new(vertices_array),
            Arc::new(rings_array),
            Arc::new(surfaces_array),
            Arc::new(shells_array),
            Arc::new(solids_array),
            Arc::new(semantics_points_array),
            Arc::new(semantics_linestrings_array),
            Arc::new(semantics_surfaces_array),
            Arc::new(materials_array),
            Arc::new(instance_template_array),
            Arc::new(instance_reference_point_array),
            Arc::new(matrix_array),
        ];

        let batch = RecordBatch::try_new(Arc::new(schema), arrays).unwrap();

        // ----- STEP 2: Call arrow_to_geometries function -----
        let geometry_pool = arrow_to_geometries::<OwnedStringStorage>(&batch).unwrap();

        // ----- STEP 3: Verify results -----

        // Check we have three geometries
        assert_eq!(geometry_pool.len(), 3, "Expected 3 geometries in the pool");

        // Find each geometry by type
        let types_count = geometry_pool
            .iter()
            .fold([0, 0, 0], |mut counts, (_id, geom)| {
                match geom.type_geometry() {
                    GeometryType::MultiPoint => {
                        counts[0] += 1;
                    }
                    GeometryType::MultiSurface => {
                        counts[1] += 1;
                    }
                    GeometryType::GeometryInstance => {
                        counts[2] += 1;
                    }
                    _ => {}
                }
                counts
            });

        assert_eq!(types_count, [1, 1, 1], "Expected one geometry of each type");

        // Check each geometry in detail
        for (_id, geom) in geometry_pool.iter() {
            match geom.type_geometry() {
                GeometryType::MultiPoint => {
                    // Verify MultiPoint properties
                    assert_eq!(geom.lod(), Some(&LoD::LoD1), "Expected LoD1 for MultiPoint");

                    // Check boundary vertices
                    let boundary = geom.boundaries().expect("MultiPoint should have boundary");
                    let vertices = boundary.vertices();
                    assert_eq!(vertices.len(), 2, "Expected 2 vertices in MultiPoint");
                    assert_eq!(vertices[0].value(), 10, "First vertex should be 10");
                    assert_eq!(vertices[1].value(), 20, "Second vertex should be 20");

                    // Should have empty rings/surfaces/etc.
                    assert!(
                        boundary.rings().is_empty(),
                        "MultiPoint should have no rings"
                    );
                    assert!(
                        boundary.surfaces().is_empty(),
                        "MultiPoint should have no surfaces"
                    );
                }
                GeometryType::MultiSurface => {
                    // Verify MultiSurface properties
                    assert_eq!(
                        geom.lod(),
                        Some(&LoD::LoD2),
                        "Expected LoD2 for MultiSurface"
                    );

                    // Check boundary structure
                    let boundary = geom
                        .boundaries()
                        .expect("MultiSurface should have boundary");

                    // Check vertices
                    let vertices = boundary.vertices();
                    assert_eq!(vertices.len(), 5, "Expected 5 vertices in MultiSurface");

                    // Check ring
                    let rings = boundary.rings();
                    assert_eq!(rings.len(), 1, "Expected 1 ring in MultiSurface");
                    assert_eq!(rings[0].value(), 0, "Ring should start at vertex 0");

                    // Check surface
                    let surfaces = boundary.surfaces();
                    assert_eq!(surfaces.len(), 1, "Expected 1 surface in MultiSurface");
                    assert_eq!(surfaces[0].value(), 0, "Surface should start at ring 0");
                }
                GeometryType::GeometryInstance => {
                    // Verify GeometryInstance properties
                    assert_eq!(geom.lod(), None, "Expected no LoD for GeometryInstance");

                    // Check instance properties
                    let template = geom.instance_template().expect("Should have template");
                    assert_eq!(template.index(), 42, "Template ID should be 42");

                    let ref_point = geom
                        .instance_reference_point()
                        .expect("Should have reference point");
                    assert_eq!(ref_point.value(), 5, "Reference point should be 5");

                    let matrix = geom
                        .instance_transformation_matrix()
                        .expect("Should have matrix");
                    assert_eq!(matrix[0], 2.0, "Matrix[0] should be 2.0 (x scale)");
                    assert_eq!(matrix[5], 2.0, "Matrix[5] should be 2.0 (y scale)");
                    assert_eq!(matrix[10], 2.0, "Matrix[10] should be 2.0 (z scale)");
                    assert_eq!(
                        matrix[12], 10.0,
                        "Matrix[12] should be 10.0 (x translation)"
                    );
                    assert_eq!(
                        matrix[13], 20.0,
                        "Matrix[13] should be 20.0 (y translation)"
                    );
                    assert_eq!(
                        matrix[14], 30.0,
                        "Matrix[14] should be 30.0 (z translation)"
                    );

                    // Should not have a boundary
                    assert!(
                        geom.boundaries().is_none(),
                        "GeometryInstance should not have boundary"
                    );
                }
                _ => panic!("Unexpected geometry type: {:?}", geom.type_geometry()),
            }
        }
    }
}
