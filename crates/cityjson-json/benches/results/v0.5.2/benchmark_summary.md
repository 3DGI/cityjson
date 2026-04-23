# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-json | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | owned 847.122 ms (440.4 MiB/s) | 1.328 s (280.9 MiB/s) | 0.64x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | owned 35.206 ms (206.5 MiB/s); borrowed 34.153 ms (212.8 MiB/s) | 25.303 ms (287.3 MiB/s) | 1.39x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | owned 30.462 ms (182.8 MiB/s); borrowed 25.802 ms (215.8 MiB/s) | 21.266 ms (261.9 MiB/s) | 1.43x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | owned 15.427 ms (225.0 MiB/s); borrowed 14.255 ms (243.5 MiB/s) | 13.597 ms (255.3 MiB/s) | 1.13x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | owned 8.135 ms (325.1 MiB/s); borrowed 8.104 ms (326.3 MiB/s) | 10.249 ms (258.0 MiB/s) | 0.79x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | owned 39.971 ms (332.5 MiB/s); borrowed 39.550 ms (336.0 MiB/s) | 51.061 ms (260.3 MiB/s) | 0.78x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | owned 7.313 ms (297.1 MiB/s); borrowed 7.071 ms (307.3 MiB/s) | 7.162 ms (303.4 MiB/s) | 1.02x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | owned 2.418 ms (295.4 MiB/s); borrowed 2.321 ms (307.8 MiB/s) | 2.317 ms (308.3 MiB/s) | 1.04x |

### Write Benchmarks

| Case | Description | cityjson-json | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | as_json_to_value 646.119 ms (577.0 MiB/s); to_string 392.985 ms (948.6 MiB/s); to_string_validated 393.093 ms (948.4 MiB/s) | 471.668 ms (790.4 MiB/s) | 0.83x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | as_json_to_value 18.268 ms (383.9 MiB/s); to_string 9.807 ms (715.2 MiB/s); to_string_validated 9.831 ms (713.5 MiB/s) | 9.547 ms (734.7 MiB/s) | 1.03x |
| appearance_and_validation_stress | Serializer-heavy case with materials, textures, templates, and semantics | as_json_to_value 3.924 ms (402.0 MiB/s); to_string 2.030 ms (777.1 MiB/s); to_string_validated 2.023 ms (779.8 MiB/s) | 2.023 ms (779.8 MiB/s) | 1.00x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | as_json_to_value 24.949 ms (223.2 MiB/s); to_string 10.543 ms (528.2 MiB/s); to_string_validated 10.499 ms (530.5 MiB/s) | 10.989 ms (506.8 MiB/s) | 0.96x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | as_json_to_value 10.632 ms (326.5 MiB/s); to_string 6.222 ms (557.9 MiB/s); to_string_validated 6.217 ms (558.4 MiB/s) | 5.123 ms (677.6 MiB/s) | 1.21x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | as_json_to_value 4.959 ms (533.3 MiB/s); to_string 5.245 ms (504.2 MiB/s); to_string_validated 5.250 ms (503.7 MiB/s) | 3.982 ms (664.1 MiB/s) | 1.32x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | as_json_to_value 26.478 ms (501.9 MiB/s); to_string 27.101 ms (490.4 MiB/s); to_string_validated 27.183 ms (488.9 MiB/s) | 22.804 ms (582.8 MiB/s) | 1.19x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | as_json_to_value 4.928 ms (440.9 MiB/s); to_string 3.376 ms (643.6 MiB/s); to_string_validated 3.376 ms (643.6 MiB/s) | 2.457 ms (884.3 MiB/s) | 1.37x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | as_json_to_value 1.545 ms (462.2 MiB/s); to_string 1.082 ms (660.1 MiB/s); to_string_validated 1.087 ms (657.3 MiB/s) | 747.534 us (955.6 MiB/s) | 1.45x |
