use clap::Parser;
use serde_cityjson::v1_1::{CityObjectType, GeometryType};
type IndexType = u32;

/// Configuration for CityJSON fake data generation
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct CJFakeConfig {
    /// Restrict the CityObject types to the provided types
    #[arg(long, value_delimiter = ',', value_parser = parse_cityobject_type )]
    pub allowed_types_cityobject: Option<Vec<CityObjectType>>,

    /// Restrict the Geometry types to the provided types
    #[arg(long, value_delimiter = ',', value_parser = parse_geometry_type )]
    pub allowed_types_geometry: Option<Vec<GeometryType>>,

    /// Minimum number of CityObjects to generate
    #[arg(long, default_value_t = 1)]
    pub min_cityobjects: IndexType,

    /// Maximum number of CityObjects to generate
    #[arg(long, default_value_t = 1)]
    pub max_cityobjects: IndexType,

    /// Whether to generate hierarchical CityObjects (parent-child relationships)
    #[arg(long, default_value_t = true)]
    pub cityobject_hierarchy: bool,

    /// Minimum coordinate value for geometry vertices
    #[arg(long, default_value_t = i64::MIN)]
    pub min_coordinate: i64,

    /// Maximum coordinate value for geometry vertices
    #[arg(long, default_value_t = i64::MAX)]
    pub max_coordinate: i64,

    /// Minimum number of vertices in geometry objects
    #[arg(long, default_value_t = 8)]
    pub min_vertices: IndexType,

    /// Maximum number of vertices in geometry objects
    #[arg(long, default_value_t = 8)]
    pub max_vertices: IndexType,

    /// Minimum number of points in MultiPoint geometries
    #[arg(long, default_value_t = 11)]
    pub min_members_multipoint: IndexType,

    /// Maximum number of points in MultiPoint geometries
    #[arg(long, default_value_t = 11)]
    pub max_members_multipoint: IndexType,

    /// Minimum number of linestrings in MultiLineString geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_multilinestring: IndexType,

    /// Maximum number of linestrings in MultiLineString geometries
    #[arg(long, default_value_t = 1)]
    pub max_members_multilinestring: IndexType,

    /// Minimum number of surfaces in MultiSurface geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_multisurface: IndexType,

    /// Maximum number of surfaces in MultiSurface geometries
    #[arg(long, default_value_t = 1)]
    pub max_members_multisurface: IndexType,

    /// Minimum number of shells in Solid geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_solid: IndexType,

    /// Maximum number of shells in Solid geometries
    #[arg(long, default_value_t = 3)]
    pub max_members_solid: IndexType,

    /// Minimum number of solids in MultiSolid geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_multisolid: IndexType,

    /// Maximum number of solids in MultiSolid geometries
    #[arg(long, default_value_t = 3)]
    pub max_members_multisolid: IndexType,

    /// Minimum number of geometries per CityObject
    #[arg(long, default_value_t = 1)]
    pub min_members_cityobject_geometries: IndexType,

    /// Maximum number of geometries per CityObject
    #[arg(long, default_value_t = 1)]
    pub max_members_cityobject_geometries: IndexType,

    /// Minimum number of materials
    #[arg(long, default_value_t = 1)]
    pub min_materials: IndexType,

    /// Maximum number of materials
    #[arg(long, default_value_t = 3)]
    pub max_materials: IndexType,

    /// Number of material themes
    #[arg(long, default_value_t = 3)]
    pub nr_themes_materials: IndexType,

    /// Minimum number of textures
    #[arg(long, default_value_t = 2)]
    pub min_textures: IndexType,

    /// Maximum number of textures
    #[arg(long, default_value_t = 2)]
    pub max_textures: IndexType,

    /// Number of texture themes
    #[arg(long, default_value_t = 3)]
    pub nr_themes_textures: IndexType,

    /// Maximum number of vertices in texture coordinates
    #[arg(long, default_value_t = 10)]
    pub max_vertices_textures: IndexType,

    /// Minimum number of templates
    #[arg(long, default_value_t = 1)]
    pub min_templates: IndexType,

    /// Maximum number of templates
    #[arg(long, default_value_t = 10)]
    pub max_templates: IndexType,

    /// Generate GeometryInstances too
    #[arg(long, default_value_t = true)]
    pub use_templates: bool,

    /// Allow null in the texture values
    #[arg(long, default_value_t = false)]
    pub texture_allow_none: bool,
}

impl Default for CJFakeConfig {
    fn default() -> Self {
        Self {
            allowed_types_cityobject: None,
            allowed_types_geometry: None,
            min_cityobjects: 1,
            max_cityobjects: 1,
            cityobject_hierarchy: true,
            min_coordinate: i64::MIN,
            max_coordinate: i64::MAX,
            min_vertices: 8,
            max_vertices: 8,
            min_members_multipoint: 11,
            max_members_multipoint: 11,
            min_members_multilinestring: 1,
            max_members_multilinestring: 1,
            min_members_multisurface: 1,
            max_members_multisurface: 1,
            min_members_solid: 1,
            max_members_solid: 3,
            min_members_multisolid: 1,
            max_members_multisolid: 3,
            min_members_cityobject_geometries: 1,
            max_members_cityobject_geometries: 1,
            min_materials: 1,
            max_materials: 3,
            nr_themes_materials: 3,
            min_textures: 2,
            max_textures: 2,
            nr_themes_textures: 3,
            max_vertices_textures: 10,
            min_templates: 1,
            max_templates: 10,
            use_templates: true,
            texture_allow_none: false,
        }
    }
}

fn parse_cityobject_type(s: &str) -> Result<CityObjectType, String> {
    serde_json::from_str(&format!(r#""{}""#, s)).map_err(|e| format!("Failed to parse CityObjectType: {}", e))
}

fn parse_geometry_type(s: &str) -> Result<GeometryType, String> {
    serde_json::from_str(&format!(r#""{}""#, s)).map_err(|e| format!("Failed to parse GeometryType: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cjfakeconfig_defaults() {
        let config = CJFakeConfig::default();

        assert_eq!(config.allowed_types_cityobject, None);
        assert_eq!(config.allowed_types_geometry, None);
        assert_eq!(config.min_cityobjects, 1);
        assert_eq!(config.max_cityobjects, 1);
        assert_eq!(config.cityobject_hierarchy, true);
        assert_eq!(config.min_coordinate, i64::MIN);
        assert_eq!(config.max_coordinate, i64::MAX);
        assert_eq!(config.min_vertices, 8);
        assert_eq!(config.max_vertices, 8);
        assert_eq!(config.min_members_multipoint, 11);
        assert_eq!(config.max_members_multipoint, 11);
        assert_eq!(config.min_members_multilinestring, 1);
        assert_eq!(config.max_members_multilinestring, 1);
        assert_eq!(config.min_members_multisurface, 1);
        assert_eq!(config.max_members_multisurface, 1);
        assert_eq!(config.min_members_solid, 1);
        assert_eq!(config.max_members_solid, 3);
        assert_eq!(config.min_members_multisolid, 1);
        assert_eq!(config.max_members_multisolid, 3);
        assert_eq!(config.min_members_cityobject_geometries, 1);
        assert_eq!(config.max_members_cityobject_geometries, 1);
        assert_eq!(config.min_materials, 1);
        assert_eq!(config.max_materials, 3);
        assert_eq!(config.nr_themes_materials, 3);
        assert_eq!(config.min_textures, 2);
        assert_eq!(config.max_textures, 2);
        assert_eq!(config.nr_themes_textures, 3);
        assert_eq!(config.max_vertices_textures, 10);
        assert_eq!(config.min_templates, 1);
        assert_eq!(config.max_templates, 10);
        assert_eq!(config.use_templates, true);
        assert_eq!(config.texture_allow_none, false);
    }

    #[test]
    fn test_cli_argument_parsing() {
        let args = vec![
            "cjfake",
            "--allowed-types-cityobject",
            "Building,Bridge",
            "--allowed-types-geometry",
            "MultiSurface,Solid",
            "--min-cityobjects",
            "5",
            "--max-cityobjects",
            "10",
            "--cityobject-hierarchy",
            "--min-coordinate=-1000",
            "--max-coordinate=1000",
            "--min-vertices",
            "4",
            "--max-vertices",
            "20",
            "--min-members-multipoint",
            "2",
            "--max-members-multipoint",
            "5",
            "--min-members-multilinestring",
            "3",
            "--max-members-multilinestring",
            "6",
            "--min-members-multisurface",
            "1",
            "--max-members-multisurface",
            "3",
            "--min-members-solid",
            "2",
            "--max-members-solid",
            "4",
            "--min-members-multisolid",
            "1",
            "--max-members-multisolid",
            "2",
            "--min-members-cityobject-geometries",
            "1",
            "--max-members-cityobject-geometries",
            "3",
            "--min-materials",
            "1",
            "--max-materials",
            "2",
            "--nr-themes-materials",
            "2",
            "--min-textures",
            "1",
            "--max-textures",
            "3",
            "--nr-themes-textures",
            "2",
            "--max-vertices-textures",
            "15",
            "--min-templates",
            "2",
            "--max-templates",
            "5",
            "--use-templates",
            "--texture-allow-none",
        ];

        let config = CJFakeConfig::parse_from(args);

        assert_eq!(
            config.allowed_types_cityobject,
            Some(vec![CityObjectType::Building, CityObjectType::Bridge])
        );
        assert_eq!(
            config.allowed_types_geometry,
            Some(vec![GeometryType::MultiSurface, GeometryType::Solid])
        );
        assert_eq!(config.min_cityobjects, 5);
        assert_eq!(config.max_cityobjects, 10);
        assert_eq!(config.cityobject_hierarchy, true);
        assert_eq!(config.min_coordinate, -1000);
        assert_eq!(config.max_coordinate, 1000);
        assert_eq!(config.min_vertices, 4);
        assert_eq!(config.max_vertices, 20);
        assert_eq!(config.min_members_multipoint, 2);
        assert_eq!(config.max_members_multipoint, 5);
        assert_eq!(config.min_members_multilinestring, 3);
        assert_eq!(config.max_members_multilinestring, 6);
        assert_eq!(config.min_members_multisurface, 1);
        assert_eq!(config.max_members_multisurface, 3);
        assert_eq!(config.min_members_solid, 2);
        assert_eq!(config.max_members_solid, 4);
        assert_eq!(config.min_members_multisolid, 1);
        assert_eq!(config.max_members_multisolid, 2);
        assert_eq!(config.min_members_cityobject_geometries, 1);
        assert_eq!(config.max_members_cityobject_geometries, 3);
        assert_eq!(config.min_materials, 1);
        assert_eq!(config.max_materials, 2);
        assert_eq!(config.nr_themes_materials, 2);
        assert_eq!(config.min_textures, 1);
        assert_eq!(config.max_textures, 3);
        assert_eq!(config.nr_themes_textures, 2);
        assert_eq!(config.max_vertices_textures, 15);
        assert_eq!(config.min_templates, 2);
        assert_eq!(config.max_templates, 5);
        assert_eq!(config.use_templates, true);
        assert_eq!(config.texture_allow_none, true);
    }
}
