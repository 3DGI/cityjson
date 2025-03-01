//! # Semantic Mapping
//!
//! This module provides types for mapping semantic information to CityJSON geometry elements.
//! It defines a specialized map type that associates semantic resources with specific
//! geometry parts such as surfaces, linestrings, or points.
//!
//! ## Overview
//!
//! The semantic mapping module contains:
//!
//! - [`SemanticMap`]: A type for mapping semantics to geometric elements
//!
//! The `SemanticMap` is a type alias for the more generic [SemanticOrMaterialMap], specialized
//! for mapping semantic resources.
//!
//! ## Usage Examples
//!
//! ### Creating a semantic mapping for surfaces
//!
//! See [SemanticOrMaterialMap] for usage examples.
//!
//! ## Implementation Details
//!
//! The `SemanticMap` uses the same structure as `MaterialMap`, allowing semantic information to be
//! associated with geometry elements at different hierarchical levels. This design provides
//! a consistent interface for both semantic and material mappings.

use crate::resources::mapping::SemanticOrMaterialMap;

/// A mapping between geometry elements and semantic resources.
///
/// This type associates semantic resources with specific geometry elements such as
/// surfaces, linestrings, or points. It is a specialized version of the more generic
/// `SemanticOrMaterialMap`.
///
/// # Type Parameters
///
/// * `VR` - The vertex reference type (e.g., u16, u32, u64) that determines indexing sizes
/// * `RR` - The resource reference type used to identify semantics
///
pub type SemanticMap<VR, RR> = SemanticOrMaterialMap<VR, RR>;