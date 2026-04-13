#![allow(clippy::wildcard_imports)]

use super::*;

pub(crate) fn decode_parts(parts: &CityModelArrowParts) -> Result<OwnedCityModel> {
    let mut decoder = IncrementalDecoder::new(parts.header.clone(), parts.projection.clone())?;
    let mut state =
        initialize_model_from_metadata(&parts.header, &parts.projection, &parts.metadata)?;
    reserve_parts_import_state(&mut state, parts)?;
    decoder.state = Some(state);
    decoder.seen_tables.insert(CanonicalTable::Metadata);
    decoder.last_table_position = Some(canonical_table_position(CanonicalTable::Metadata));
    for (table, batch) in collect_tables(parts).into_iter().skip(1) {
        decoder.push_batch(table, &batch)?;
    }
    decoder.finish()
}

impl IncrementalDecoder {
    pub(crate) fn new(header: CityArrowHeader, projection: ProjectionLayout) -> Result<Self> {
        validate_appearance_projection_layout(&projection)?;
        Ok(Self {
            header,
            schemas: canonical_schema_set(&projection),
            projection,
            state: None,
            grouped_rows: PartBatchViews::default(),
            last_table_position: None,
            seen_tables: BTreeSet::new(),
        })
    }

    pub(crate) fn push_batch(&mut self, table: CanonicalTable, batch: &RecordBatch) -> Result<()> {
        validate_schema(
            schema_for_table(&self.schemas, table),
            batch.schema(),
            table,
        )?;
        self.validate_table_order(table)?;
        self.dispatch_table(table, batch)?;
        self.seen_tables.insert(table);
        self.last_table_position = Some(canonical_table_position(table));
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<OwnedCityModel> {
        ensure_required_tables_seen(&self.seen_tables)?;
        let mut state = self.state.ok_or_else(|| {
            Error::Unsupported("stream or package is missing metadata".to_string())
        })?;
        attach_cityobject_geometries(&mut state)?;
        apply_feature_root_id(&mut state.model, state.pending_feature_root_id.as_deref())?;
        Ok(state.model)
    }

    fn validate_table_order(&self, table: CanonicalTable) -> Result<()> {
        if self.seen_tables.contains(&table) {
            return Err(Error::Unsupported(format!(
                "duplicate '{}' canonical table batch",
                table.as_str()
            )));
        }
        let position = canonical_table_position(table);
        if let Some(previous) = self.last_table_position
            && position <= previous
        {
            return Err(Error::Unsupported(format!(
                "canonical table '{}' arrived out of order",
                table.as_str()
            )));
        }

        for required in canonical_table_order()
            .iter()
            .take(position)
            .copied()
            .filter(|candidate| candidate.is_required())
        {
            if !self.seen_tables.contains(&required) {
                return Err(Error::Unsupported(format!(
                    "missing required '{}' table before '{}'",
                    required.as_str(),
                    table.as_str()
                )));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn dispatch_table(&mut self, table: CanonicalTable, batch: &RecordBatch) -> Result<()> {
        match table {
            CanonicalTable::Metadata => {
                self.state = Some(initialize_model_from_metadata(
                    &self.header,
                    &self.projection,
                    batch,
                )?);
            }
            CanonicalTable::Transform => import_transform_batch(batch, self.state_mut()?)?,
            CanonicalTable::Extensions => import_extensions_batch(batch, self.state_mut()?)?,
            CanonicalTable::Vertices => import_vertex_batch(batch, self.state_mut()?)?,
            CanonicalTable::TemplateVertices => {
                import_template_vertex_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::TextureVertices => {
                import_texture_vertex_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::Semantics => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                reserve_model_import(
                    state,
                    CityModelCapacities {
                        semantics: batch.num_rows(),
                        ..CityModelCapacities::default()
                    },
                )?;
                let handles = import_semantics_batch(batch, projection, &mut state.model)?;
                state.semantic_handle_by_id = handles;
            }
            CanonicalTable::SemanticChildren => {
                import_semantic_child_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::Materials => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                reserve_model_import(
                    state,
                    CityModelCapacities {
                        materials: batch.num_rows(),
                        ..CityModelCapacities::default()
                    },
                )?;
                let handles = import_materials_batch(batch, projection, &mut state.model)?;
                state.material_handle_by_id = handles;
            }
            CanonicalTable::Textures => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                reserve_model_import(
                    state,
                    CityModelCapacities {
                        textures: batch.num_rows(),
                        ..CityModelCapacities::default()
                    },
                )?;
                let handles = import_textures_batch(batch, projection, &mut state.model)?;
                state.texture_handle_by_id = handles;
            }
            CanonicalTable::TemplateGeometryBoundaries => {
                let view = bind_boundary_batch_view(batch, "template_geometry_id")?;
                let row_by_id = index_unique_ids(
                    &view.id,
                    "template_geometry_id",
                    "template geometry boundary",
                )?;
                self.grouped_rows.template_boundaries = Some(UniqueBatchView { view, row_by_id });
            }
            CanonicalTable::TemplateGeometrySemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
                self.grouped_rows.template_semantics = Some(GroupedBatchView {
                    view: bind_template_semantic_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "template_geometry_id")?,
                });
            }
            CanonicalTable::TemplateGeometryMaterials => {
                let ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
                self.grouped_rows.template_materials = Some(GroupedBatchView {
                    view: bind_template_material_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "template_geometry_id")?,
                });
            }
            CanonicalTable::TemplateGeometryRingTextures => {
                let ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
                self.grouped_rows.template_ring_textures = Some(GroupedBatchView {
                    view: bind_ring_texture_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "template_geometry_id")?,
                });
            }
            CanonicalTable::TemplateGeometries => {
                let grouped_rows = &self.grouped_rows;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                import_template_geometries_batch(batch, state, grouped_rows)?;
            }
            CanonicalTable::GeometryBoundaries => {
                let view = bind_boundary_batch_view(batch, "geometry_id")?;
                let row_by_id = index_unique_ids(&view.id, "geometry_id", "geometry boundary")?;
                self.grouped_rows.boundaries = Some(UniqueBatchView { view, row_by_id });
            }
            CanonicalTable::GeometrySurfaceSemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.surface_semantics = Some(GroupedBatchView {
                    view: bind_indexed_semantic_batch_view(batch, "surface_ordinal")?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryPointSemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.point_semantics = Some(GroupedBatchView {
                    view: bind_indexed_semantic_batch_view(batch, "point_ordinal")?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryLinestringSemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.linestring_semantics = Some(GroupedBatchView {
                    view: bind_indexed_semantic_batch_view(batch, "linestring_ordinal")?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometrySurfaceMaterials => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.surface_materials = Some(GroupedBatchView {
                    view: bind_geometry_surface_material_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryRingTextures => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.ring_textures = Some(GroupedBatchView {
                    view: bind_ring_texture_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryInstances => {
                import_instance_geometries_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::Geometries => {
                let grouped_rows = &self.grouped_rows;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                import_boundary_geometries_batch(batch, state, grouped_rows)?;
            }
            CanonicalTable::CityObjects => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                import_cityobjects_batch(batch, projection, state)?;
            }
            CanonicalTable::CityObjectChildren => {
                import_cityobject_children_batch(batch, self.state_mut()?)?;
            }
        }
        Ok(())
    }

    fn state_mut(&mut self) -> Result<&mut ImportState> {
        self.state.as_mut().ok_or_else(|| {
            Error::Unsupported(
                "metadata table must arrive before other canonical tables".to_string(),
            )
        })
    }
}

fn ensure_required_tables_seen(seen_tables: &BTreeSet<CanonicalTable>) -> Result<()> {
    for table in canonical_table_order()
        .iter()
        .copied()
        .filter(|table| table.is_required())
    {
        if !seen_tables.contains(&table) {
            return Err(Error::Unsupported(format!(
                "stream or package is missing required '{}' table",
                table.as_str()
            )));
        }
    }
    Ok(())
}

fn index_unique_ids(ids: &UInt64Array, id_name: &str, label: &str) -> Result<HashMap<u64, usize>> {
    let mut row_by_id = HashMap::with_capacity(ids.len());
    let mut previous = None;
    for row in 0..ids.len() {
        let id = ids.value(row);
        ensure_strictly_increasing_u64(previous, id, id_name)?;
        previous = Some(id);
        if row_by_id.insert(id, row).is_some() {
            return Err(Error::Conversion(format!("duplicate {label} row {id}")));
        }
    }
    Ok(row_by_id)
}

fn index_grouped_ids(ids: &UInt64Array, id_name: &str) -> Result<HashMap<u64, Range<usize>>> {
    let mut rows_by_id = HashMap::new();
    let mut previous = None;
    let mut range_start = 0_usize;
    for row in 0..ids.len() {
        let id = ids.value(row);
        if let Some(previous_id) = previous {
            if id < previous_id {
                return Err(Error::Conversion(format!(
                    "{id_name} must be non-decreasing in canonical order, found {id} after {previous_id}"
                )));
            }
            if id != previous_id {
                rows_by_id.insert(previous_id, range_start..row);
                range_start = row;
            }
        }
        previous = Some(id);
    }
    if let Some(last_id) = previous {
        rows_by_id.insert(last_id, range_start..ids.len());
    }
    Ok(rows_by_id)
}

pub(super) fn grouped_row_range<V>(
    rows: Option<&GroupedBatchView<V>>,
    id: u64,
) -> Option<&Range<usize>> {
    rows.and_then(|rows| rows.rows_by_id.get(&id))
}

fn bind_u32_list_column(batch: &RecordBatch, name: &str) -> Result<U32ListColumnView> {
    let list = downcast_required::<ListArray>(batch, name)?.clone();
    let values = required_downcast::<UInt32Array>(list.values().as_ref(), "u32")?.clone();
    Ok(U32ListColumnView { list, values })
}

fn bind_u64_list_column(batch: &RecordBatch, name: &str) -> Result<U64ListColumnView> {
    let list = downcast_required::<ListArray>(batch, name)?.clone();
    let values = required_downcast::<UInt64Array>(list.values().as_ref(), "u64")?.clone();
    Ok(U64ListColumnView { list, values })
}

fn bind_boundary_batch_view(batch: &RecordBatch, id_name: &str) -> Result<BoundaryBatchView> {
    Ok(BoundaryBatchView {
        id: downcast_required::<UInt64Array>(batch, id_name)?.clone(),
        vertex_indices: bind_u32_list_column(batch, "vertex_indices")?,
        line_offsets: bind_u32_list_column(batch, "line_offsets")?,
        ring_offsets: bind_u32_list_column(batch, "ring_offsets")?,
        surface_offsets: bind_u32_list_column(batch, "surface_offsets")?,
        shell_offsets: bind_u32_list_column(batch, "shell_offsets")?,
        solid_offsets: bind_u32_list_column(batch, "solid_offsets")?,
    })
}

fn bind_indexed_semantic_batch_view(
    batch: &RecordBatch,
    ordinal_name: &str,
) -> Result<IndexedSemanticBatchView> {
    Ok(IndexedSemanticBatchView {
        semantic_id: downcast_required::<UInt64Array>(batch, "semantic_id")?.clone(),
        ordinal: downcast_required::<UInt32Array>(batch, ordinal_name)?.clone(),
    })
}

fn bind_template_semantic_batch_view(batch: &RecordBatch) -> Result<TemplateSemanticBatchView> {
    Ok(TemplateSemanticBatchView {
        primitive_type: downcast_required::<StringArray>(batch, "primitive_type")?.clone(),
        primitive_ordinal: downcast_required::<UInt32Array>(batch, "primitive_ordinal")?.clone(),
        semantic_id: downcast_required::<UInt64Array>(batch, "semantic_id")?.clone(),
    })
}

fn bind_geometry_surface_material_batch_view(
    batch: &RecordBatch,
) -> Result<GeometrySurfaceMaterialBatchView> {
    Ok(GeometrySurfaceMaterialBatchView {
        theme: downcast_required::<StringArray>(batch, "theme")?.clone(),
        surface_ordinal: downcast_required::<UInt32Array>(batch, "surface_ordinal")?.clone(),
        material_id: downcast_required::<UInt64Array>(batch, "material_id")?.clone(),
    })
}

fn bind_template_material_batch_view(batch: &RecordBatch) -> Result<TemplateMaterialBatchView> {
    Ok(TemplateMaterialBatchView {
        primitive_type: downcast_required::<StringArray>(batch, "primitive_type")?.clone(),
        primitive_ordinal: downcast_required::<UInt32Array>(batch, "primitive_ordinal")?.clone(),
        theme: downcast_required::<StringArray>(batch, "theme")?.clone(),
        material_id: downcast_required::<UInt64Array>(batch, "material_id")?.clone(),
    })
}

fn bind_ring_texture_batch_view(batch: &RecordBatch) -> Result<RingTextureBatchView> {
    Ok(RingTextureBatchView {
        surface_ordinal: downcast_required::<UInt32Array>(batch, "surface_ordinal")?.clone(),
        ring_ordinal: downcast_required::<UInt32Array>(batch, "ring_ordinal")?.clone(),
        theme: downcast_required::<StringArray>(batch, "theme")?.clone(),
        texture_id: downcast_required::<UInt64Array>(batch, "texture_id")?.clone(),
        uv_indices: bind_u64_list_column(batch, "uv_indices")?,
    })
}

fn initialize_model_from_metadata(
    header: &CityArrowHeader,
    projection: &ProjectionLayout,
    metadata: &RecordBatch,
) -> Result<ImportState> {
    let kind = CityModelType::try_from(read_string_scalar(metadata, "citymodel_kind", 0)?)?;
    let mut model = OwnedCityModel::new(kind);
    let empty_geometry_handles = HashMap::new();

    let metadata_row = read_metadata_row(metadata, projection)?;
    if metadata_row.citymodel_id != header.citymodel_id {
        return Err(Error::Conversion(format!(
            "metadata citymodel_id '{}' does not match stream/package header '{}'",
            metadata_row.citymodel_id, header.citymodel_id
        )));
    }
    if metadata_row.cityjson_version != header.cityjson_version {
        return Err(Error::Conversion(format!(
            "metadata cityjson_version '{}' does not match stream/package header '{}'",
            metadata_row.cityjson_version, header.cityjson_version
        )));
    }
    match kind {
        CityModelType::CityJSONFeature if metadata_row.feature_root_id.is_none() => {
            return Err(Error::Conversion(
                "metadata feature_root_id is required for CityJSONFeature".to_string(),
            ));
        }
        CityModelType::CityJSON if metadata_row.feature_root_id.is_some() => {
            return Err(Error::Conversion(
                "metadata feature_root_id is only valid for CityJSONFeature".to_string(),
            ));
        }
        _ => {}
    }
    apply_metadata_row(&mut model, &metadata_row, &empty_geometry_handles)?;

    Ok(ImportState {
        model,
        pending_feature_root_id: metadata_row.feature_root_id.clone(),
        semantic_handle_by_id: HashMap::new(),
        material_handle_by_id: HashMap::new(),
        texture_handle_by_id: HashMap::new(),
        template_handle_by_id: HashMap::new(),
        geometry_handle_by_id: HashMap::new(),
        cityobject_handle_by_ix: Vec::new(),
        pending_geometry_attachments: Vec::new(),
        fully_reserved: false,
    })
}

fn reserve_model_import(state: &mut ImportState, capacities: CityModelCapacities) -> Result<()> {
    if state.fully_reserved {
        return Ok(());
    }
    state.model.reserve_import(capacities).map_err(Error::from)
}

fn reserve_parts_import_state(state: &mut ImportState, parts: &CityModelArrowParts) -> Result<()> {
    let cityobject_count = parts.cityobjects.num_rows();
    let semantics_count = parts.semantics.as_ref().map_or(0, RecordBatch::num_rows);
    let materials_count = parts.materials.as_ref().map_or(0, RecordBatch::num_rows);
    let textures_count = parts.textures.as_ref().map_or(0, RecordBatch::num_rows);
    let template_geometry_count = parts
        .template_geometries
        .as_ref()
        .map_or(0, RecordBatch::num_rows);
    let geometry_count = parts.geometries.num_rows()
        + parts
            .geometry_instances
            .as_ref()
            .map_or(0, RecordBatch::num_rows);

    state
        .model
        .reserve_import(CityModelCapacities {
            cityobjects: cityobject_count,
            vertices: parts.vertices.num_rows(),
            semantics: semantics_count,
            materials: materials_count,
            textures: textures_count,
            geometries: geometry_count,
            template_vertices: parts
                .template_vertices
                .as_ref()
                .map_or(0, RecordBatch::num_rows),
            template_geometries: template_geometry_count,
            uv_coordinates: parts
                .texture_vertices
                .as_ref()
                .map_or(0, RecordBatch::num_rows),
        })
        .map_err(Error::from)?;
    state.semantic_handle_by_id.reserve(semantics_count);
    state.material_handle_by_id.reserve(materials_count);
    state.texture_handle_by_id.reserve(textures_count);
    state.template_handle_by_id.reserve(template_geometry_count);
    state.geometry_handle_by_id.reserve(geometry_count);
    state.cityobject_handle_by_ix.resize(cityobject_count, None);
    state
        .pending_geometry_attachments
        .resize_with(cityobject_count, Vec::new);
    state.fully_reserved = true;
    Ok(())
}

fn ensure_cityobject_slots_for_ix(state: &mut ImportState, max_cityobject_ix: u64) -> Result<()> {
    let slot_len = usize::try_from(max_cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?
        .checked_add(1)
        .ok_or_else(|| Error::Conversion("cityobject slot count overflow".to_string()))?;
    if state.cityobject_handle_by_ix.len() < slot_len {
        state.cityobject_handle_by_ix.resize(slot_len, None);
    }
    if state.pending_geometry_attachments.len() < slot_len {
        state
            .pending_geometry_attachments
            .resize_with(slot_len, Vec::new);
    }
    Ok(())
}

fn import_semantics_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::SemanticHandle>> {
    let empty_geometry_handles = HashMap::new();
    let mut semantic_handle_by_id = HashMap::with_capacity(batch.num_rows());
    let columns = bind_semantic_columns(batch, projection)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let semantic_id = columns.semantic_id.value(row);
        ensure_strictly_increasing_u64(previous_id, semantic_id, "semantic_id")?;
        previous_id = Some(semantic_id);
        let mut semantic =
            OwnedSemantic::new(parse_semantic_type(columns.semantic_type.value(row)));
        if !columns.parent_semantic_id.is_null(row) {
            let parent_semantic_id = columns.parent_semantic_id.value(row);
            let parent = *semantic_handle_by_id
                .get(&parent_semantic_id)
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "missing semantic {parent_semantic_id} for parent relation"
                    ))
                })?;
            semantic.set_parent(parent);
        }
        let projected = projected_attributes_from_array(
            projection.semantic_attributes.as_ref(),
            columns.attributes,
            row,
            &empty_geometry_handles,
        )?;
        if !projected.is_empty() {
            *semantic.attributes_mut() = projected;
        }
        semantic_handle_by_id.insert(semantic_id, model.add_semantic(semantic)?);
    }
    Ok(semantic_handle_by_id)
}

fn import_semantic_child_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    let parents = downcast_required::<UInt64Array>(batch, "parent_semantic_id")?;
    let children = downcast_required::<UInt64Array>(batch, "child_semantic_id")?;
    for row in 0..batch.num_rows() {
        let parent_semantic_id = parents.value(row);
        let child_semantic_id = children.value(row);
        let parent = *state
            .semantic_handle_by_id
            .get(&parent_semantic_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing semantic {parent_semantic_id} for child relation"
                ))
            })?;
        let child = *state
            .semantic_handle_by_id
            .get(&child_semantic_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing semantic {child_semantic_id} for child relation"
                ))
            })?;
        state
            .model
            .get_semantic_mut(parent)
            .ok_or_else(|| Error::Conversion("semantic parent handle missing".to_string()))?
            .children_mut()
            .push(child);
    }
    Ok(())
}

fn import_transform_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    let row = read_transform_row(batch)?;
    state.model.transform_mut().set_scale(row.scale);
    state.model.transform_mut().set_translate(row.translate);
    Ok(())
}

fn import_extensions_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    for row in 0..batch.num_rows() {
        state.model.extensions_mut().add(Extension::new(
            read_string_scalar(batch, "extension_name", row)?,
            read_large_string_scalar(batch, "uri", row)?,
            read_string_optional(batch, "version", row)?.unwrap_or_default(),
        ));
    }
    Ok(())
}

fn import_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            vertices: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_vertex_columns(batch, "vertex_id")?;
    let mut previous_id = None;
    let mut vertices = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let vertex_id = columns.vertex_id.value(row);
        ensure_strictly_increasing_u64(previous_id, vertex_id, "vertex_id")?;
        previous_id = Some(vertex_id);
        vertices.push(cityjson::v2_0::RealWorldCoordinate::new(
            columns.x.value(row),
            columns.y.value(row),
            columns.z.value(row),
        ));
    }
    let _ = state.model.add_vertices(&vertices)?;
    Ok(())
}

fn import_template_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            template_vertices: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_vertex_columns(batch, "template_vertex_id")?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let template_vertex_id = columns.vertex_id.value(row);
        ensure_strictly_increasing_u64(previous_id, template_vertex_id, "template_vertex_id")?;
        previous_id = Some(template_vertex_id);
        state
            .model
            .add_template_vertex(cityjson::v2_0::RealWorldCoordinate::new(
                columns.x.value(row),
                columns.y.value(row),
                columns.z.value(row),
            ))?;
    }
    Ok(())
}

fn import_texture_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            uv_coordinates: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_uv_columns(batch)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let uv_id = columns.uv_id.value(row);
        ensure_strictly_increasing_u64(previous_id, uv_id, "uv_id")?;
        previous_id = Some(uv_id);
        state.model.add_uv_coordinate(UVCoordinate::new(
            columns.u.value(row),
            columns.v.value(row),
        ))?;
    }
    Ok(())
}

fn import_materials_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::MaterialHandle>> {
    let mut material_handle_by_id = HashMap::with_capacity(batch.num_rows());
    let columns = bind_material_columns(batch, projection)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let material_id = columns.material_id.value(row);
        ensure_strictly_increasing_u64(previous_id, material_id, "material_id")?;
        previous_id = Some(material_id);
        let mut material = OwnedMaterial::new(columns.name.value(row).to_string());
        material.set_ambient_intensity(
            read_f64_array_optional(columns.ambient_intensity, row).map(decode_payload_f32),
        );
        material.set_diffuse_color(
            read_list_f64_array_optional::<3>(columns.diffuse_color, row)?.map(rgb_from_components),
        );
        material.set_emissive_color(
            read_list_f64_array_optional::<3>(columns.emissive_color, row)?
                .map(rgb_from_components),
        );
        material.set_specular_color(
            read_list_f64_array_optional::<3>(columns.specular_color, row)?
                .map(rgb_from_components),
        );
        material
            .set_shininess(read_f64_array_optional(columns.shininess, row).map(decode_payload_f32));
        material.set_transparency(
            read_f64_array_optional(columns.transparency, row).map(decode_payload_f32),
        );
        material.set_is_smooth(read_bool_array_optional(columns.is_smooth, row));
        material_handle_by_id.insert(material_id, model.add_material(material)?);
    }
    Ok(material_handle_by_id)
}

fn import_textures_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::TextureHandle>> {
    let mut texture_handle_by_id = HashMap::with_capacity(batch.num_rows());
    let columns = bind_texture_columns(batch, projection)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let texture_id = columns.texture_id.value(row);
        ensure_strictly_increasing_u64(previous_id, texture_id, "texture_id")?;
        previous_id = Some(texture_id);
        let mut texture = OwnedTexture::new(
            columns.image_uri.value(row).to_string(),
            parse_image_type(columns.image_type.value(row))?,
        );
        texture.set_wrap_mode(
            read_large_string_array_optional(columns.wrap_mode, row)
                .as_deref()
                .map(parse_wrap_mode)
                .transpose()?,
        );
        texture.set_texture_type(
            read_large_string_array_optional(columns.texture_type, row)
                .as_deref()
                .map(parse_texture_mapping_type)
                .transpose()?,
        );
        texture.set_border_color(
            read_list_f64_array_optional::<4>(columns.border_color, row)?.map(rgba_from_components),
        );
        texture_handle_by_id.insert(texture_id, model.add_texture(texture)?);
    }
    Ok(texture_handle_by_id)
}

fn import_template_geometries_batch(
    batch: &RecordBatch,
    state: &mut ImportState,
    grouped_rows: &PartBatchViews,
) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            template_geometries: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    state.template_handle_by_id.reserve(batch.num_rows());
    let columns = bind_template_geometry_columns(batch)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let template_geometry_id = columns.template_geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, template_geometry_id, "template_geometry_id")?;
        previous_id = Some(template_geometry_id);
        let boundary_row = grouped_rows
            .template_boundaries
            .as_ref()
            .and_then(|rows| rows.row_by_id.get(&template_geometry_id).copied())
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing boundary row for template geometry {template_geometry_id}"
                ))
            })?;
        let boundary = grouped_rows
            .template_boundaries
            .as_ref()
            .expect("checked above")
            .view
            .payload(boundary_row)?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: parse_geometry_type(columns.geometry_type.value(row))?,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: Some(boundary_from_payload(
                &boundary,
                columns.geometry_type.value(row),
            )?),
            semantics: build_template_semantic_map(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.template_semantics.as_ref(),
                template_geometry_id,
                &state.semantic_handle_by_id,
            )?,
            materials: build_template_material_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.template_materials.as_ref(),
                template_geometry_id,
                &state.material_handle_by_id,
            )?,
            textures: build_template_texture_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.template_ring_textures.as_ref(),
                template_geometry_id,
                &state.texture_handle_by_id,
            )?,
            instance: None,
        });
        state.template_handle_by_id.insert(
            template_geometry_id,
            state.model.add_geometry_template(geometry)?,
        );
    }
    Ok(())
}

fn import_boundary_geometries_batch(
    batch: &RecordBatch,
    state: &mut ImportState,
    grouped_rows: &PartBatchViews,
) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            geometries: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    state.geometry_handle_by_id.reserve(batch.num_rows());
    let columns = bind_geometry_columns(batch)?;
    if batch.num_rows() > 0 {
        ensure_cityobject_slots_for_ix(state, columns.cityobject_ix.value(batch.num_rows() - 1))?;
    }
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let geometry_id = columns.geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, geometry_id, "geometry_id")?;
        previous_id = Some(geometry_id);
        let boundary_row = grouped_rows
            .boundaries
            .as_ref()
            .and_then(|rows| rows.row_by_id.get(&geometry_id).copied())
            .ok_or_else(|| {
                Error::Conversion(format!("missing boundary row for geometry {geometry_id}"))
            })?;
        let boundary = grouped_rows
            .boundaries
            .as_ref()
            .expect("checked above")
            .view
            .payload(boundary_row)?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: parse_geometry_type(columns.geometry_type.value(row))?,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: Some(boundary_from_payload(
                &boundary,
                columns.geometry_type.value(row),
            )?),
            semantics: build_semantic_map(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.surface_semantics.as_ref(),
                grouped_rows.point_semantics.as_ref(),
                grouped_rows.linestring_semantics.as_ref(),
                geometry_id,
                &state.semantic_handle_by_id,
            )?,
            materials: build_material_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.surface_materials.as_ref(),
                geometry_id,
                &state.material_handle_by_id,
            )?,
            textures: build_texture_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.ring_textures.as_ref(),
                geometry_id,
                &state.texture_handle_by_id,
            )?,
            instance: None,
        });
        insert_unique_geometry_handle(
            &mut state.geometry_handle_by_id,
            geometry_id,
            state.model.add_geometry(geometry)?,
        )?;
        push_pending_geometry_attachment(
            state,
            columns.cityobject_ix.value(row),
            columns.geometry_ordinal.value(row),
            geometry_id,
        )?;
    }
    Ok(())
}

fn import_instance_geometries_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            geometries: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    state.geometry_handle_by_id.reserve(batch.num_rows());
    let columns = bind_geometry_instance_columns(batch)?;
    if batch.num_rows() > 0 {
        ensure_cityobject_slots_for_ix(state, columns.cityobject_ix.value(batch.num_rows() - 1))?;
    }
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let geometry_id = columns.geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, geometry_id, "geometry_instance_id")?;
        previous_id = Some(geometry_id);
        let template = *state
            .template_handle_by_id
            .get(&columns.template_geometry_id.value(row))
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing template geometry {}",
                    columns.template_geometry_id.value(row)
                ))
            })?;
        let reference_point =
            u32::try_from(columns.reference_point_vertex_id.value(row)).map_err(|_| {
                Error::Conversion(format!(
                    "reference point vertex id {} does not fit into u32",
                    columns.reference_point_vertex_id.value(row)
                ))
            })?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: GeometryType::GeometryInstance,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: None,
            semantics: None,
            materials: None,
            textures: None,
            instance: Some(StoredGeometryInstance {
                template,
                reference_point: cityjson::v2_0::VertexIndex::new(reference_point),
                transformation: read_fixed_size_list_array_optional::<16>(
                    columns.transform_matrix,
                    "transform_matrix",
                    row,
                )?
                .map(cityjson::v2_0::AffineTransform3D::from)
                .unwrap_or_default(),
            }),
        });
        insert_unique_geometry_handle(
            &mut state.geometry_handle_by_id,
            geometry_id,
            state.model.add_geometry(geometry)?,
        )?;
        push_pending_geometry_attachment(
            state,
            columns.cityobject_ix.value(row),
            columns.geometry_ordinal.value(row),
            geometry_id,
        )?;
    }
    Ok(())
}

fn insert_unique_geometry_handle(
    handles: &mut HashMap<u64, cityjson::prelude::GeometryHandle>,
    geometry_id: u64,
    handle: cityjson::prelude::GeometryHandle,
) -> Result<()> {
    if handles.insert(geometry_id, handle).is_some() {
        return Err(Error::Conversion(format!(
            "duplicate geometry id {geometry_id}"
        )));
    }
    Ok(())
}

fn ensure_slot<T: Default>(slots: &mut Vec<T>, index: usize) {
    if slots.len() <= index {
        slots.resize_with(index + 1, T::default);
    }
}

fn push_pending_geometry_attachment(
    state: &mut ImportState,
    cityobject_ix: u64,
    geometry_ordinal: u32,
    geometry_id: u64,
) -> Result<()> {
    let cityobject_ix = usize::try_from(cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?;
    ensure_slot(&mut state.pending_geometry_attachments, cityobject_ix);
    let attachments = &mut state.pending_geometry_attachments[cityobject_ix];
    if let Some((last_ordinal, last_geometry_id)) = attachments.last()
        && (geometry_ordinal < *last_ordinal
            || (geometry_ordinal == *last_ordinal && geometry_id <= *last_geometry_id))
    {
        return Err(Error::Conversion(format!(
            "geometry attachment order for cityobject_ix {cityobject_ix} is not strictly increasing"
        )));
    }
    attachments.push((geometry_ordinal, geometry_id));
    Ok(())
}

fn register_cityobject_handle(
    state: &mut ImportState,
    cityobject_ix: u64,
    handle: cityjson::prelude::CityObjectHandle,
) -> Result<()> {
    let cityobject_ix = usize::try_from(cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?;
    ensure_slot(&mut state.cityobject_handle_by_ix, cityobject_ix);
    let slot = &mut state.cityobject_handle_by_ix[cityobject_ix];
    if slot.replace(handle).is_some() {
        return Err(Error::Conversion(format!(
            "duplicate cityobject_ix {cityobject_ix}"
        )));
    }
    Ok(())
}

fn cityobject_handle(
    state: &ImportState,
    cityobject_ix: u64,
) -> Result<cityjson::prelude::CityObjectHandle> {
    let cityobject_ix = usize::try_from(cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?;
    state
        .cityobject_handle_by_ix
        .get(cityobject_ix)
        .and_then(|handle| *handle)
        .ok_or_else(|| Error::Conversion(format!("missing cityobject_ix {cityobject_ix}")))
}

fn import_cityobjects_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    state: &mut ImportState,
) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            cityobjects: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_cityobject_columns(batch, projection)?;
    if batch.num_rows() > 0 {
        ensure_cityobject_slots_for_ix(state, columns.cityobject_ix.value(batch.num_rows() - 1))?;
    }
    let mut previous_ix = None;
    for row in 0..batch.num_rows() {
        let object_index = columns.cityobject_ix.value(row);
        ensure_strictly_increasing_u64(previous_ix, object_index, "cityobject_ix")?;
        previous_ix = Some(object_index);
        let object_id = columns.cityobject_id.value(row).to_string();
        let mut object = CityObject::new(
            CityObjectIdentifier::new(object_id.clone()),
            columns
                .object_type
                .value(row)
                .parse::<CityObjectType<_>>()?,
        );
        if let Some(extent) = read_fixed_size_list_array_optional::<6>(
            columns.geographical_extent,
            "geographical_extent",
            row,
        )? {
            object.set_geographical_extent(Some(BBox::from(extent)));
        }
        let projected_attributes = projected_attributes_from_array(
            projection.cityobject_attributes.as_ref(),
            columns.attributes,
            row,
            &state.geometry_handle_by_id,
        )?;
        if !projected_attributes.is_empty() {
            *object.attributes_mut() = projected_attributes;
        }
        let projected_extra = projected_attributes_from_array(
            projection.cityobject_extra.as_ref(),
            columns.extra,
            row,
            &state.geometry_handle_by_id,
        )?;
        if !projected_extra.is_empty() {
            *object.extra_mut() = projected_extra;
        }
        let handle = state.model.cityobjects_mut().add(object)?;
        register_cityobject_handle(state, object_index, handle)?;
    }
    Ok(())
}

fn attach_cityobject_geometries(state: &mut ImportState) -> Result<()> {
    for (cityobject_ix, attachments) in state.pending_geometry_attachments.iter_mut().enumerate() {
        if attachments.is_empty() {
            continue;
        }
        let object = state
            .cityobject_handle_by_ix
            .get(cityobject_ix)
            .and_then(|handle| *handle)
            .ok_or_else(|| Error::Conversion(format!("missing cityobject_ix {cityobject_ix}")))?;
        let object = state
            .model
            .cityobjects_mut()
            .get_mut(object)
            .ok_or_else(|| Error::Conversion("missing cityobject handle".to_string()))?;
        for (_, geometry_id) in attachments.iter() {
            let geometry = state
                .geometry_handle_by_id
                .get(geometry_id)
                .copied()
                .ok_or_else(|| Error::Conversion(format!("missing geometry {geometry_id}")))?;
            object.add_geometry(geometry);
        }
    }
    Ok(())
}

fn apply_feature_root_id(model: &mut OwnedCityModel, feature_root_id: Option<&str>) -> Result<()> {
    let Some(feature_root_id) = feature_root_id else {
        return Ok(());
    };
    let handle = model
        .cityobjects()
        .iter()
        .find_map(|(handle, cityobject)| (cityobject.id() == feature_root_id).then_some(handle))
        .ok_or_else(|| {
            Error::Conversion(format!(
                "feature_root_id does not resolve to a CityObject: {feature_root_id}"
            ))
        })?;
    model.set_id(Some(handle));
    Ok(())
}

fn import_cityobject_children_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    let parents = downcast_required::<UInt64Array>(batch, "parent_cityobject_ix")?;
    let children = downcast_required::<UInt64Array>(batch, "child_cityobject_ix")?;
    for row in 0..batch.num_rows() {
        let parent = cityobject_handle(state, parents.value(row))?;
        let child = cityobject_handle(state, children.value(row))?;
        state
            .model
            .cityobjects_mut()
            .get_mut(parent)
            .ok_or_else(|| Error::Conversion("missing parent handle".to_string()))?
            .add_child(child);
        state
            .model
            .cityobjects_mut()
            .get_mut(child)
            .ok_or_else(|| Error::Conversion("missing child handle".to_string()))?
            .add_parent(parent);
    }
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
pub(super) fn decode_payload_f32(value: f64) -> f32 {
    value as f32
}
