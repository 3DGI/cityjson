//! Geometry types for the nested backend.
//!

use crate::backend::nested::appearance::{MaterialValues, TextureValues};
use crate::backend::nested::boundary::Boundary;
use crate::backend::nested::semantics::Semantics;
use crate::prelude::{GeometryType, LoD, RealWorldCoordinate, ResourceRef, StringStorage};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct Geometry<SS: StringStorage, RR: ResourceRef> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary>,
    semantics: Option<Semantics<SS, RR>>,
    materials: Option<HashMap<String, MaterialValues>>,
    textures: Option<HashMap<String, TextureValues>>,
    instance_template: Option<usize>,
    instance_reference_point: Option<RealWorldCoordinate>,
    instance_transformation_matrix: Option<[f64; 16]>,
}

impl<SS: StringStorage, RR: ResourceRef> Geometry<SS, RR> {
    // Constructor
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary>,
        semantics: Option<Semantics<SS, RR>>,
        materials: Option<HashMap<String, MaterialValues>>,
        textures: Option<HashMap<String, TextureValues>>,
        instance_template: Option<usize>,
        instance_reference_point: Option<RealWorldCoordinate>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self {
            type_geometry,
            lod,
            boundaries,
            semantics,
            materials,
            textures,
            instance_template,
            instance_reference_point,
            instance_transformation_matrix,
        }
    }

    // Getters
    pub fn type_geometry(&self) -> &GeometryType {
        &self.type_geometry
    }

    pub fn lod(&self) -> Option<&LoD> {
        self.lod.as_ref()
    }

    pub fn boundaries(&self) -> Option<&Boundary> {
        self.boundaries.as_ref()
    }

    pub fn semantics(&self) -> Option<&Semantics<SS, RR>> {
        self.semantics.as_ref()
    }

    pub fn materials(&self) -> Option<&HashMap<String, MaterialValues>> {
        self.materials.as_ref()
    }

    pub fn textures(&self) -> Option<&HashMap<String, TextureValues>> {
        self.textures.as_ref()
    }

    pub fn instance_template(&self) -> Option<usize> {
        self.instance_template
    }

    pub fn instance_reference_point(&self) -> Option<&RealWorldCoordinate> {
        self.instance_reference_point.as_ref()
    }

    pub fn instance_transformation_matrix(&self) -> Option<&[f64; 16]> {
        self.instance_transformation_matrix.as_ref()
    }
}

impl<SS: StringStorage, RR: ResourceRef> Display for Geometry<SS, RR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Default, Debug)]
pub struct GeometryTemplates<SS: StringStorage, RR: ResourceRef> {
    pub templates: Vec<Geometry<SS, RR>>,
    pub vertices_templates: VerticesTemplates,
}

pub type VerticesTemplates = Vec<[f64; 3]>;
