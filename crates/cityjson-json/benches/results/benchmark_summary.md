# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | serde_cityjson | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | owned 821.944 ms (453.9 MiB/s) | 1.242 s (300.5 MiB/s) | 0.66x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | owned 35.435 ms (205.1 MiB/s); borrowed 34.607 ms (210.0 MiB/s) | 23.360 ms (311.2 MiB/s) | 1.52x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | owned 27.007 ms (206.2 MiB/s); borrowed 21.911 ms (254.2 MiB/s) | 19.336 ms (288.0 MiB/s) | 1.40x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | owned 14.500 ms (239.4 MiB/s); borrowed 13.192 ms (263.2 MiB/s) | 13.333 ms (260.4 MiB/s) | 1.09x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | owned 8.339 ms (317.1 MiB/s); borrowed 8.336 ms (317.2 MiB/s) | 10.190 ms (259.5 MiB/s) | 0.82x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | owned 40.135 ms (331.1 MiB/s); borrowed 39.853 ms (333.5 MiB/s) | 51.264 ms (259.2 MiB/s) | 0.78x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | owned 7.399 ms (293.6 MiB/s); borrowed 7.301 ms (297.6 MiB/s) | 6.823 ms (318.4 MiB/s) | 1.08x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | owned 2.413 ms (296.1 MiB/s); borrowed 2.385 ms (299.5 MiB/s) | 2.216 ms (322.4 MiB/s) | 1.09x |

### Write Benchmarks

| Case | Description | serde_cityjson | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | to_string 2.056 s (181.3 MiB/s); to_string_validated 2.058 s (181.2 MiB/s) | 505.770 ms (737.1 MiB/s) | 4.07x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | to_string 45.950 ms (152.6 MiB/s); to_string_validated 48.937 ms (143.3 MiB/s) | 10.997 ms (637.8 MiB/s) | 4.18x |
| appearance_and_validation_stress | Serializer-heavy case with materials, textures, templates, and semantics | to_string 7.187 ms (219.5 MiB/s); to_string_validated 7.451 ms (211.7 MiB/s) | 1.840 ms (857.2 MiB/s) | 3.91x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | to_string 57.078 ms (97.6 MiB/s); to_string_validated 56.207 ms (99.1 MiB/s) | 10.384 ms (536.3 MiB/s) | 5.50x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | to_string 25.851 ms (134.3 MiB/s); to_string_validated 25.928 ms (133.9 MiB/s) | 7.169 ms (484.3 MiB/s) | 3.61x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | to_string 14.376 ms (183.9 MiB/s); to_string_validated 14.462 ms (182.9 MiB/s) | 4.257 ms (621.2 MiB/s) | 3.38x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | to_string 78.413 ms (169.5 MiB/s); to_string_validated 98.400 ms (135.1 MiB/s) | 25.328 ms (524.7 MiB/s) | 3.10x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | to_string 11.547 ms (188.2 MiB/s); to_string_validated 10.369 ms (209.5 MiB/s) | 2.686 ms (808.8 MiB/s) | 4.30x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | to_string 3.268 ms (218.6 MiB/s); to_string_validated 3.150 ms (226.8 MiB/s) | 773.753 us (923.3 MiB/s) | 4.22x |
