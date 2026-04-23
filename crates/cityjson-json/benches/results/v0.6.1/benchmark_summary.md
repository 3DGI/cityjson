# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-json | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | owned 29.303 ms (195.6 MiB/s); borrowed 27.493 ms (208.4 MiB/s) | 19.124 ms (299.6 MiB/s) | 1.53x |
| io_3dbag_cityjson_cluster_4x |  | owned 106.774 ms (187.9 MiB/s); borrowed 100.506 ms (199.7 MiB/s) | 67.874 ms (295.6 MiB/s) | 1.57x |
| io_basisvoorziening_3d_cityjson |  | owned 598.255 ms (283.7 MiB/s); borrowed 537.743 ms (315.6 MiB/s) | 613.944 ms (276.4 MiB/s) | 0.97x |
| stress_attribute_heavy |  | owned 8.036 ms (181.4 MiB/s); borrowed 6.852 ms (212.8 MiB/s) | 6.605 ms (220.7 MiB/s) | 1.22x |
| stress_boundary_heavy |  | owned 4.270 ms (322.2 MiB/s); borrowed 4.278 ms (321.6 MiB/s) | 6.336 ms (217.1 MiB/s) | 0.67x |
| stress_geometry_heavy |  | owned 3.691 ms (281.4 MiB/s); borrowed 3.720 ms (279.3 MiB/s) | 4.678 ms (222.0 MiB/s) | 0.79x |
| stress_hierarchy_heavy |  | owned 6.407 ms (195.4 MiB/s); borrowed 6.356 ms (197.0 MiB/s) | 5.354 ms (233.8 MiB/s) | 1.20x |
| stress_resource_heavy |  | owned 5.733 ms (163.6 MiB/s); borrowed 5.718 ms (164.1 MiB/s) | 4.018 ms (233.5 MiB/s) | 1.43x |
| stress_vertex_heavy |  | owned 11.394 ms (363.5 MiB/s); borrowed 11.338 ms (365.3 MiB/s) | 16.830 ms (246.1 MiB/s) | 0.68x |

### Write Benchmarks

| Case | Description | cityjson-json | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | as_json_to_value 16.899 ms (333.5 MiB/s); to_string 7.998 ms (704.6 MiB/s); to_string_validated 8.010 ms (703.5 MiB/s) | 6.890 ms (817.9 MiB/s) | 1.16x |
| io_3dbag_cityjson_cluster_4x |  | as_json_to_value 63.012 ms (310.0 MiB/s); to_string 41.700 ms (468.4 MiB/s); to_string_validated 41.877 ms (466.4 MiB/s) | 35.023 ms (557.7 MiB/s) | 1.19x |
| io_basisvoorziening_3d_cityjson |  | as_json_to_value 434.568 ms (388.1 MiB/s); to_string 246.891 ms (683.1 MiB/s); to_string_validated 247.064 ms (682.6 MiB/s) | 249.611 ms (675.6 MiB/s) | 0.99x |
| stress_attribute_heavy |  | as_json_to_value 5.714 ms (255.1 MiB/s); to_string 2.072 ms (703.6 MiB/s); to_string_validated 2.058 ms (708.3 MiB/s) | 1.328 ms (1097.7 MiB/s) | 1.56x |
| stress_boundary_heavy |  | as_json_to_value 3.392 ms (405.5 MiB/s); to_string 2.806 ms (490.1 MiB/s); to_string_validated 2.813 ms (488.9 MiB/s) | 2.197 ms (625.9 MiB/s) | 1.28x |
| stress_geometry_heavy |  | as_json_to_value 3.005 ms (345.6 MiB/s); to_string 2.018 ms (514.5 MiB/s); to_string_validated 2.024 ms (513.0 MiB/s) | 1.512 ms (686.7 MiB/s) | 1.33x |
| stress_hierarchy_heavy |  | as_json_to_value 4.165 ms (300.4 MiB/s); to_string 2.308 ms (541.9 MiB/s); to_string_validated 2.304 ms (543.0 MiB/s) | 1.548 ms (808.1 MiB/s) | 1.49x |
| stress_resource_heavy |  | as_json_to_value 3.290 ms (285.0 MiB/s); to_string 1.847 ms (507.8 MiB/s); to_string_validated 1.843 ms (508.7 MiB/s) | 1.456 ms (643.9 MiB/s) | 1.27x |
| stress_vertex_heavy |  | as_json_to_value 9.284 ms (446.1 MiB/s); to_string 8.771 ms (472.2 MiB/s); to_string_validated 8.742 ms (473.7 MiB/s) | 7.184 ms (576.5 MiB/s) | 1.22x |
