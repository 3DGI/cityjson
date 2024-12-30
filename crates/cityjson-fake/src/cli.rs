use clap::Parser;

type IndexType = u32;

/// Configuration for CityJSON fake data generation
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct CJFakeConfig {
    /// Minimum number of CityObjects to generate
    #[arg(long, default_value_t = 1)]
    pub min_nr_cityobjects: IndexType,

    /// Maximum number of CityObjects to generate
    #[arg(long, default_value_t = 1)]
    pub max_nr_cityobjects: IndexType,

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
    pub min_nr_vertices: IndexType,

    /// Maximum number of vertices in geometry objects
    #[arg(long, default_value_t = 8)]
    pub max_nr_vertices: IndexType,

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
    pub min_nr_materials: IndexType,

    /// Maximum number of materials
    #[arg(long, default_value_t = 3)]
    pub max_nr_materials: IndexType,

    /// Number of material themes
    #[arg(long, default_value_t = 3)]
    pub nr_themes_materials: IndexType,

    /// Minimum number of textures
    #[arg(long, default_value_t = 2)]
    pub min_nr_textures: IndexType,

    /// Maximum number of textures
    #[arg(long, default_value_t = 2)]
    pub max_nr_textures: IndexType,

    /// Number of texture themes
    #[arg(long, default_value_t = 3)]
    pub nr_themes_textures: IndexType,

    /// Maximum number of vertices in texture coordinates
    #[arg(long, default_value_t = 10)]
    pub max_nr_vertices_texture: IndexType,

    /// Minimum number of templates
    #[arg(long, default_value_t = 1)]
    pub min_nr_templates: IndexType,

    /// Maximum number of templates
    #[arg(long, default_value_t = 10)]
    pub max_nr_templates: IndexType,
}

impl Default for CJFakeConfig {
    fn default() -> Self {
        Self {
            min_nr_cityobjects: 1,
            max_nr_cityobjects: 1,
            cityobject_hierarchy: true,
            min_coordinate: i64::MIN,
            max_coordinate: i64::MAX,
            min_nr_vertices: 8,
            max_nr_vertices: 8,
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
            min_nr_materials: 1,
            max_nr_materials: 3,
            nr_themes_materials: 3,
            min_nr_textures: 2,
            max_nr_textures: 2,
            nr_themes_textures: 3,
            max_nr_vertices_texture: 10,
            min_nr_templates: 1,
            max_nr_templates: 10,
        }
    }
}
