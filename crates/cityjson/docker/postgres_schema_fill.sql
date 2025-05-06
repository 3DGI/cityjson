-- Sample data for CityJSON PostgreSQL schema
BEGIN;

-- Create the schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS cityjson;

-- 1. Add metadata entries
INSERT INTO cityjson.metadata (geographical_extent, identifier, point_of_contact, reference_date, reference_system, title)
VALUES
(ST_MakeEnvelope(0, 0, 100, 100, 4326), 'city-model-001',
 '{"contact_name": "John Doe", "email_address": "john.doe@example.com", "role": "Author", "contact_type": "Individual"}',
 '2023-01-15', 'https://www.opengis.net/def/crs/EPSG/0/4326', 'Example City Model'),
(ST_MakeEnvelope(100, 100, 200, 200, 3857), 'city-model-002',
 '{"contact_name": "Jane Smith", "email_address": "jane.smith@example.com", "role": "Publisher", "contact_type": "Organization"}',
 '2023-02-20', 'https://www.opengis.net/def/crs/EPSG/0/3857', 'Another Example City Model');

-- 2. Add transform entries
INSERT INTO cityjson.transform (scale_x, scale_y, scale_z, translate_x, translate_y, translate_z)
VALUES
(0.001, 0.001, 0.001, 4424648.79, 5427344.63, 12.0),
(0.01, 0.01, 0.01, 3856245.22, 5128732.45, 25.5);

-- 3. Add city models
INSERT INTO cityjson.citymodel (type_citymodel, version, metadata_id, transform_id)
VALUES
('CityJSON', '2.0', 1, 1),
('CityJSONFeature', '2.0', 2, 2);

-- 4. Add extension entries
INSERT INTO cityjson.extension (name, url, version)
VALUES
('Noise', 'https://example.com/extensions/noise/1.0', '1.0'),
('Solar', 'https://example.com/extensions/solar/1.2', '1.2');

-- 5. Link extensions to city models
INSERT INTO cityjson.extensions (citymodel_id, extension_id)
VALUES
(1, 1),
(1, 2),
(2, 2);

-- 6. Add vertices
INSERT INTO cityjson.vertices (point)
VALUES
(ST_MakePoint(0, 0, 0)),
(ST_MakePoint(10, 0, 0)),
(ST_MakePoint(10, 10, 0)),
(ST_MakePoint(0, 10, 0)),
(ST_MakePoint(0, 0, 10)),
(ST_MakePoint(10, 0, 10)),
(ST_MakePoint(10, 10, 10)),
(ST_MakePoint(0, 10, 10)),
(ST_MakePoint(20, 20, 0)),
(ST_MakePoint(30, 20, 0)),
(ST_MakePoint(30, 30, 0)),
(ST_MakePoint(20, 30, 0));

-- 7. Link vertices to city models
INSERT INTO cityjson.citymodel_vertices (citymodel_id, vertex_id)
VALUES
(1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7), (1, 8),
(2, 9), (2, 10), (2, 11), (2, 12);

-- 8. Add semantic objects
INSERT INTO cityjson.semantics (type_semantic)
VALUES
('RoofSurface'),
('WallSurface'),
('GroundSurface'),
('Door'),
('Window');

-- 9. Link semantics to city models
INSERT INTO cityjson.citymodel_semantics (citymodel_id, semantic_id)
VALUES
(1, 1), (1, 2), (1, 3), (1, 4), (1, 5),
(2, 1), (2, 2), (2, 3);

-- 10. Set up semantic hierarchy
INSERT INTO cityjson.semantics_children (parent_id, child_id)
VALUES
(2, 4),  -- Door is a child of WallSurface
(2, 5);  -- Window is a child of WallSurface

-- 11. Add attributes
INSERT INTO cityjson.attributes (key, type, bool_value, int_value, uint_value, float_value, string_value)
VALUES
('isExterior', 'Bool', true, NULL, NULL, NULL, NULL),
('transparency', 'Float', NULL, NULL, NULL, 0.8, NULL),
('material', 'String', NULL, NULL, NULL, NULL, 'brick'),
('height', 'Float', NULL, NULL, NULL, 25.5, NULL),
('name', 'String', NULL, NULL, NULL, NULL, 'Main Building'),
('floors', 'Integer', NULL, 5, NULL, NULL, NULL),
('built', 'Integer', NULL, 2010, NULL, NULL, NULL);

-- 12. Link attributes to semantics
INSERT INTO cityjson.semantic_attributes (semantic_id, attribute_id)
VALUES
(1, 2),  -- RoofSurface has transparency attribute
(2, 1),  -- WallSurface has isExterior attribute
(2, 3),  -- WallSurface has material attribute
(4, 2),  -- Door has transparency attribute
(5, 2);  -- Window has transparency attribute

-- 13. Add extra attributes to city models and metadata
INSERT INTO cityjson.extra_citymodel (citymodel_id, attribute_id)
VALUES
(1, 4),  -- City model 1 has height attribute
(1, 5),  -- City model 1 has name attribute
(2, 6),  -- City model 2 has floors attribute
(2, 7);  -- City model 2 has built attribute

INSERT INTO cityjson.extra_metadata (metadata_id, attribute_id)
VALUES
(1, 5),  -- Metadata 1 has name attribute
(2, 7);  -- Metadata 2 has built attribute

-- 14. Add materials
INSERT INTO cityjson.material (name, ambient_intensity, diffuse_color, emissive_color, specular_color, shininess, transparency, is_smooth)
VALUES
('Brick', 0.5, ARRAY[0.8, 0.3, 0.2], NULL, ARRAY[0.9, 0.9, 0.9], 0.2, 0.0, false),
('Glass', 0.7, ARRAY[0.9, 0.9, 0.95], NULL, ARRAY[1.0, 1.0, 1.0], 0.8, 0.6, true),
('Concrete', 0.4, ARRAY[0.7, 0.7, 0.7], NULL, ARRAY[0.5, 0.5, 0.5], 0.1, 0.0, false);

-- 15. Link materials to city models
INSERT INTO cityjson.citymodel_material (citymodel_id, material_id)
VALUES
(1, 1), (1, 2),  -- City model 1 has Brick and Glass materials
(2, 2), (2, 3);  -- City model 2 has Glass and Concrete materials

-- 16. Add textures
INSERT INTO cityjson.texture (image_type, image, wrap_mode, texture_type, border_color)
VALUES
('JPG', 'textures/brick.jpg', 'Wrap', 'Specific', NULL),
('PNG', 'textures/window.png', 'Clamp', 'Typical', ARRAY[0.0, 0.0, 0.0, 1.0]),
('JPG', 'textures/concrete.jpg', 'Wrap', 'Specific', NULL);

-- 17. Link textures to city models
INSERT INTO cityjson.citymodel_texture (citymodel_id, texture_id)
VALUES
(1, 1), (1, 2),  -- City model 1 has brick and window textures
(2, 2), (2, 3);  -- City model 2 has window and concrete textures

-- 18. Add boundaries
INSERT INTO cityjson.boundary (id, vertices, rings, surfaces, shells, solids)
VALUES
(1, ARRAY[1, 2, 3, 4, 1], ARRAY[0], ARRAY[0], NULL, NULL),  -- Simple surface (one ring)
(2, ARRAY[1, 2, 3, 4, 1, 5, 6, 7, 8, 5], ARRAY[0, 5], ARRAY[0, 1], ARRAY[0], NULL),  -- Simple solid (box with two surfaces)
(3, ARRAY[9, 10, 11, 12, 9], ARRAY[0], ARRAY[0], NULL, NULL);  -- Another simple surface

-- 19. Add semantic maps
INSERT INTO cityjson.semantic_map (id, points, linestrings, surfaces, shells, solids)
VALUES
(1, NULL, NULL, ARRAY[1], NULL, NULL),  -- Map semantic 1 (RoofSurface) to surface 1
(2, NULL, NULL, ARRAY[1, 2], NULL, NULL),  -- Map semantics to solid surfaces
(3, NULL, NULL, ARRAY[3], NULL, NULL);  -- Map semantic 3 (GroundSurface) to surface 3

-- 20. Add material maps
INSERT INTO cityjson.material_map (id, theme, surfaces)
VALUES
(1, 'default', ARRAY[1]),  -- Map Brick material to surface 1
(2, 'default', ARRAY[1, 2]);  -- Map materials to solid surfaces

-- 21. Add texture maps
INSERT INTO cityjson.texture_map (id, theme, vertices, rings, rings_textures, surfaces)
VALUES
(1, 'default', ARRAY[1, 2, 3, 4], ARRAY[0], ARRAY[1], ARRAY[0]),  -- Map texture 1 to surface 1 vertices
(2, 'default', ARRAY[9, 10, 11, 12], ARRAY[0], ARRAY[2], ARRAY[0]);  -- Map texture 2 to surface 2 vertices

-- 22. Add geometries
INSERT INTO cityjson.geometry (id, type_geometry, lod, boundaries, semantics, instance_template, instance_reference_point, instance_transformation_matrix)
VALUES
(1, 'MultiSurface', 'LoD2', 1, 1, NULL, NULL, NULL),  -- MultiSurface geometry
(2, 'Solid', 'LoD2', 2, 2, NULL, NULL, NULL),  -- Solid geometry
(3, 'MultiSurface', 'LoD1', 3, 3, NULL, NULL, NULL),  -- Another MultiSurface geometry
(4, 'GeometryInstance', 'LoD2', NULL, NULL, 1, 9, ARRAY[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 10.0, 0.0, 1.0]);  -- Geometry instance

-- 23. Link material maps to geometries
INSERT INTO cityjson.geometry_material_map (geometry_id, material_map_id)
VALUES
(1, 1),
(2, 2);

-- 24. Link texture maps to geometries
INSERT INTO cityjson.geometry_texture_map (geometry_id, texture_map_id)
VALUES
(1, 1),
(3, 2);

-- 25. Link geometries to city models
INSERT INTO cityjson.citymodel_geometries (citymodel_id, geometry_id)
VALUES
(1, 1), (1, 2),  -- City model 1 has MultiSurface and Solid geometries
(2, 3), (2, 4);  -- City model 2 has MultiSurface and GeometryInstance geometries


COMMIT;