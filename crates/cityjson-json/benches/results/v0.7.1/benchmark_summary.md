# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-json | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | owned 29.573 ms (193.8 MiB/s) | 16.787 ms (341.4 MiB/s) | 1.76x |
| io_3dbag_cityjson_cluster_4x |  | owned 108.088 ms (185.6 MiB/s) | 61.538 ms (326.1 MiB/s) | 1.76x |
| io_basisvoorziening_3d_cityjson |  | owned 597.308 ms (284.1 MiB/s) | 612.935 ms (276.9 MiB/s) | 0.97x |
| stress_attribute_heavy_heterogenous |  | owned 8.557 ms (158.2 MiB/s) | 6.842 ms (197.8 MiB/s) | 1.25x |
| stress_attribute_heavy_homogenous |  | owned 6.075 ms (168.9 MiB/s) | 4.827 ms (212.5 MiB/s) | 1.26x |
| stress_boundary_heavy |  | owned 4.302 ms (319.8 MiB/s) | 6.464 ms (212.9 MiB/s) | 0.67x |
| stress_geometry_heavy |  | owned 3.725 ms (278.9 MiB/s) | 4.751 ms (218.7 MiB/s) | 0.78x |
| stress_hierarchy_heavy |  | owned 6.431 ms (194.7 MiB/s) | 5.445 ms (229.9 MiB/s) | 1.18x |
| stress_resource_heavy |  | owned 5.762 ms (162.8 MiB/s) | 4.008 ms (234.0 MiB/s) | 1.44x |
| stress_vertex_heavy |  | owned 11.590 ms (357.4 MiB/s) | 16.806 ms (246.5 MiB/s) | 0.69x |

### Write Benchmarks

| Case | Description | cityjson-json | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | to_vec 8.642 ms (652.0 MiB/s); to_vec_validated 8.636 ms (652.5 MiB/s) | 8.558 ms (658.4 MiB/s) | 1.01x |
| io_3dbag_cityjson_cluster_4x |  | to_vec 41.844 ms (466.8 MiB/s); to_vec_validated 41.962 ms (465.5 MiB/s) | 41.547 ms (470.1 MiB/s) | 1.01x |
| io_basisvoorziening_3d_cityjson |  | to_vec 250.494 ms (673.2 MiB/s); to_vec_validated 249.642 ms (675.5 MiB/s) | 309.005 ms (545.8 MiB/s) | 0.81x |
| stress_attribute_heavy_heterogenous |  | to_vec 2.523 ms (536.2 MiB/s); to_vec_validated 2.538 ms (533.1 MiB/s) | 2.384 ms (567.4 MiB/s) | 1.06x |
| stress_attribute_heavy_homogenous |  | to_vec 1.754 ms (584.6 MiB/s); to_vec_validated 1.764 ms (581.2 MiB/s) | 1.275 ms (804.1 MiB/s) | 1.38x |
| stress_boundary_heavy |  | to_vec 2.974 ms (462.4 MiB/s); to_vec_validated 2.968 ms (463.4 MiB/s) | 2.443 ms (562.9 MiB/s) | 1.22x |
| stress_geometry_heavy |  | to_vec 2.128 ms (487.8 MiB/s); to_vec_validated 2.125 ms (488.5 MiB/s) | 1.612 ms (643.9 MiB/s) | 1.32x |
| stress_hierarchy_heavy |  | to_vec 2.476 ms (505.2 MiB/s); to_vec_validated 2.482 ms (504.1 MiB/s) | 1.777 ms (704.0 MiB/s) | 1.39x |
| stress_resource_heavy |  | to_vec 1.986 ms (472.1 MiB/s); to_vec_validated 1.989 ms (471.4 MiB/s) | 1.638 ms (572.5 MiB/s) | 1.21x |
| stress_vertex_heavy |  | to_vec 9.089 ms (455.6 MiB/s); to_vec_validated 9.102 ms (455.0 MiB/s) | 8.130 ms (509.4 MiB/s) | 1.12x |
