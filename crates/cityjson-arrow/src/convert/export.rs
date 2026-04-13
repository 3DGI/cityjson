#![allow(clippy::wildcard_imports)]

use super::*;

pub(crate) fn encode_parts(model: &OwnedCityModel) -> Result<CityModelArrowParts> {
    let mut sink = PartsSink::default();
    emit_tables(model, &mut sink)?;
    sink.finish()
}

#[allow(clippy::too_many_lines)]
pub(crate) fn emit_tables<S: CanonicalTableSink>(
    model: &OwnedCityModel,
    sink: &mut S,
) -> Result<()> {
    reject_unsupported_modules(model)?;
    let context = build_export_context(model)?;
    sink.start(&context.header, &context.projection)?;

    let core = export_core_batches(&context)?;
    sink.push_batch(CanonicalTable::Metadata, core.metadata)?;
    push_optional_batch(sink, CanonicalTable::Transform, core.transform)?;
    push_optional_batch(sink, CanonicalTable::Extensions, core.extensions)?;
    sink.push_batch(CanonicalTable::Vertices, core.vertices)?;

    let ExportedGeometryTables {
        geometries,
        boundaries,
        instances,
        surface_semantics,
        point_semantics,
        linestring_semantics,
        surface_materials,
        ring_textures,
    } = geometry_tables(context.model)?;
    let ExportedTemplateGeometryTables {
        geometries: template_geometries,
        boundaries: template_geometry_boundaries,
        semantics: template_geometry_semantics,
        materials: template_geometry_materials,
        ring_textures: template_geometry_ring_textures,
    } = template_geometry_tables(context.model)?;
    let geometry = export_geometry_batches(
        &context,
        geometries,
        boundaries,
        instances,
        template_geometries,
        template_geometry_boundaries,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateVertices,
        geometry.template_vertices,
    )?;

    let semantics = export_semantic_batches(
        &context,
        surface_semantics,
        point_semantics,
        linestring_semantics,
        template_geometry_semantics,
    )?;
    let appearance = export_appearance_batches(
        &context,
        surface_materials,
        ring_textures,
        template_geometry_materials,
        template_geometry_ring_textures,
    )?;

    push_optional_batch(
        sink,
        CanonicalTable::TextureVertices,
        appearance.texture_vertices,
    )?;
    push_optional_batch(sink, CanonicalTable::Semantics, semantics.semantics)?;
    push_optional_batch(
        sink,
        CanonicalTable::SemanticChildren,
        semantics.semantic_children,
    )?;
    push_optional_batch(sink, CanonicalTable::Materials, appearance.materials)?;
    push_optional_batch(sink, CanonicalTable::Textures, appearance.textures)?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometryBoundaries,
        geometry.template_geometry_boundaries,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometrySemantics,
        semantics.template_geometry_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometryMaterials,
        appearance.template_geometry_materials,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometryRingTextures,
        appearance.template_geometry_ring_textures,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometries,
        geometry.template_geometries,
    )?;
    sink.push_batch(
        CanonicalTable::GeometryBoundaries,
        geometry.geometry_boundaries,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometrySurfaceSemantics,
        semantics.geometry_surface_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryPointSemantics,
        semantics.geometry_point_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryLinestringSemantics,
        semantics.geometry_linestring_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometrySurfaceMaterials,
        appearance.geometry_surface_materials,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryRingTextures,
        appearance.geometry_ring_textures,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryInstances,
        geometry.geometry_instances,
    )?;
    sink.push_batch(CanonicalTable::Geometries, geometry.geometries)?;
    sink.push_batch(CanonicalTable::CityObjects, core.cityobjects)?;
    push_optional_batch(
        sink,
        CanonicalTable::CityObjectChildren,
        core.cityobject_children,
    )?;

    Ok(())
}

pub(crate) fn emit_part_tables<S: CanonicalTableSink>(
    parts: &CityModelArrowParts,
    sink: &mut S,
) -> Result<()> {
    sink.start(&parts.header, &parts.projection)?;
    for (table, batch) in collect_tables(parts) {
        sink.push_batch(table, batch)?;
    }
    Ok(())
}

pub(crate) fn build_parts_from_tables(
    header: &CityArrowHeader,
    projection: &ProjectionLayout,
    tables: Vec<(CanonicalTable, RecordBatch)>,
) -> Result<CityModelArrowParts> {
    let mut sink = PartsSink::default();
    sink.start(header, projection)?;
    for (table, batch) in tables {
        sink.push_batch(table, batch)?;
    }
    sink.finish()
}

fn build_export_context(model: &OwnedCityModel) -> Result<ExportContext<'_>> {
    let citymodel_id = infer_citymodel_id(model);
    let projection = discover_projection_layout(model)?;
    Ok(ExportContext {
        model,
        header: CityArrowHeader::new(
            CityArrowPackageVersion::V3Alpha2,
            citymodel_id,
            model
                .version()
                .unwrap_or(cityjson::CityJSONVersion::V2_0)
                .to_string(),
        ),
        projection: projection.clone(),
        schemas: canonical_schema_set(&projection),
    })
}

fn push_optional_batch<S: CanonicalTableSink>(
    sink: &mut S,
    table: CanonicalTable,
    batch: Option<RecordBatch>,
) -> Result<()> {
    if let Some(batch) = batch {
        sink.push_batch(table, batch)?;
    }
    Ok(())
}

impl CanonicalTableSink for PartsSink {
    fn start(&mut self, header: &CityArrowHeader, projection: &ProjectionLayout) -> Result<()> {
        self.header = Some(header.clone());
        self.projection = Some(projection.clone());
        Ok(())
    }

    fn push_batch(&mut self, table: CanonicalTable, batch: RecordBatch) -> Result<()> {
        let slot = match table {
            CanonicalTable::Metadata => &mut self.metadata,
            CanonicalTable::Transform => &mut self.transform,
            CanonicalTable::Extensions => &mut self.extensions,
            CanonicalTable::Vertices => &mut self.vertices,
            CanonicalTable::TemplateVertices => &mut self.template_vertices,
            CanonicalTable::TextureVertices => &mut self.texture_vertices,
            CanonicalTable::Semantics => &mut self.semantics,
            CanonicalTable::SemanticChildren => &mut self.semantic_children,
            CanonicalTable::Materials => &mut self.materials,
            CanonicalTable::Textures => &mut self.textures,
            CanonicalTable::TemplateGeometryBoundaries => &mut self.template_geometry_boundaries,
            CanonicalTable::TemplateGeometrySemantics => &mut self.template_geometry_semantics,
            CanonicalTable::TemplateGeometryMaterials => &mut self.template_geometry_materials,
            CanonicalTable::TemplateGeometryRingTextures => {
                &mut self.template_geometry_ring_textures
            }
            CanonicalTable::TemplateGeometries => &mut self.template_geometries,
            CanonicalTable::GeometryBoundaries => &mut self.geometry_boundaries,
            CanonicalTable::GeometrySurfaceSemantics => &mut self.geometry_surface_semantics,
            CanonicalTable::GeometryPointSemantics => &mut self.geometry_point_semantics,
            CanonicalTable::GeometryLinestringSemantics => &mut self.geometry_linestring_semantics,
            CanonicalTable::GeometrySurfaceMaterials => &mut self.geometry_surface_materials,
            CanonicalTable::GeometryRingTextures => &mut self.geometry_ring_textures,
            CanonicalTable::GeometryInstances => &mut self.geometry_instances,
            CanonicalTable::Geometries => &mut self.geometries,
            CanonicalTable::CityObjects => &mut self.cityobjects,
            CanonicalTable::CityObjectChildren => &mut self.cityobject_children,
        };
        assign_table_slot(slot, table, batch)
    }
}

impl PartsSink {
    fn finish(self) -> Result<CityModelArrowParts> {
        Ok(CityModelArrowParts {
            header: self
                .header
                .ok_or_else(|| Error::Conversion("missing canonical table header".to_string()))?,
            projection: self.projection.ok_or_else(|| {
                Error::Conversion("missing canonical table projection".to_string())
            })?,
            metadata: required_batch(self.metadata, CanonicalTable::Metadata)?,
            transform: self.transform,
            extensions: self.extensions,
            vertices: required_batch(self.vertices, CanonicalTable::Vertices)?,
            cityobjects: required_batch(self.cityobjects, CanonicalTable::CityObjects)?,
            cityobject_children: self.cityobject_children,
            geometries: required_batch(self.geometries, CanonicalTable::Geometries)?,
            geometry_boundaries: required_batch(
                self.geometry_boundaries,
                CanonicalTable::GeometryBoundaries,
            )?,
            geometry_instances: self.geometry_instances,
            template_vertices: self.template_vertices,
            template_geometries: self.template_geometries,
            template_geometry_boundaries: self.template_geometry_boundaries,
            semantics: self.semantics,
            semantic_children: self.semantic_children,
            geometry_surface_semantics: self.geometry_surface_semantics,
            geometry_point_semantics: self.geometry_point_semantics,
            geometry_linestring_semantics: self.geometry_linestring_semantics,
            template_geometry_semantics: self.template_geometry_semantics,
            materials: self.materials,
            geometry_surface_materials: self.geometry_surface_materials,
            template_geometry_materials: self.template_geometry_materials,
            textures: self.textures,
            texture_vertices: self.texture_vertices,
            geometry_ring_textures: self.geometry_ring_textures,
            template_geometry_ring_textures: self.template_geometry_ring_textures,
        })
    }
}

fn assign_table_slot(
    slot: &mut Option<RecordBatch>,
    table: CanonicalTable,
    batch: RecordBatch,
) -> Result<()> {
    if slot.replace(batch).is_some() {
        return Err(Error::Unsupported(format!(
            "duplicate '{}' canonical table batch",
            table.as_str()
        )));
    }
    Ok(())
}

fn required_batch(batch: Option<RecordBatch>, table: CanonicalTable) -> Result<RecordBatch> {
    batch.ok_or_else(|| {
        Error::Unsupported(format!(
            "package or stream is missing required '{}' table",
            table.as_str()
        ))
    })
}

fn export_core_batches(context: &ExportContext<'_>) -> Result<ExportCoreBatches> {
    let metadata = metadata_batch(
        &context.schemas.metadata,
        metadata_row(context.model, &context.header),
        &context.projection,
    )?;
    let transform_row = context.model.transform().map(|transform| TransformRow {
        scale: transform.scale(),
        translate: transform.translate(),
    });

    Ok(ExportCoreBatches {
        metadata,
        transform: transform_row
            .map(|row| transform_batch(&context.schemas.transform, row))
            .transpose()?,
        extensions: extensions_batch_from_model(&context.schemas.extensions, context.model)?,
        vertices: vertices_batch_from_model(&context.schemas.vertices, context.model)?,
        cityobjects: cityobjects_batch_from_model(
            &context.schemas.cityobjects,
            context.model,
            &context.projection,
        )?,
        cityobject_children: cityobject_children_batch_from_model(
            &context.schemas.cityobject_children,
            context.model,
        )?,
    })
}

fn export_geometry_batches(
    context: &ExportContext<'_>,
    geometries: GeometryTableBuffer,
    geometry_boundaries: GeometryBoundaryTableBuffer,
    geometry_instances: GeometryInstanceTableBuffer,
    template_geometries: TemplateGeometryTableBuffer,
    template_geometry_boundaries: TemplateGeometryBoundaryTableBuffer,
) -> Result<ExportGeometryBatches> {
    Ok(ExportGeometryBatches {
        geometries: geometries_batch(&context.schemas.geometries, geometries)?,
        geometry_boundaries: geometry_boundaries_batch(
            &context.schemas.geometry_boundaries,
            geometry_boundaries,
        )?,
        geometry_instances: optional_batch_from(geometry_instances.is_empty(), || {
            geometry_instances_batch(&context.schemas.geometry_instances, geometry_instances)
        })?,
        template_vertices: template_vertices_batch_from_model(
            &context.schemas.template_vertices,
            context.model,
        )?,
        template_geometries: optional_batch_from(template_geometries.is_empty(), || {
            template_geometries_batch(&context.schemas.template_geometries, template_geometries)
        })?,
        template_geometry_boundaries: optional_batch_from(
            template_geometry_boundaries.is_empty(),
            || {
                template_geometry_boundaries_batch(
                    &context.schemas.template_geometry_boundaries,
                    template_geometry_boundaries,
                )
            },
        )?,
    })
}

fn export_semantic_batches(
    context: &ExportContext<'_>,
    geometry_surface_semantics: GeometrySurfaceSemanticTableBuffer,
    geometry_point_semantics: GeometryPointSemanticTableBuffer,
    geometry_linestring_semantics: GeometryLinestringSemanticTableBuffer,
    template_geometry_semantics: TemplateGeometrySemanticTableBuffer,
) -> Result<ExportSemanticBatches> {
    Ok(ExportSemanticBatches {
        semantics: semantics_batch_from_model(
            &context.schemas.semantics,
            context.model,
            &context.projection,
        )?,
        semantic_children: semantic_children_batch_from_model(
            &context.schemas.semantic_children,
            context.model,
        )?,
        geometry_surface_semantics: optional_batch_from(
            geometry_surface_semantics.is_empty(),
            || {
                geometry_surface_semantics_batch(
                    &context.schemas.geometry_surface_semantics,
                    geometry_surface_semantics,
                )
            },
        )?,
        geometry_point_semantics: optional_batch_from(geometry_point_semantics.is_empty(), || {
            geometry_point_semantics_batch(
                &context.schemas.geometry_point_semantics,
                geometry_point_semantics,
            )
        })?,
        geometry_linestring_semantics: optional_batch_from(
            geometry_linestring_semantics.is_empty(),
            || {
                geometry_linestring_semantics_batch(
                    &context.schemas.geometry_linestring_semantics,
                    geometry_linestring_semantics,
                )
            },
        )?,
        template_geometry_semantics: optional_batch_from(
            template_geometry_semantics.is_empty(),
            || {
                template_geometry_semantics_batch(
                    &context.schemas.template_geometry_semantics,
                    template_geometry_semantics,
                )
            },
        )?,
    })
}

fn export_appearance_batches(
    context: &ExportContext<'_>,
    geometry_surface_materials: GeometrySurfaceMaterialTableBuffer,
    geometry_ring_textures: GeometryRingTextureTableBuffer,
    template_geometry_materials: TemplateGeometryMaterialTableBuffer,
    template_geometry_ring_textures: TemplateGeometryRingTextureTableBuffer,
) -> Result<ExportAppearanceBatches> {
    Ok(ExportAppearanceBatches {
        materials: materials_batch_from_model(
            &context.schemas.materials,
            context.model,
            &context.projection,
        )?,
        geometry_surface_materials: optional_batch_from(
            geometry_surface_materials.is_empty(),
            || {
                geometry_surface_materials_batch(
                    &context.schemas.geometry_surface_materials,
                    geometry_surface_materials,
                )
            },
        )?,
        template_geometry_materials: optional_batch_from(
            template_geometry_materials.is_empty(),
            || {
                template_geometry_materials_batch(
                    &context.schemas.template_geometry_materials,
                    template_geometry_materials,
                )
            },
        )?,
        textures: textures_batch_from_model(
            &context.schemas.textures,
            context.model,
            &context.projection,
        )?,
        texture_vertices: texture_vertices_batch_from_model(
            &context.schemas.texture_vertices,
            context.model,
        )?,
        geometry_ring_textures: optional_batch_from(geometry_ring_textures.is_empty(), || {
            geometry_ring_textures_batch(
                &context.schemas.geometry_ring_textures,
                geometry_ring_textures,
            )
        })?,
        template_geometry_ring_textures: optional_batch_from(
            template_geometry_ring_textures.is_empty(),
            || {
                template_geometry_ring_textures_batch(
                    &context.schemas.template_geometry_ring_textures,
                    template_geometry_ring_textures,
                )
            },
        )?,
    })
}

fn reject_unsupported_modules(model: &OwnedCityModel) -> Result<()> {
    for (_, geometry) in model.iter_geometries() {
        if geometry.textures().is_some() {
            ensure_surface_backed_geometry(*geometry.type_geometry(), "geometry textures")?;
        }
    }
    for (_, geometry) in model.iter_geometry_templates() {
        if geometry.instance().is_some() {
            return Err(Error::Unsupported(
                "geometry instances in template geometry pool".to_string(),
            ));
        }
        if geometry.textures().is_some() {
            ensure_surface_backed_geometry(
                *geometry.type_geometry(),
                "template geometry textures",
            )?;
        }
    }
    Ok(())
}

fn infer_citymodel_id(model: &OwnedCityModel) -> String {
    model
        .metadata()
        .and_then(|metadata| metadata.identifier().map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CITYMODEL_ID.to_string())
}

pub(super) fn usize_to_u32(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value)
        .map_err(|_| Error::Conversion(format!("{label} {value} does not fit into u32")))
}

pub(super) fn usize_to_i32(value: usize, label: &str) -> Result<i32> {
    i32::try_from(value)
        .map_err(|_| Error::Conversion(format!("{label} {value} does not fit into i32")))
}

pub(super) trait RawPartsHandle {
    fn raw_parts(self) -> (u32, u16);
}

impl RawPartsHandle for cityjson::prelude::GeometryHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::GeometryTemplateHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::SemanticHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::MaterialHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::TextureHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

pub(super) fn raw_id_from_handle(handle: impl RawPartsHandle) -> u64 {
    let (index, generation) = handle.raw_parts();
    (u64::from(index) << 16) | u64::from(generation)
}

fn metadata_row(model: &OwnedCityModel, header: &CityArrowHeader) -> MetadataRow {
    let metadata = model.metadata();
    MetadataRow {
        citymodel_id: header.citymodel_id.clone(),
        cityjson_version: header.cityjson_version.clone(),
        citymodel_kind: model.type_citymodel().to_string(),
        feature_root_id: model.id().and_then(|handle| {
            model
                .cityobjects()
                .get(handle)
                .map(|cityobject| cityobject.id().to_string())
        }),
        identifier: metadata.and_then(|item| item.identifier().map(ToString::to_string)),
        title: metadata.and_then(Metadata::title).map(ToString::to_string),
        reference_system: metadata
            .and_then(|item| item.reference_system().map(ToString::to_string)),
        geographical_extent: metadata
            .and_then(Metadata::geographical_extent)
            .map(|bbox| bbox.as_slice().try_into().expect("bbox is 6 long")),
        reference_date: metadata
            .and_then(Metadata::reference_date)
            .map(ToString::to_string),
        default_material_theme: model.default_material_theme().map(ToString::to_string),
        default_texture_theme: model.default_texture_theme().map(ToString::to_string),
        point_of_contact: metadata
            .and_then(Metadata::point_of_contact)
            .map(|contact| MetadataContactRow {
                contact_name: contact.contact_name().to_string(),
                email_address: contact.email_address().to_string(),
                role: contact.role().map(|value| value.to_string()),
                website: contact.website().clone(),
                contact_type: contact.contact_type().map(|value| value.to_string()),
                phone: contact.phone().clone(),
                organization: contact.organization().clone(),
                address: contact.address().cloned(),
            }),
        root_extra: cloned_attributes(model.extra()),
        metadata_extra: metadata.and_then(Metadata::extra).cloned(),
    }
}

fn cityobject_ix_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::CityObjectHandle, u64> {
    model
        .cityobjects()
        .iter()
        .enumerate()
        .map(|(index, (handle, _))| {
            (
                handle,
                u64::try_from(index).expect("cityobject index fits into u64"),
            )
        })
        .collect()
}

fn extensions_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    let Some(extensions) = model.extensions() else {
        return Ok(None);
    };
    if extensions.is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(
                extensions
                    .iter()
                    .map(|extension| Some(extension.name().clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LargeStringArray::from(
                extensions
                    .iter()
                    .map(|extension| Some(extension.url().clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                extensions
                    .iter()
                    .map(|extension| Some(extension.version().clone()))
                    .collect::<Vec<_>>(),
            )),
        ],
    )?))
}

fn vertices_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<RecordBatch> {
    vertex_batch_from_coordinates(schema, model.vertices().as_slice(), "vertex_id")
}

fn cityobjects_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut cityobject_id = Vec::with_capacity(model.cityobjects().len());
    let mut cityobject_index = Vec::with_capacity(model.cityobjects().len());
    let mut object_type = Vec::with_capacity(model.cityobjects().len());
    let mut geographical_extent: Vec<Option<[f64; 6]>> =
        Vec::with_capacity(model.cityobjects().len());
    let mut attributes = Vec::with_capacity(model.cityobjects().len());
    let mut extra = Vec::with_capacity(model.cityobjects().len());

    for (index, (_, object)) in model.cityobjects().iter().enumerate() {
        cityobject_id.push(Some(object.id().to_string()));
        cityobject_index.push(u64::try_from(index).expect("cityobject index fits into u64"));
        object_type.push(Some(object.type_cityobject().to_string()));
        geographical_extent.push(
            object
                .geographical_extent()
                .map(|bbox| bbox.as_slice().try_into().expect("bbox is 6 long")),
        );
        attributes.push(non_empty_attributes(object.attributes()));
        extra.push(non_empty_attributes(object.extra()));
    }

    let mut fields = SchemaFieldLookup::new(schema);
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(cityobject_id)),
        Arc::new(UInt64Array::from(cityobject_index)),
        Arc::new(StringArray::from(object_type)),
        Arc::new(fixed_size_f64_array(
            &fields.field("geographical_extent")?,
            6,
            geographical_extent,
        )?),
    ];

    if let Some(spec) = projection.cityobject_attributes.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("attributes")?,
            spec,
            &attributes,
        )?);
    }
    if let Some(spec) = projection.cityobject_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("extra")?,
            spec,
            &extra,
        )?);
    }

    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn cityobject_children_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    let cityobject_ix_map = cityobject_ix_map(model);
    let mut parent_cityobject_ix = Vec::new();
    let mut child_ordinal = Vec::new();
    let mut child_cityobject_ix = Vec::new();
    for (parent_handle, object) in model.cityobjects().iter() {
        let parent_ix = cityobject_ix_map
            .get(&parent_handle)
            .copied()
            .unwrap_or_default();
        if let Some(children) = object.children() {
            for (ordinal, child) in children.iter().enumerate() {
                if let Some(child_ix) = cityobject_ix_map.get(child).copied() {
                    parent_cityobject_ix.push(parent_ix);
                    child_ordinal.push(usize_to_u32(ordinal, "child ordinal")?);
                    child_cityobject_ix.push(child_ix);
                }
            }
        }
    }
    if parent_cityobject_ix.is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(parent_cityobject_ix)),
            Arc::new(UInt32Array::from(child_ordinal)),
            Arc::new(UInt64Array::from(child_cityobject_ix)),
        ],
    )?))
}

fn template_vertices_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    if model.template_vertices().as_slice().is_empty() {
        return Ok(None);
    }
    Ok(Some(vertex_batch_from_coordinates(
        schema,
        model.template_vertices().as_slice(),
        "template_vertex_id",
    )?))
}

fn semantics_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<Option<RecordBatch>> {
    if model.semantic_count() == 0 {
        return Ok(None);
    }
    let semantic_type = model
        .iter_semantics()
        .map(|(_, semantic)| Some(encode_semantic_type(semantic.type_semantic())))
        .collect::<Vec<_>>();
    let semantic_id = model
        .iter_semantics()
        .map(|(handle, _)| raw_id_from_handle(handle))
        .collect::<Vec<_>>();
    let parent_semantic_id = model
        .iter_semantics()
        .map(|(_, semantic)| semantic.parent().map(raw_id_from_handle))
        .collect::<Vec<_>>();
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(semantic_id)),
        Arc::new(StringArray::from(semantic_type)),
        Arc::new(UInt64Array::from(parent_semantic_id)),
    ];
    if let Some(spec) = projection.semantic_attributes.as_ref() {
        let attrs = model
            .iter_semantics()
            .map(|(_, semantic)| semantic.attributes())
            .collect::<Vec<_>>();
        arrays.push(projected_struct_array_from_attributes(
            &field_from_schema(schema, "attributes")?,
            spec,
            &attrs,
        )?);
    }
    Ok(Some(RecordBatch::try_new(schema.clone(), arrays)?))
}

fn semantic_children_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    let mut parent_semantic_id = Vec::new();
    let mut child_ordinal = Vec::new();
    let mut child_semantic_id = Vec::new();
    for (handle, semantic) in model.iter_semantics() {
        if let Some(children) = semantic.children() {
            let parent_id = raw_id_from_handle(handle);
            for (ordinal, child) in children.iter().enumerate() {
                let child_id = raw_id_from_handle(*child);
                parent_semantic_id.push(parent_id);
                child_ordinal.push(usize_to_u32(ordinal, "child ordinal")?);
                child_semantic_id.push(child_id);
            }
        }
    }
    if parent_semantic_id.is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(parent_semantic_id)),
            Arc::new(UInt32Array::from(child_ordinal)),
            Arc::new(UInt64Array::from(child_semantic_id)),
        ],
    )?))
}

fn materials_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<Option<RecordBatch>> {
    if model.material_count() == 0 {
        return Ok(None);
    }
    let mut material_id = Vec::with_capacity(model.material_count());
    let mut name = Vec::with_capacity(model.material_count());
    let mut ambient_intensity = Vec::with_capacity(model.material_count());
    let mut diffuse_color = Vec::with_capacity(model.material_count());
    let mut emissive_color = Vec::with_capacity(model.material_count());
    let mut specular_color = Vec::with_capacity(model.material_count());
    let mut shininess = Vec::with_capacity(model.material_count());
    let mut transparency = Vec::with_capacity(model.material_count());
    let mut is_smooth = Vec::with_capacity(model.material_count());

    for (handle, material) in model.iter_materials() {
        material_id.push(raw_id_from_handle(handle));
        name.push(Some(material.name().clone()));
        ambient_intensity.push(material.ambient_intensity().map(f64::from));
        diffuse_color.push(material.diffuse_color().map(rgb_to_components));
        emissive_color.push(material.emissive_color().map(rgb_to_components));
        specular_color.push(material.specular_color().map(rgb_to_components));
        shininess.push(material.shininess().map(f64::from));
        transparency.push(material.transparency().map(f64::from));
        is_smooth.push(material.is_smooth());
    }

    let mut arrays: Vec<ArrayRef> = vec![Arc::new(UInt64Array::from(material_id))];
    if let Some(specs) = &projection.material_payload {
        for spec in &specs.fields {
            arrays.push(match spec.name.as_str() {
                FIELD_MATERIAL_NAME => Arc::new(LargeStringArray::from(name.clone())) as ArrayRef,
                FIELD_MATERIAL_AMBIENT_INTENSITY => {
                    Arc::new(Float64Array::from(ambient_intensity.clone())) as ArrayRef
                }
                FIELD_MATERIAL_DIFFUSE_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    diffuse_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                FIELD_MATERIAL_EMISSIVE_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    emissive_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                FIELD_MATERIAL_SPECULAR_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    specular_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                FIELD_MATERIAL_SHININESS => {
                    Arc::new(Float64Array::from(shininess.clone())) as ArrayRef
                }
                FIELD_MATERIAL_TRANSPARENCY => {
                    Arc::new(Float64Array::from(transparency.clone())) as ArrayRef
                }
                FIELD_MATERIAL_IS_SMOOTH => {
                    Arc::new(::arrow::array::BooleanArray::from(is_smooth.clone())) as ArrayRef
                }
                other => {
                    return Err(Error::Conversion(format!(
                        "unsupported material projection column {other}"
                    )));
                }
            });
        }
    }

    Ok(Some(RecordBatch::try_new(schema.clone(), arrays)?))
}

fn textures_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<Option<RecordBatch>> {
    if model.texture_count() == 0 {
        return Ok(None);
    }
    let mut texture_id = Vec::with_capacity(model.texture_count());
    let mut image_uri = Vec::with_capacity(model.texture_count());
    let mut image_type = Vec::with_capacity(model.texture_count());
    let mut wrap_mode = Vec::with_capacity(model.texture_count());
    let mut texture_type = Vec::with_capacity(model.texture_count());
    let mut border_color = Vec::with_capacity(model.texture_count());

    for (handle, texture) in model.iter_textures() {
        texture_id.push(raw_id_from_handle(handle));
        image_uri.push(Some(texture.image().clone()));
        image_type.push(Some(texture.image_type().to_string()));
        wrap_mode.push(texture.wrap_mode().map(|value| value.to_string()));
        texture_type.push(texture.texture_type().map(|value| value.to_string()));
        border_color.push(texture.border_color().map(rgba_to_components));
    }

    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(texture_id)),
        Arc::new(LargeStringArray::from(image_uri)),
    ];
    if let Some(specs) = &projection.texture_payload {
        for spec in &specs.fields {
            arrays.push(match spec.name.as_str() {
                FIELD_TEXTURE_IMAGE_TYPE => {
                    Arc::new(LargeStringArray::from(image_type.clone())) as ArrayRef
                }
                FIELD_TEXTURE_WRAP_MODE => {
                    Arc::new(LargeStringArray::from(wrap_mode.clone())) as ArrayRef
                }
                FIELD_TEXTURE_TEXTURE_TYPE => {
                    Arc::new(LargeStringArray::from(texture_type.clone())) as ArrayRef
                }
                FIELD_TEXTURE_BORDER_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    border_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                other => {
                    return Err(Error::Conversion(format!(
                        "unsupported texture projection column {other}"
                    )));
                }
            });
        }
    }
    Ok(Some(RecordBatch::try_new(schema.clone(), arrays)?))
}

fn texture_vertices_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    if model.vertices_texture().as_slice().is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                (0..model.vertices_texture().as_slice().len())
                    .map(|index| index as u64)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                model
                    .vertices_texture()
                    .as_slice()
                    .iter()
                    .map(cityjson::v2_0::UVCoordinate::u)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                model
                    .vertices_texture()
                    .as_slice()
                    .iter()
                    .map(cityjson::v2_0::UVCoordinate::v)
                    .collect::<Vec<_>>(),
            )),
        ],
    )?))
}

fn vertex_batch_from_coordinates(
    schema: &Arc<::arrow::datatypes::Schema>,
    coordinates: &[cityjson::v2_0::RealWorldCoordinate],
    _id_name: &str,
) -> Result<RecordBatch> {
    let count = coordinates.len();
    let mut ids = MutableBuffer::new(count * std::mem::size_of::<u64>());
    let mut x = MutableBuffer::new(count * std::mem::size_of::<f64>());
    let mut y = MutableBuffer::new(count * std::mem::size_of::<f64>());
    let mut z = MutableBuffer::new(count * std::mem::size_of::<f64>());

    for (index, coordinate) in coordinates.iter().enumerate() {
        ids.push(index as u64);
        x.push(coordinate.x());
        y.push(coordinate.y());
        z.push(coordinate.z());
    }

    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::new(ScalarBuffer::from(ids), None)),
            Arc::new(Float64Array::new(ScalarBuffer::from(x), None)),
            Arc::new(Float64Array::new(ScalarBuffer::from(y), None)),
            Arc::new(Float64Array::new(ScalarBuffer::from(z), None)),
        ],
    )
    .map_err(Error::from)
}

fn encode_semantic_type(
    semantic_type: &SemanticType<cityjson::prelude::OwnedStringStorage>,
) -> String {
    match semantic_type {
        SemanticType::Default => "Default".to_string(),
        SemanticType::RoofSurface => "RoofSurface".to_string(),
        SemanticType::GroundSurface => "GroundSurface".to_string(),
        SemanticType::WallSurface => "WallSurface".to_string(),
        SemanticType::ClosureSurface => "ClosureSurface".to_string(),
        SemanticType::OuterCeilingSurface => "OuterCeilingSurface".to_string(),
        SemanticType::OuterFloorSurface => "OuterFloorSurface".to_string(),
        SemanticType::Window => "Window".to_string(),
        SemanticType::Door => "Door".to_string(),
        SemanticType::InteriorWallSurface => "InteriorWallSurface".to_string(),
        SemanticType::CeilingSurface => "CeilingSurface".to_string(),
        SemanticType::FloorSurface => "FloorSurface".to_string(),
        SemanticType::WaterSurface => "WaterSurface".to_string(),
        SemanticType::WaterGroundSurface => "WaterGroundSurface".to_string(),
        SemanticType::WaterClosureSurface => "WaterClosureSurface".to_string(),
        SemanticType::TrafficArea => "TrafficArea".to_string(),
        SemanticType::AuxiliaryTrafficArea => "AuxiliaryTrafficArea".to_string(),
        SemanticType::TransportationMarking => "TransportationMarking".to_string(),
        SemanticType::TransportationHole => "TransportationHole".to_string(),
        SemanticType::Extension(value) => value.clone(),
        other => other.to_string(),
    }
}

fn cloned_attributes(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
) -> Option<cityjson::v2_0::OwnedAttributes> {
    attributes
        .cloned()
        .filter(|attributes| !attributes.is_empty())
}

fn non_empty_attributes(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
) -> Option<&cityjson::v2_0::OwnedAttributes> {
    attributes.filter(|attributes| !attributes.is_empty())
}

fn metadata_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    row: MetadataRow,
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let MetadataRow {
        citymodel_id,
        cityjson_version,
        citymodel_kind,
        feature_root_id,
        identifier,
        title,
        reference_system,
        geographical_extent,
        reference_date,
        default_material_theme,
        default_texture_theme,
        point_of_contact,
        root_extra,
        metadata_extra,
    } = row;
    let mut fields = SchemaFieldLookup::new(schema);
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(vec![Some(citymodel_id)])),
        Arc::new(StringArray::from(vec![Some(cityjson_version)])),
        Arc::new(StringArray::from(vec![Some(citymodel_kind)])),
        Arc::new(LargeStringArray::from(vec![feature_root_id])),
        Arc::new(LargeStringArray::from(vec![identifier])),
        Arc::new(LargeStringArray::from(vec![title])),
        Arc::new(LargeStringArray::from(vec![reference_system])),
        Arc::new(fixed_size_f64_array(
            &fields.field("geographical_extent")?,
            6,
            vec![geographical_extent],
        )?),
        Arc::new(StringArray::from(vec![reference_date])),
        Arc::new(StringArray::from(vec![default_material_theme])),
        Arc::new(StringArray::from(vec![default_texture_theme])),
        point_of_contact_array(
            &fields.field("point_of_contact")?,
            point_of_contact.as_ref(),
            projection.metadata_point_of_contact_address.as_ref(),
        )?,
    ];
    if let Some(spec) = projection.root_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("root_extra")?,
            spec,
            &[root_extra.as_ref()],
        )?);
    }
    if let Some(spec) = projection.metadata_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("metadata_extra")?,
            spec,
            &[metadata_extra.as_ref()],
        )?);
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn transform_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    row: TransformRow,
) -> Result<RecordBatch> {
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(fixed_size_f64_array(
                &fields.field("scale")?,
                3,
                vec![Some(row.scale)],
            )?),
            Arc::new(fixed_size_f64_array(
                &fields.field("translate")?,
                3,
                vec![Some(row.translate)],
            )?),
        ],
    )
    .map_err(Error::from)
}

fn geometries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryTableBuffer,
) -> Result<RecordBatch> {
    let GeometryTableBuffer {
        geometry_id,
        cityobject_ix,
        geometry_ordinal,
        geometry_type,
        lod,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt64Array::from(cityobject_ix)),
            Arc::new(UInt32Array::from(geometry_ordinal)),
            Arc::new(StringArray::from(
                geometry_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(lod)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_boundaries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryBoundaryTableBuffer,
) -> Result<RecordBatch> {
    let GeometryBoundaryTableBuffer {
        geometry_id,
        vertex_indices,
        line_offsets,
        ring_offsets,
        surface_offsets,
        shell_offsets,
        solid_offsets,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(vertex_indices.into_array(&fields.field("vertex_indices")?)?),
            Arc::new(line_offsets.into_array(&fields.field("line_offsets")?)?),
            Arc::new(ring_offsets.into_array(&fields.field("ring_offsets")?)?),
            Arc::new(surface_offsets.into_array(&fields.field("surface_offsets")?)?),
            Arc::new(shell_offsets.into_array(&fields.field("shell_offsets")?)?),
            Arc::new(solid_offsets.into_array(&fields.field("solid_offsets")?)?),
        ],
    )
    .map_err(Error::from)
}

fn geometry_instances_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryInstanceTableBuffer,
) -> Result<RecordBatch> {
    let GeometryInstanceTableBuffer {
        geometry_id,
        cityobject_ix,
        geometry_ordinal,
        lod,
        template_geometry_id,
        reference_point_vertex_id,
        transform_matrix,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(geometry_id)),
        Arc::new(UInt64Array::from(cityobject_ix)),
        Arc::new(UInt32Array::from(geometry_ordinal)),
        Arc::new(StringArray::from(lod)),
        Arc::new(UInt64Array::from(template_geometry_id)),
        Arc::new(UInt64Array::from(reference_point_vertex_id)),
        Arc::new(fixed_size_f64_array(
            &fields.field("transform_matrix")?,
            16,
            transform_matrix,
        )?),
    ];
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn template_geometries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryTableBuffer {
        template_geometry_id,
        geometry_type,
        lod,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(StringArray::from(
                geometry_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(lod)),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_boundaries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryBoundaryTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryBoundaryTableBuffer {
        template_geometry_id,
        vertex_indices,
        line_offsets,
        ring_offsets,
        surface_offsets,
        shell_offsets,
        solid_offsets,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(vertex_indices.into_array(&fields.field("vertex_indices")?)?),
            Arc::new(line_offsets.into_array(&fields.field("line_offsets")?)?),
            Arc::new(ring_offsets.into_array(&fields.field("ring_offsets")?)?),
            Arc::new(surface_offsets.into_array(&fields.field("surface_offsets")?)?),
            Arc::new(shell_offsets.into_array(&fields.field("shell_offsets")?)?),
            Arc::new(solid_offsets.into_array(&fields.field("solid_offsets")?)?),
        ],
    )
    .map_err(Error::from)
}

fn geometry_surface_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometrySurfaceSemanticTableBuffer,
) -> Result<RecordBatch> {
    let GeometrySurfaceSemanticTableBuffer {
        geometry_id,
        surface_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_point_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryPointSemanticTableBuffer,
) -> Result<RecordBatch> {
    let GeometryPointSemanticTableBuffer {
        geometry_id,
        point_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(point_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_linestring_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryLinestringSemanticTableBuffer,
) -> Result<RecordBatch> {
    let GeometryLinestringSemanticTableBuffer {
        geometry_id,
        linestring_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(linestring_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometrySemanticTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometrySemanticTableBuffer {
        template_geometry_id,
        primitive_type,
        primitive_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(StringArray::from(
                primitive_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(primitive_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_surface_materials_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometrySurfaceMaterialTableBuffer,
) -> Result<RecordBatch> {
    let GeometrySurfaceMaterialTableBuffer {
        geometry_id,
        surface_ordinal,
        theme,
        material_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(material_id)),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_materials_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryMaterialTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryMaterialTableBuffer {
        template_geometry_id,
        primitive_type,
        primitive_ordinal,
        theme,
        material_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(StringArray::from(
                primitive_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(primitive_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(material_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_ring_textures_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryRingTextureTableBuffer,
) -> Result<RecordBatch> {
    let GeometryRingTextureTableBuffer {
        geometry_id,
        surface_ordinal,
        ring_ordinal,
        theme,
        texture_id,
        uv_indices,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(UInt32Array::from(ring_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(texture_id)),
            Arc::new(uv_indices.into_array(&fields.field("uv_indices")?)?),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_ring_textures_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryRingTextureTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryRingTextureTableBuffer {
        template_geometry_id,
        surface_ordinal,
        ring_ordinal,
        theme,
        texture_id,
        uv_indices,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(UInt32Array::from(ring_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(texture_id)),
            Arc::new(uv_indices.into_array(&fields.field("uv_indices")?)?),
        ],
    )
    .map_err(Error::from)
}
