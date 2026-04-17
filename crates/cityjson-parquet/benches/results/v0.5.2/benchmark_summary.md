# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-parquet/read_file | cityjson-arrow/stream_read | cityjson-json/owned | Factor |
| --- | --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | 9.519 ms (593.6 MiB/s) | 9.521 ms (592.0 MiB/s) | 29.620 ms (193.5 MiB/s) | 0.32x |
| io_3dbag_cityjson_cluster_4x |  | 36.873 ms (549.6 MiB/s) | 38.330 ms (528.4 MiB/s) | 108.329 ms (185.2 MiB/s) | 0.34x |
| io_basisvoorziening_3d_cityjson |  | 266.138 ms (646.6 MiB/s) | 273.519 ms (629.0 MiB/s) | 602.830 ms (281.5 MiB/s) | 0.44x |
| stress_attribute_heavy |  | 4.787 ms (207.0 MiB/s) | 5.383 ms (181.9 MiB/s) | 8.444 ms (172.7 MiB/s) | 0.57x |
| stress_boundary_heavy |  | 672.605 us (3282.8 MiB/s) | 641.930 us (3432.8 MiB/s) | 4.272 ms (322.1 MiB/s) | 0.16x |
| stress_geometry_heavy |  | 1.029 ms (1542.2 MiB/s) | 1.023 ms (1547.0 MiB/s) | 3.692 ms (281.4 MiB/s) | 0.28x |
| stress_hierarchy_heavy |  | 1.301 ms (1119.4 MiB/s) | 1.258 ms (1152.7 MiB/s) | 6.453 ms (194.0 MiB/s) | 0.20x |
| stress_resource_heavy |  | 1.581 ms (777.9 MiB/s) | 1.539 ms (792.9 MiB/s) | 5.794 ms (161.9 MiB/s) | 0.27x |
| stress_vertex_heavy |  | 1.567 ms (4391.0 MiB/s) | 1.503 ms (4572.5 MiB/s) | 11.553 ms (358.5 MiB/s) | 0.14x |

### Write Benchmarks

| Case | Description | cityjson-parquet/write_file | cityjson-arrow/stream_write | cityjson-json/to_vec | Factor |
| --- | --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | 23.775 ms (237.7 MiB/s) | 19.925 ms (282.9 MiB/s) | 9.515 ms (592.2 MiB/s) | 2.50x |
| io_3dbag_cityjson_cluster_4x |  | 116.738 ms (173.6 MiB/s) | 100.852 ms (200.8 MiB/s) | 44.148 ms (442.4 MiB/s) | 2.64x |
| io_basisvoorziening_3d_cityjson |  | 871.102 ms (197.5 MiB/s) | 771.883 ms (222.9 MiB/s) | 256.043 ms (658.6 MiB/s) | 3.40x |
| stress_attribute_heavy |  | 12.116 ms (81.8 MiB/s) | 11.596 ms (84.4 MiB/s) | 2.165 ms (673.3 MiB/s) | 5.60x |
| stress_boundary_heavy |  | 4.114 ms (536.7 MiB/s) | 758.194 us (2906.4 MiB/s) | 2.905 ms (473.4 MiB/s) | 1.42x |
| stress_geometry_heavy |  | 3.119 ms (508.7 MiB/s) | 1.008 ms (1569.5 MiB/s) | 2.105 ms (493.2 MiB/s) | 1.48x |
| stress_hierarchy_heavy |  | 2.872 ms (507.1 MiB/s) | 1.157 ms (1254.0 MiB/s) | 2.452 ms (510.2 MiB/s) | 1.17x |
| stress_resource_heavy |  | 3.342 ms (368.1 MiB/s) | 960.835 us (1270.1 MiB/s) | 1.889 ms (496.3 MiB/s) | 1.77x |
| stress_vertex_heavy |  | 13.704 ms (502.0 MiB/s) | 2.073 ms (3315.6 MiB/s) | 9.132 ms (453.5 MiB/s) | 1.50x |
