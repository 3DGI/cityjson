# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-json | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | owned 29.568 ms (193.8 MiB/s) | 16.634 ms (344.5 MiB/s) | 1.78x |
| io_3dbag_cityjson_cluster_4x |  | owned 106.807 ms (187.9 MiB/s) | 65.983 ms (304.1 MiB/s) | 1.62x |
| io_basisvoorziening_3d_cityjson |  | owned 594.057 ms (285.7 MiB/s) | 618.328 ms (274.5 MiB/s) | 0.96x |
| stress_attribute_heavy |  | owned 8.108 ms (179.8 MiB/s) | 6.473 ms (225.2 MiB/s) | 1.25x |
| stress_boundary_heavy |  | owned 4.287 ms (320.9 MiB/s) | 6.416 ms (214.4 MiB/s) | 0.67x |
| stress_geometry_heavy |  | owned 3.711 ms (279.9 MiB/s) | 4.765 ms (218.0 MiB/s) | 0.78x |
| stress_hierarchy_heavy |  | owned 6.482 ms (193.1 MiB/s) | 5.350 ms (234.0 MiB/s) | 1.21x |
| stress_resource_heavy |  | owned 5.764 ms (162.7 MiB/s) | 3.957 ms (237.1 MiB/s) | 1.46x |
| stress_vertex_heavy |  | owned 11.502 ms (360.1 MiB/s) | 17.050 ms (242.9 MiB/s) | 0.67x |

### Write Benchmarks

| Case | Description | cityjson-json | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | to_vec 8.300 ms (679.0 MiB/s); to_vec_validated 8.329 ms (676.5 MiB/s) | 8.062 ms (699.0 MiB/s) | 1.03x |
| io_3dbag_cityjson_cluster_4x |  | to_vec 40.751 ms (479.3 MiB/s); to_vec_validated 40.500 ms (482.3 MiB/s) | 40.965 ms (476.8 MiB/s) | 0.99x |
| io_basisvoorziening_3d_cityjson |  | to_vec 245.263 ms (687.6 MiB/s); to_vec_validated 244.156 ms (690.7 MiB/s) | 305.802 ms (551.5 MiB/s) | 0.80x |
| stress_attribute_heavy |  | to_vec 2.223 ms (655.7 MiB/s); to_vec_validated 2.244 ms (649.7 MiB/s) | 1.428 ms (1021.1 MiB/s) | 1.56x |
| stress_boundary_heavy |  | to_vec 2.959 ms (464.8 MiB/s); to_vec_validated 2.964 ms (464.0 MiB/s) | 2.449 ms (561.5 MiB/s) | 1.21x |
| stress_geometry_heavy |  | to_vec 2.146 ms (483.8 MiB/s); to_vec_validated 2.148 ms (483.3 MiB/s) | 1.648 ms (630.1 MiB/s) | 1.30x |
| stress_hierarchy_heavy |  | to_vec 2.491 ms (502.2 MiB/s); to_vec_validated 2.502 ms (500.0 MiB/s) | 1.765 ms (708.9 MiB/s) | 1.41x |
| stress_resource_heavy |  | to_vec 1.983 ms (473.0 MiB/s); to_vec_validated 1.979 ms (473.8 MiB/s) | 1.627 ms (576.5 MiB/s) | 1.22x |
| stress_vertex_heavy |  | to_vec 9.033 ms (458.5 MiB/s); to_vec_validated 9.049 ms (457.7 MiB/s) | 8.086 ms (512.2 MiB/s) | 1.12x |
