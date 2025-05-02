INSERT INTO "cityjson"."extension" (name, url, version)
VALUES ('Noise', 'http://schemas.cityjson.org/noise/1.0', '1.0');

INSERT INTO "cityjson"."attributes" ( id, key, type, bool_value, int_value
                                    , uint_value, float_value, string_value
                                    , geometry_value)
VALUES (1, 'float', 'float', NULL, NULL, NULL, 55.0, NULL, NULL)
     , (2, 'bool', 'bool', TRUE, NULL, NULL, NULL, NULL, NULL)
     , (3, 'int', 'int', NULL, 1234567890, NULL, NULL, NULL, NULL)
     , (4, 'uint', 'uint', NULL, NULL, 1234567890, NULL, NULL, NULL)
     , (5, 'string', 'string', NULL, NULL, NULL, NULL, 'test string', NULL)
     , (6, 'null', 'null', NULL, NULL, NULL, NULL, NULL, NULL);

INSERT INTO "cityjson"."metadata" ( geographical_extent, identifier, point_of_contact
                                  , reference_date, reference_system, title)
VALUES ( ST_3DMakeBox(
                 ST_MakePoint(-989502.1875, 528439.5625, 10),
                 ST_MakePoint(-987121.375, 529933.1875, 10))
       , 'asdf-afd-123'
       , jsonb_build_object()
       , date(now())
       , 'EPSG:25832'
       , 'Test City');

INSERT INTO "cityjson"."citymodel" (type_citymodel, version, metadata_id)
VALUES ('CityModel', '2.0', (SELECT id
                             FROM "cityjson"."metadata"
                             WHERE identifier = 'asdf-afd-123'));

INSERT INTO "cityjson"."extensions" (citymodel_id, extension_id)
VALUES ( (SELECT id FROM "cityjson"."citymodel" WHERE type_citymodel = 'CityModel')
       , (SELECT id FROM "cityjson"."extension" WHERE name = 'Noise'));

INSERT INTO "cityjson"."extra_citymodel" (citymodel_id, attribute_id)
VALUES ( (SELECT id FROM "cityjson"."citymodel" WHERE type_citymodel = 'CityModel')
       , (SELECT id FROM "cityjson"."attributes" WHERE key = 'float'))
     , ( (SELECT id FROM "cityjson"."citymodel" WHERE type_citymodel = 'CityModel')
       , (SELECT id FROM "cityjson"."attributes" WHERE key = 'bool'));
