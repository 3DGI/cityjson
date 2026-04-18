# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-arrow | cityjson-json/owned | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | stream_read 9.338 ms (603.7 MiB/s) | 29.727 ms (192.8 MiB/s) | 3.18x |
| io_3dbag_cityjson_cluster_4x |  | stream_read 37.610 ms (538.5 MiB/s) | 107.936 ms (185.9 MiB/s) | 2.87x |
| io_basisvoorziening_3d_cityjson |  | stream_read 281.578 ms (611.0 MiB/s) | 593.242 ms (286.1 MiB/s) | 2.11x |
| stress_attribute_heavy_heterogenous |  | stream_read 9.173 ms (258.3 MiB/s) | 8.896 ms (152.2 MiB/s) | 0.97x |
| stress_attribute_heavy_homogenous |  | stream_read 3.816 ms (182.7 MiB/s) | 6.147 ms (166.9 MiB/s) | 1.61x |
| stress_boundary_heavy |  | stream_read 638.748 us (3449.9 MiB/s) | 4.290 ms (320.7 MiB/s) | 6.72x |
| stress_geometry_heavy |  | stream_read 1.026 ms (1541.5 MiB/s) | 3.697 ms (281.0 MiB/s) | 3.60x |
| stress_hierarchy_heavy |  | stream_read 1.261 ms (1149.9 MiB/s) | 6.557 ms (190.9 MiB/s) | 5.20x |
| stress_resource_heavy |  | stream_read 1.534 ms (795.3 MiB/s) | 5.864 ms (160.0 MiB/s) | 3.82x |
| stress_vertex_heavy |  | stream_read 1.430 ms (4808.9 MiB/s) | 11.361 ms (364.6 MiB/s) | 7.95x |

### Write Benchmarks

| Case | Description | cityjson-arrow | cityjson-json/to_vec | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | stream_write 20.399 ms (276.3 MiB/s) | 8.827 ms (638.4 MiB/s) | 0.43x |
| io_3dbag_cityjson_cluster_4x |  | stream_write 100.966 ms (200.6 MiB/s) | 41.488 ms (470.8 MiB/s) | 0.41x |
| io_basisvoorziening_3d_cityjson |  | stream_write 739.427 ms (232.7 MiB/s) | 245.337 ms (687.4 MiB/s) | 0.33x |
| stress_attribute_heavy_heterogenous |  | stream_write 21.926 ms (108.1 MiB/s) | 2.555 ms (529.4 MiB/s) | 0.12x |
| stress_attribute_heavy_homogenous |  | stream_write 5.869 ms (118.8 MiB/s) | 1.842 ms (556.6 MiB/s) | 0.31x |
| stress_boundary_heavy |  | stream_write 753.796 us (2923.4 MiB/s) | 2.856 ms (481.4 MiB/s) | 3.79x |
| stress_geometry_heavy |  | stream_write 1.062 ms (1490.0 MiB/s) | 2.058 ms (504.4 MiB/s) | 1.94x |
| stress_hierarchy_heavy |  | stream_write 1.272 ms (1140.0 MiB/s) | 2.502 ms (500.0 MiB/s) | 1.97x |
| stress_resource_heavy |  | stream_write 1.062 ms (1148.6 MiB/s) | 2.091 ms (448.4 MiB/s) | 1.97x |
| stress_vertex_heavy |  | stream_write 2.582 ms (2662.2 MiB/s) | 8.833 ms (468.9 MiB/s) | 3.42x |
