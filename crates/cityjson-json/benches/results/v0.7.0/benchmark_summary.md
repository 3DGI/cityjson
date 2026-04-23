# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-json | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | owned 29.608 ms (193.5 MiB/s) | 16.829 ms (340.5 MiB/s) | 1.76x |
| io_3dbag_cityjson_cluster_4x |  | owned 107.384 ms (186.9 MiB/s) | 61.862 ms (324.4 MiB/s) | 1.74x |
| io_basisvoorziening_3d_cityjson |  | owned 599.833 ms (282.9 MiB/s) | 623.412 ms (272.2 MiB/s) | 0.96x |
| stress_attribute_heavy |  | owned 8.104 ms (179.9 MiB/s) | 6.428 ms (226.8 MiB/s) | 1.26x |
| stress_boundary_heavy |  | owned 4.290 ms (320.7 MiB/s) | 6.444 ms (213.5 MiB/s) | 0.67x |
| stress_geometry_heavy |  | owned 3.699 ms (280.8 MiB/s) | 4.726 ms (219.8 MiB/s) | 0.78x |
| stress_hierarchy_heavy |  | owned 6.400 ms (195.6 MiB/s) | 5.370 ms (233.1 MiB/s) | 1.19x |
| stress_resource_heavy |  | owned 6.227 ms (150.7 MiB/s) | 4.111 ms (228.2 MiB/s) | 1.51x |
| stress_vertex_heavy |  | owned 11.408 ms (363.1 MiB/s) | 17.049 ms (243.0 MiB/s) | 0.67x |

### Write Benchmarks

| Case | Description | cityjson-json | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | to_vec 8.564 ms (658.0 MiB/s); to_vec_validated 8.694 ms (648.2 MiB/s) | 8.604 ms (654.9 MiB/s) | 1.00x |
| io_3dbag_cityjson_cluster_4x |  | to_vec 41.470 ms (471.0 MiB/s); to_vec_validated 41.239 ms (473.6 MiB/s) | 41.837 ms (466.9 MiB/s) | 0.99x |
| io_basisvoorziening_3d_cityjson |  | to_vec 248.640 ms (678.3 MiB/s); to_vec_validated 247.541 ms (681.3 MiB/s) | 308.833 ms (546.1 MiB/s) | 0.81x |
| stress_attribute_heavy |  | to_vec 2.254 ms (646.6 MiB/s); to_vec_validated 2.262 ms (644.4 MiB/s) | 1.450 ms (1005.0 MiB/s) | 1.55x |
| stress_boundary_heavy |  | to_vec 2.964 ms (463.9 MiB/s); to_vec_validated 2.961 ms (464.4 MiB/s) | 2.429 ms (566.2 MiB/s) | 1.22x |
| stress_geometry_heavy |  | to_vec 2.119 ms (490.1 MiB/s); to_vec_validated 2.123 ms (489.2 MiB/s) | 1.615 ms (642.7 MiB/s) | 1.31x |
| stress_hierarchy_heavy |  | to_vec 2.484 ms (503.6 MiB/s); to_vec_validated 2.491 ms (502.3 MiB/s) | 1.772 ms (705.9 MiB/s) | 1.40x |
| stress_resource_heavy |  | to_vec 1.971 ms (475.8 MiB/s); to_vec_validated 1.970 ms (476.1 MiB/s) | 1.660 ms (564.7 MiB/s) | 1.19x |
| stress_vertex_heavy |  | to_vec 9.108 ms (454.7 MiB/s); to_vec_validated 9.113 ms (454.5 MiB/s) | 8.173 ms (506.7 MiB/s) | 1.11x |
