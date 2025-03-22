-- Enums for type safety
CREATE TYPE city_object_type AS ENUM (
    'Bridge', 'BridgePart', 'BridgeInstallation', 'BridgeConstructiveElement', 'BridgeRoom', 'BridgeFurniture',
    'Building', 'BuildingPart', 'BuildingInstallation', 'BuildingConstructiveElement', 'BuildingFurniture',
    'BuildingStorey', 'BuildingRoom', 'BuildingUnit', 'CityFurniture', 'CityObjectGroup', 'Default',
    'LandUse', 'OtherConstruction', 'PlantCover', 'SolitaryVegetationObject', 'TINRelief', 'WaterBody',
    'Road', 'Railway', 'Waterway', 'TransportSquare', 'Tunnel', 'TunnelPart', 'TunnelInstallation',
    'TunnelConstructiveElement', 'TunnelHollowSpace', 'TunnelFurniture'
);

CREATE TYPE semantic_type AS ENUM (
    'RoofSurface', 'GroundSurface', 'WallSurface', 'ClosureSurface', 'OuterCeilingSurface',
    'OuterFloorSurface', 'Window', 'Door', 'InteriorWallSurface', 'CeilingSurface',
    'FloorSurface', 'WaterSurface', 'WaterGroundSurface', 'WaterClosureSurface',
    'TrafficArea', 'AuxiliaryTrafficArea', 'TransportationMarking', 'TransportationHole'
);

CREATE TYPE geometry_type AS ENUM (
    'MultiPoint', 'MultiLineString', 'MultiSurface', 'CompositeSurface',
    'Solid', 'MultiSolid', 'CompositeSolid'
);

CREATE TYPE image_type AS ENUM ('PNG', 'JPG');
CREATE TYPE wrap_mode AS ENUM ('REPEAT', 'MIRROR', 'CLAMP');
CREATE TYPE texture_type AS ENUM ('DIFFUSE', 'SPECULAR', 'NORMAL', 'EMISSIVE');

-- Table for extension types
CREATE TABLE extension_types (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

-- Main CityModel table
CREATE TABLE city_models (
    id SERIAL PRIMARY KEY,
    title TEXT,
    identifier TEXT,
    geographical_extent JSONB,
    reference_date DATE,
    reference_system TEXT,
    point_of_contact JSONB,
    extra JSONB
);

-- Transform for coordinate transformations
CREATE TABLE transforms (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    scale DOUBLE PRECISION[3],
    translate DOUBLE PRECISION[3]
);

-- Vertices tables
CREATE TABLE vertices_real (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    z DOUBLE PRECISION NOT NULL
);

CREATE TABLE vertices_texture (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    u REAL NOT NULL,
    v REAL NOT NULL
);

-- Materials
CREATE TABLE materials (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    ambient_intensity REAL,
    diffuse_color JSONB, -- {r, g, b}
    emissive_color JSONB, -- {r, g, b}
    specular_color JSONB, -- {r, g, b}
    shininess REAL,
    transparency REAL,
    is_smooth BOOLEAN
);

-- Textures
CREATE TABLE textures (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    image_type image_type NOT NULL,
    image TEXT NOT NULL, -- Path or URL to image
    wrap_mode wrap_mode,
    texture_type texture_type,
    border_color JSONB -- {r, g, b, a}
);

-- Semantic surfaces
CREATE TABLE semantics (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    type semantic_type,
    extension_type_id INTEGER REFERENCES extension_types(id),
    parent_id INTEGER REFERENCES semantics(id),
    attributes JSONB,
    CHECK ((type IS NOT NULL AND extension_type_id IS NULL) OR
           (type IS NULL AND extension_type_id IS NOT NULL))
);

-- Semantic parent-child relationships
CREATE TABLE semantic_children (
    parent_id INTEGER REFERENCES semantics(id) ON DELETE CASCADE,
    child_id INTEGER REFERENCES semantics(id) ON DELETE CASCADE,
    PRIMARY KEY (parent_id, child_id)
);

-- Boundary storage
CREATE TABLE boundaries (
    id SERIAL PRIMARY KEY,
    vertices JSONB NOT NULL, -- Array of vertex indices
    rings JSONB NOT NULL,    -- Array of ring indices
    surfaces JSONB NOT NULL, -- Array of surface indices
    shells JSONB NOT NULL,   -- Array of shell indices
    solids JSONB NOT NULL    -- Array of solid indices
);

-- Geometries
CREATE TABLE geometries (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    type geometry_type NOT NULL,
    lod REAL,
    boundary_id INTEGER REFERENCES boundaries(id),
    template_boundaries INTEGER,
    template_transformation_matrix DOUBLE PRECISION[16]
);

-- Semantic maps (linking semantics to geometry parts)
CREATE TABLE semantic_maps (
    id SERIAL PRIMARY KEY,
    geometry_id INTEGER REFERENCES geometries(id) ON DELETE CASCADE,
    points JSONB,      -- Array of semantic refs for points
    linestrings JSONB, -- Array of semantic refs for linestrings
    surfaces JSONB,    -- Array of semantic refs for surfaces
    shells JSONB,      -- Array of indices
    solids JSONB       -- Array of indices
);

-- Material maps (linking materials to geometry parts)
CREATE TABLE material_maps (
    id SERIAL PRIMARY KEY,
    geometry_id INTEGER REFERENCES geometries(id) ON DELETE CASCADE,
    points JSONB,      -- Array of material refs
    linestrings JSONB, -- Array of material refs
    surfaces JSONB,    -- Array of material refs
    shells JSONB,      -- Array of indices
    solids JSONB       -- Array of indices
);

-- Texture maps (linking textures to geometry parts)
CREATE TABLE texture_maps (
    id SERIAL PRIMARY KEY,
    geometry_id INTEGER REFERENCES geometries(id) ON DELETE CASCADE,
    vertices JSONB,     -- Array of texture vertex indices
    rings JSONB,        -- Array of indices
    ring_textures JSONB, -- Array of texture refs
    surfaces JSONB,     -- Array of indices
    shells JSONB,       -- Array of indices
    solids JSONB        -- Array of indices
);

-- City Objects
CREATE TABLE city_objects (
    id SERIAL PRIMARY KEY,
    city_model_id INTEGER REFERENCES city_models(id) ON DELETE CASCADE,
    type city_object_type,
    extension_type_id INTEGER REFERENCES extension_types(id),
    geographical_extent JSONB, -- BBox {min_x, min_y, min_z, max_x, max_y, max_z}
    attributes JSONB,
    extra JSONB,
    CHECK ((type != 'Extension' AND extension_type_id IS NULL) OR
           (type = 'Extension' AND extension_type_id IS NOT NULL))
);

-- City Object hierarchy (parent-child relationships)
CREATE TABLE city_object_relationships (
    parent_id INTEGER REFERENCES city_objects(id) ON DELETE CASCADE,
    child_id INTEGER REFERENCES city_objects(id) ON DELETE CASCADE,
    PRIMARY KEY (parent_id, child_id)
);

-- Link between city objects and geometries
CREATE TABLE city_object_geometries (
    city_object_id INTEGER REFERENCES city_objects(id) ON DELETE CASCADE,
    geometry_id INTEGER REFERENCES geometries(id) ON DELETE CASCADE,
    PRIMARY KEY (city_object_id, geometry_id)
);

-- Table for attribute values (can be shared by multiple entities)
CREATE TABLE attributes (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    value JSONB NOT NULL
);