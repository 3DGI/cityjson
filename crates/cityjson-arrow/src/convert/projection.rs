#![allow(clippy::wildcard_imports)]

use super::*;

pub(super) fn discover_projection_layout(
    relational: &ModelRelationalView<'_>,
) -> Result<ProjectionLayout> {
    let model = relational.model();
    Ok(ProjectionLayout {
        root_extra: discover_optional_attribute_projection(model.extra())?,
        metadata_extra: discover_optional_attribute_projection(
            model.metadata().and_then(Metadata::extra),
        )?,
        metadata_point_of_contact_address: discover_optional_attribute_projection(
            model
                .metadata()
                .and_then(Metadata::point_of_contact)
                .and_then(Contact::address),
        )?,
        cityobject_attributes: discover_attribute_projection(
            model
                .cityobjects()
                .iter()
                .filter_map(|(_, object)| object.attributes()),
        )?,
        cityobject_extra: discover_attribute_projection(
            model
                .cityobjects()
                .iter()
                .filter_map(|(_, object)| object.extra()),
        )?,
        geometry_extra: None,
        semantic_attributes: discover_attribute_projection(
            model
                .iter_semantics()
                .filter_map(|(_, semantic)| semantic.attributes()),
        )?,
        material_payload: (model.material_count() > 0).then(canonical_material_projection),
        texture_payload: (model.texture_count() > 0).then(canonical_texture_projection),
    })
}

pub(super) fn canonical_material_projection() -> ProjectedStructSpec {
    ProjectedStructSpec::new(vec![
        ProjectedFieldSpec::new(FIELD_MATERIAL_NAME, ProjectedValueSpec::Utf8, false),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_AMBIENT_INTENSITY,
            ProjectedValueSpec::Float64,
            true,
        ),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_DIFFUSE_COLOR,
            ProjectedValueSpec::List {
                item_nullable: false,
                item: Box::new(ProjectedValueSpec::Float64),
            },
            true,
        ),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_EMISSIVE_COLOR,
            ProjectedValueSpec::List {
                item_nullable: false,
                item: Box::new(ProjectedValueSpec::Float64),
            },
            true,
        ),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_SPECULAR_COLOR,
            ProjectedValueSpec::List {
                item_nullable: false,
                item: Box::new(ProjectedValueSpec::Float64),
            },
            true,
        ),
        ProjectedFieldSpec::new(FIELD_MATERIAL_SHININESS, ProjectedValueSpec::Float64, true),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_TRANSPARENCY,
            ProjectedValueSpec::Float64,
            true,
        ),
        ProjectedFieldSpec::new(FIELD_MATERIAL_IS_SMOOTH, ProjectedValueSpec::Boolean, true),
    ])
}

pub(super) fn canonical_texture_projection() -> ProjectedStructSpec {
    ProjectedStructSpec::new(vec![
        ProjectedFieldSpec::new(FIELD_TEXTURE_IMAGE_TYPE, ProjectedValueSpec::Utf8, false),
        ProjectedFieldSpec::new(FIELD_TEXTURE_WRAP_MODE, ProjectedValueSpec::Utf8, true),
        ProjectedFieldSpec::new(FIELD_TEXTURE_TEXTURE_TYPE, ProjectedValueSpec::Utf8, true),
        ProjectedFieldSpec::new(
            FIELD_TEXTURE_BORDER_COLOR,
            ProjectedValueSpec::List {
                item_nullable: false,
                item: Box::new(ProjectedValueSpec::Float64),
            },
            true,
        ),
    ])
}

pub(super) fn validate_appearance_projection_layout(layout: &ProjectionLayout) -> Result<()> {
    let supported_material = canonical_material_projection()
        .fields
        .into_iter()
        .map(|spec| spec.name)
        .collect::<BTreeSet<_>>();
    if let Some(specs) = &layout.material_payload {
        for spec in &specs.fields {
            if !supported_material.contains(&spec.name) {
                return Err(Error::Unsupported(format!(
                    "material payload column {}",
                    spec.name
                )));
            }
        }
    }

    let supported_texture = canonical_texture_projection()
        .fields
        .into_iter()
        .map(|spec| spec.name)
        .collect::<BTreeSet<_>>();
    if let Some(specs) = &layout.texture_payload {
        for spec in &specs.fields {
            if !supported_texture.contains(&spec.name) {
                return Err(Error::Unsupported(format!(
                    "texture payload column {}",
                    spec.name
                )));
            }
        }
    }
    Ok(())
}

pub(super) fn discover_optional_attribute_projection(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
) -> Result<Option<ProjectedStructSpec>> {
    match attributes {
        Some(attributes) => discover_attribute_projection(std::iter::once(attributes)),
        None => Ok(None),
    }
}

pub(super) fn discover_attribute_projection<'a, I>(
    attributes: I,
) -> Result<Option<ProjectedStructSpec>>
where
    I: IntoIterator<Item = &'a cityjson::v2_0::OwnedAttributes>,
{
    let mut layout = ProjectedStructSpec::new(Vec::new());
    let mut seen_rows = 0_usize;

    for attrs in attributes {
        merge_attribute_map_into_spec(&mut layout, attrs, seen_rows)?;
        seen_rows += 1;
    }

    if seen_rows == 0 || layout.is_empty() {
        Ok(None)
    } else {
        sort_projected_struct(&mut layout);
        Ok(Some(layout))
    }
}

pub(super) fn merge_attribute_map_into_spec(
    spec: &mut ProjectedStructSpec,
    attributes: &cityjson::v2_0::OwnedAttributes,
    seen_rows: usize,
) -> Result<()> {
    let present = attributes.keys().cloned().collect::<BTreeSet<_>>();
    for field in &mut spec.fields {
        if !present.contains(&field.name) {
            field.nullable = true;
        }
    }

    for (key, value) in attributes.iter() {
        if let Some(field) = spec.fields.iter_mut().find(|field| field.name == *key) {
            merge_projected_field(field, value)?;
        } else {
            spec.fields.push(ProjectedFieldSpec::new(
                key.clone(),
                infer_projected_value_spec(value)?,
                seen_rows > 0 || matches!(value, AttributeValue::Null),
            ));
        }
    }

    sort_projected_struct(spec);
    Ok(())
}

pub(super) fn merge_projected_field(
    field: &mut ProjectedFieldSpec,
    value: &OwnedAttributeValue,
) -> Result<()> {
    if matches!(value, AttributeValue::Null) {
        field.nullable = true;
        return Ok(());
    }

    let inferred = infer_projected_value_spec(value)?;
    field.value = merge_projected_value_specs(field.value.clone(), inferred)?;
    Ok(())
}

pub(super) fn infer_projected_value_spec(
    value: &OwnedAttributeValue,
) -> Result<ProjectedValueSpec> {
    Ok(match value {
        AttributeValue::Null => ProjectedValueSpec::Null,
        AttributeValue::Bool(_) => ProjectedValueSpec::Boolean,
        AttributeValue::Unsigned(_) => ProjectedValueSpec::UInt64,
        AttributeValue::Integer(_) => ProjectedValueSpec::Int64,
        AttributeValue::Float(_) => ProjectedValueSpec::Float64,
        AttributeValue::String(_) => ProjectedValueSpec::Utf8,
        AttributeValue::Geometry(_) => ProjectedValueSpec::GeometryRef,
        AttributeValue::Vec(values) => {
            let mut item_nullable = false;
            let mut item_spec = ProjectedValueSpec::Null;
            let mut has_non_null = false;
            for item in values {
                if matches!(item, AttributeValue::Null) {
                    item_nullable = true;
                    continue;
                }
                let inferred = infer_projected_value_spec(item)?;
                item_spec = if has_non_null {
                    merge_projected_value_specs(item_spec, inferred)?
                } else {
                    inferred
                };
                has_non_null = true;
            }
            ProjectedValueSpec::List {
                item_nullable,
                item: Box::new(item_spec),
            }
        }
        AttributeValue::Map(values) => {
            let mut fields = ProjectedStructSpec::new(Vec::new());
            let mut attributes = cityjson::v2_0::OwnedAttributes::default();
            for (key, value) in values {
                attributes.insert(key.clone(), value.clone());
            }
            merge_attribute_map_into_spec(&mut fields, &attributes, 0)?;
            ProjectedValueSpec::Struct(fields)
        }
        other => {
            return Err(Error::Unsupported(format!(
                "unsupported attribute value variant {other}"
            )));
        }
    })
}

pub(super) fn merge_projected_value_specs(
    current: ProjectedValueSpec,
    incoming: ProjectedValueSpec,
) -> Result<ProjectedValueSpec> {
    Ok(match (current, incoming) {
        (ProjectedValueSpec::Null, other) | (other, ProjectedValueSpec::Null) => other,
        (ProjectedValueSpec::Boolean, ProjectedValueSpec::Boolean) => ProjectedValueSpec::Boolean,
        (ProjectedValueSpec::UInt64, ProjectedValueSpec::UInt64) => ProjectedValueSpec::UInt64,
        (ProjectedValueSpec::Int64 | ProjectedValueSpec::UInt64, ProjectedValueSpec::Int64)
        | (ProjectedValueSpec::Int64, ProjectedValueSpec::UInt64) => ProjectedValueSpec::Int64,
        (
            ProjectedValueSpec::Float64 | ProjectedValueSpec::UInt64 | ProjectedValueSpec::Int64,
            ProjectedValueSpec::Float64,
        )
        | (ProjectedValueSpec::Float64, ProjectedValueSpec::UInt64 | ProjectedValueSpec::Int64) => {
            ProjectedValueSpec::Float64
        }
        (ProjectedValueSpec::Utf8, ProjectedValueSpec::Utf8) => ProjectedValueSpec::Utf8,
        (ProjectedValueSpec::GeometryRef, ProjectedValueSpec::GeometryRef) => {
            ProjectedValueSpec::GeometryRef
        }
        (
            ProjectedValueSpec::List {
                item_nullable: left_nullable,
                item: left_item,
            },
            ProjectedValueSpec::List {
                item_nullable: right_nullable,
                item: right_item,
            },
        ) => ProjectedValueSpec::List {
            item_nullable: left_nullable || right_nullable,
            item: Box::new(merge_projected_value_specs(*left_item, *right_item)?),
        },
        (ProjectedValueSpec::Struct(left), ProjectedValueSpec::Struct(right)) => {
            ProjectedValueSpec::Struct(merge_projected_struct_specs(left, right)?)
        }
        // Incompatible types: fall back to JSON-string encoding so no data is lost.
        _ => ProjectedValueSpec::Json,
    })
}

pub(super) fn merge_projected_struct_specs(
    mut left: ProjectedStructSpec,
    right: ProjectedStructSpec,
) -> Result<ProjectedStructSpec> {
    let right_names = right
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<BTreeSet<_>>();
    for field in &mut left.fields {
        if !right_names.contains(&field.name) {
            field.nullable = true;
        }
    }

    for incoming in right.fields {
        if let Some(existing) = left
            .fields
            .iter_mut()
            .find(|field| field.name == incoming.name)
        {
            existing.nullable |= incoming.nullable;
            existing.value = merge_projected_value_specs(existing.value.clone(), incoming.value)?;
        } else {
            let mut incoming = incoming;
            incoming.nullable = true;
            left.fields.push(incoming);
        }
    }

    sort_projected_struct(&mut left);
    Ok(left)
}

pub(super) fn sort_projected_struct(spec: &mut ProjectedStructSpec) {
    spec.fields
        .sort_by(|left, right| left.name.cmp(&right.name));
    for field in &mut spec.fields {
        if let ProjectedValueSpec::Struct(child) = &mut field.value {
            sort_projected_struct(child);
        } else if let ProjectedValueSpec::List { item, .. } = &mut field.value
            && let ProjectedValueSpec::Struct(child) = item.as_mut()
        {
            sort_projected_struct(child);
        }
    }
}
