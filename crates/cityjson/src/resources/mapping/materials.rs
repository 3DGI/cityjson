//! # Material Mapping
//!
//! This module provides types for mapping materials to CityJSON geometry elements.
//! It defines a specialized map type that associates material resources with specific
//! geometry parts such as surfaces, linestrings, or points.
//!
//! ## Overview
//!
//! The material mapping module contains:
//!
//! - [`MaterialMap`]: A type for mapping materials to geometric elements
//!
//! The `MaterialMap` is a type alias for the more generic [SemanticOrMaterialMap], specialized
//! for mapping material resources.
//!
//! ## Usage Examples
//!
//! See [SemanticOrMaterialMap] for usage examples.
//!
//! ## Implementation Details
//!
//! The `MaterialMap` uses the same structure as `SemanticMap`, allowing materials to be
//! associated with geometry elements at different hierarchical levels. This design provides
//! a consistent interface for both semantic and material mappings.

use crate::resources::mapping::SemanticOrMaterialMap;

/// A mapping between geometry elements and material resources.
///
/// This type associates material resources with specific geometry elements such as
/// surfaces, linestrings, or points. It is a specialized version of the more generic
/// [SemanticOrMaterialMap].
///
/// # Type Parameters
///
/// * `VR` - The vertex reference type (e.g., u16, u32, u64) that determines indexing sizes
/// * `RR` - The resource reference type used to identify materials
///
pub type MaterialMap<VR, RR> = SemanticOrMaterialMap<VR, RR>;
