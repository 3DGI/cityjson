-- Query 1: Select all cityobjects within a polygon
SELECT cm.id              AS citymodel_id
     , cm.type_citymodel
     , cm.version
     , v.id               AS vertex_id
     , ST_AsText(v.point) AS point_wkt
FROM cityjson.citymodel cm
         JOIN cityjson.citymodel_vertices cv ON cm.id = cv.citymodel_id
         JOIN cityjson.vertices v ON cv.vertex_id = v.id
WHERE ST_Within(
              v.point,
              ST_GeomFromText('POLYGON((0 0, 15 0, 15 15, 0 15, 0 0))')
      );

-- Query 2: Select all cityobjects that have an attribute 'floors' that is less than or equal to 5
SELECT cm.id       AS citymodel_id
     , cm.type_citymodel
     , cm.version
     , a.key       AS attribute_key
     , a.int_value AS floors
FROM cityjson.citymodel cm
         JOIN cityjson.extra_citymodel ecm ON cm.id = ecm.citymodel_id
         JOIN cityjson.attributes a ON ecm.attribute_id = a.id
WHERE a.key = 'floors'
  AND a.int_value <= 5;

-- Query 3: Select the boundary surface of all roof surfaces
SELECT s.id AS semantic_id
     , s.type_semantic
     , g.id AS geometry_id
     , g.type_geometry
     , g.lod
     , b.id AS boundary_id
     , b.vertices
     , b.rings
     , b.surfaces
FROM cityjson.semantics s
         JOIN
     cityjson.semantic_map sm ON s.id = ANY (sm.surfaces)
         JOIN
     cityjson.geometry g ON g.semantics = sm.id
         JOIN
     cityjson.boundary b ON g.boundaries = b.id
WHERE s.type_semantic = 'RoofSurface';
