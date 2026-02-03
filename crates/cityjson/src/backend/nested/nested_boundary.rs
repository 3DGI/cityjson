//! Boundary types for the nested backend.
//!

use crate::Error;
use crate::prelude::{GeometryType, VertexIndex32};

#[derive(Clone, Debug, PartialEq)]
pub enum Boundary {
    MultiPoint(BoundaryMultiPoint),
    MultiLineString(BoundaryMultiLineString),
    MultiSurface(BoundaryMultiOrCompositeSurface),
    CompositeSurface(BoundaryMultiOrCompositeSurface),
    Solid(BoundarySolid),
    MultiSolid(BoundaryMultiOrCompositeSolid),
    CompositeSolid(BoundaryMultiOrCompositeSolid),
}

impl Boundary {
    /// Returns the geometry type that matches this boundary structure.
    pub fn check_type(&self) -> GeometryType {
        match self {
            Boundary::MultiPoint(_) => GeometryType::MultiPoint,
            Boundary::MultiLineString(_) => GeometryType::MultiLineString,
            Boundary::MultiSurface(_) => GeometryType::MultiSurface,
            Boundary::CompositeSurface(_) => GeometryType::CompositeSurface,
            Boundary::Solid(_) => GeometryType::Solid,
            Boundary::MultiSolid(_) => GeometryType::MultiSolid,
            Boundary::CompositeSolid(_) => GeometryType::CompositeSolid,
        }
    }

    /// Validates the boundary structure.
    /// Returns Ok(()) if valid, or an error describing the validation failure.
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Boundary::MultiPoint(points) => {
                if points.is_empty() {
                    return Err(Error::InvalidGeometry(
                        "MultiPoint boundary cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            Boundary::MultiLineString(linestrings) => {
                if linestrings.is_empty() {
                    return Err(Error::InvalidGeometry(
                        "MultiLineString boundary cannot be empty".to_string(),
                    ));
                }
                for (i, linestring) in linestrings.iter().enumerate() {
                    if linestring.len() < 2 {
                        return Err(Error::InvalidGeometry(format!(
                            "LineString {} has {} vertices, minimum is 2",
                            i,
                            linestring.len()
                        )));
                    }
                }
                Ok(())
            }
            Boundary::MultiSurface(surfaces) | Boundary::CompositeSurface(surfaces) => {
                if surfaces.is_empty() {
                    return Err(Error::InvalidGeometry(
                        "Surface boundary cannot be empty".to_string(),
                    ));
                }
                for (i, surface) in surfaces.iter().enumerate() {
                    if surface.is_empty() {
                        return Err(Error::InvalidGeometry(format!(
                            "Surface {} has no rings",
                            i
                        )));
                    }
                    for (j, ring) in surface.iter().enumerate() {
                        if ring.len() < 3 {
                            return Err(Error::InvalidGeometry(format!(
                                "Surface {} ring {} has {} vertices, minimum is 3",
                                i,
                                j,
                                ring.len()
                            )));
                        }
                    }
                }
                Ok(())
            }
            Boundary::Solid(shells) => {
                if shells.is_empty() {
                    return Err(Error::InvalidGeometry(
                        "Solid boundary cannot be empty".to_string(),
                    ));
                }
                for (i, shell) in shells.iter().enumerate() {
                    if shell.is_empty() {
                        return Err(Error::InvalidGeometry(format!(
                            "Solid shell {} has no surfaces",
                            i
                        )));
                    }
                    for (j, surface) in shell.iter().enumerate() {
                        if surface.is_empty() {
                            return Err(Error::InvalidGeometry(format!(
                                "Solid shell {} surface {} has no rings",
                                i, j
                            )));
                        }
                        for (k, ring) in surface.iter().enumerate() {
                            if ring.len() < 3 {
                                return Err(Error::InvalidGeometry(format!(
                                    "Solid shell {} surface {} ring {} has {} vertices, minimum is 3",
                                    i,
                                    j,
                                    k,
                                    ring.len()
                                )));
                            }
                        }
                    }
                }
                Ok(())
            }
            Boundary::MultiSolid(solids) | Boundary::CompositeSolid(solids) => {
                if solids.is_empty() {
                    return Err(Error::InvalidGeometry(
                        "MultiSolid boundary cannot be empty".to_string(),
                    ));
                }
                for (i, solid) in solids.iter().enumerate() {
                    if solid.is_empty() {
                        return Err(Error::InvalidGeometry(format!(
                            "MultiSolid solid {} has no shells",
                            i
                        )));
                    }
                    for (j, shell) in solid.iter().enumerate() {
                        if shell.is_empty() {
                            return Err(Error::InvalidGeometry(format!(
                                "MultiSolid solid {} shell {} has no surfaces",
                                i, j
                            )));
                        }
                        for (k, surface) in shell.iter().enumerate() {
                            if surface.is_empty() {
                                return Err(Error::InvalidGeometry(format!(
                                    "MultiSolid solid {} shell {} surface {} has no rings",
                                    i, j, k
                                )));
                            }
                            for (l, ring) in surface.iter().enumerate() {
                                if ring.len() < 3 {
                                    return Err(Error::InvalidGeometry(format!(
                                        "MultiSolid solid {} shell {} surface {} ring {} has {} vertices, minimum is 3",
                                        i,
                                        j,
                                        k,
                                        l,
                                        ring.len()
                                    )));
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

pub type BoundaryMultiPoint = Vec<VertexIndex32>;
pub type BoundaryMultiLineString = Vec<BoundaryMultiPoint>;
pub type BoundaryMultiOrCompositeSurface = Vec<BoundaryMultiLineString>;

pub type BoundarySolid = Vec<BoundaryMultiOrCompositeSurface>;

pub type BoundaryMultiOrCompositeSolid = Vec<BoundarySolid>;
