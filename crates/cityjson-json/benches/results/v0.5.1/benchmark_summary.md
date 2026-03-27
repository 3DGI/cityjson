# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | serde_cityjson | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | owned 856.549 ms (435.6 MiB/s) | 1.242 s (300.5 MiB/s) | 0.69x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | owned 35.907 ms (202.4 MiB/s); borrowed 34.749 ms (209.2 MiB/s) | 27.012 ms (269.1 MiB/s) | 1.33x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | owned 27.236 ms (204.5 MiB/s); borrowed 23.442 ms (237.6 MiB/s) | 18.824 ms (295.9 MiB/s) | 1.45x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | owned 14.583 ms (238.1 MiB/s); borrowed 13.317 ms (260.7 MiB/s) | 12.927 ms (268.6 MiB/s) | 1.13x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | owned 8.332 ms (317.4 MiB/s); borrowed 8.349 ms (316.7 MiB/s) | 10.207 ms (259.1 MiB/s) | 0.82x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | owned 40.063 ms (331.7 MiB/s); borrowed 39.175 ms (339.2 MiB/s) | 57.892 ms (229.6 MiB/s) | 0.69x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | owned 7.406 ms (293.4 MiB/s); borrowed 7.217 ms (301.1 MiB/s) | 6.805 ms (319.3 MiB/s) | 1.09x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | owned 2.395 ms (298.3 MiB/s); borrowed 2.321 ms (307.8 MiB/s) | 2.196 ms (325.3 MiB/s) | 1.09x |

### Write Benchmarks

| Case | Description | serde_cityjson | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | as_json_to_value 2.551 s (146.1 MiB/s); to_string 2.083 s (178.9 MiB/s); to_string_validated 2.088 s (178.6 MiB/s) | 414.105 ms (900.3 MiB/s) | 5.03x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | as_json_to_value 56.464 ms (124.2 MiB/s); to_string 51.235 ms (136.9 MiB/s); to_string_validated 51.355 ms (136.6 MiB/s) | 9.161 ms (765.6 MiB/s) | 5.59x |
| appearance_and_validation_stress | Serializer-heavy case with materials, textures, templates, and semantics | as_json_to_value 9.493 ms (166.1 MiB/s); to_string 7.341 ms (214.8 MiB/s); to_string_validated 7.355 ms (214.4 MiB/s) | 1.981 ms (796.2 MiB/s) | 3.71x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | as_json_to_value 69.734 ms (79.9 MiB/s); to_string 57.463 ms (96.9 MiB/s); to_string_validated 57.428 ms (97.0 MiB/s) | 9.319 ms (597.6 MiB/s) | 6.17x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | as_json_to_value 31.447 ms (110.4 MiB/s); to_string 26.156 ms (132.7 MiB/s); to_string_validated 26.157 ms (132.7 MiB/s) | 5.089 ms (682.2 MiB/s) | 5.14x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | as_json_to_value 16.513 ms (160.1 MiB/s); to_string 14.477 ms (182.7 MiB/s); to_string_validated 14.347 ms (184.3 MiB/s) | 3.938 ms (671.4 MiB/s) | 3.68x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | as_json_to_value 88.551 ms (150.1 MiB/s); to_string 80.917 ms (164.2 MiB/s); to_string_validated 80.733 ms (164.6 MiB/s) | 20.935 ms (634.8 MiB/s) | 3.87x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | as_json_to_value 13.099 ms (165.9 MiB/s); to_string 10.747 ms (202.2 MiB/s); to_string_validated 10.701 ms (203.1 MiB/s) | 2.443 ms (889.4 MiB/s) | 4.40x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | as_json_to_value 3.779 ms (189.0 MiB/s); to_string 3.150 ms (226.8 MiB/s); to_string_validated 3.146 ms (227.1 MiB/s) | 751.118 us (951.1 MiB/s) | 4.19x |
