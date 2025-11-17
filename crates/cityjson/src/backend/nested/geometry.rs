//! Geometry types for the nested backend.
//!
//! TODO: Implement nested backend geometry types.

use crate::backend::nested::appearance::{MaterialValues, TextureValues};
use crate::backend::nested::boundary::Boundary;
use crate::backend::nested::semantics::Semantics;
use crate::prelude::{GeometryType, LoD, RealWorldCoordinate, StringStorage};
use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub struct Geometry<SS: StringStorage> {
    _phantom: PhantomData<SS>,
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary>,
    semantics: Option<Semantics<SS>>,
    materials: Option<HashMap<String, MaterialValues>>,
    textures: Option<HashMap<String, TextureValues>>,
    instance_template: Option<usize>,
    instance_reference_point: Option<RealWorldCoordinate>,
    instance_transformation_matrix: Option<[f64; 16]>,
}

impl<SS> Display for Geometry<SS>
where
    SS: StringStorage,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Default, Debug)]
pub struct GeometryTemplates<SS: StringStorage> {
    pub templates: Vec<Geometry<SS>>,
    pub vertices_templates: VerticesTemplates,
}

pub type VerticesTemplates = Vec<[f64; 3]>;
