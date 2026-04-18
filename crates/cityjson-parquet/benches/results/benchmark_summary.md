# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-parquet/read_file | cityjson-arrow/stream_read | cityjson-json/owned | Factor |
| --- | --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | 9.469 ms (596.7 MiB/s) | 9.421 ms (598.3 MiB/s) | 30.063 ms (190.6 MiB/s) | 3.17x |
| io_3dbag_cityjson_cluster_4x |  | 36.981 ms (548.0 MiB/s) | 38.201 ms (530.2 MiB/s) | 109.556 ms (183.2 MiB/s) | 2.96x |
| io_basisvoorziening_3d_cityjson |  | 285.922 ms (601.8 MiB/s) | 281.189 ms (611.9 MiB/s) | 600.269 ms (282.7 MiB/s) | 2.10x |
| stress_attribute_heavy_heterogenous |  | 9.510 ms (252.3 MiB/s) | 9.725 ms (243.7 MiB/s) | 8.915 ms (151.8 MiB/s) | 0.94x |
| stress_attribute_heavy_homogenous |  | 3.943 ms (179.9 MiB/s) | 3.997 ms (174.4 MiB/s) | 6.319 ms (162.3 MiB/s) | 1.60x |
| stress_boundary_heavy |  | 720.340 us (3065.3 MiB/s) | 650.337 us (3388.4 MiB/s) | 4.331 ms (317.7 MiB/s) | 6.01x |
| stress_geometry_heavy |  | 1.046 ms (1517.5 MiB/s) | 1.039 ms (1522.6 MiB/s) | 3.822 ms (271.8 MiB/s) | 3.66x |
| stress_hierarchy_heavy |  | 1.325 ms (1099.5 MiB/s) | 1.306 ms (1110.3 MiB/s) | 6.627 ms (188.9 MiB/s) | 5.00x |
| stress_resource_heavy |  | 1.611 ms (763.3 MiB/s) | 1.586 ms (769.3 MiB/s) | 6.071 ms (154.5 MiB/s) | 3.77x |
| stress_vertex_heavy |  | 1.573 ms (4374.5 MiB/s) | 1.550 ms (4435.5 MiB/s) | 11.582 ms (357.6 MiB/s) | 7.37x |

### Write Benchmarks

| Case | Description | cityjson-parquet/write_file | cityjson-arrow/stream_write | cityjson-json/to_vec | Factor |
| --- | --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | 22.927 ms (246.5 MiB/s) | 22.026 ms (255.9 MiB/s) | 13.536 ms (416.3 MiB/s) | 0.59x |
| io_3dbag_cityjson_cluster_4x |  | 128.362 ms (157.9 MiB/s) | 116.513 ms (173.8 MiB/s) | 49.370 ms (395.6 MiB/s) | 0.38x |
| io_basisvoorziening_3d_cityjson |  | 1.043 s (165.0 MiB/s) | 885.699 ms (194.3 MiB/s) | 266.128 ms (633.7 MiB/s) | 0.26x |
| stress_attribute_heavy_heterogenous |  | 26.649 ms (90.0 MiB/s) | 22.076 ms (107.3 MiB/s) | 2.525 ms (535.9 MiB/s) | 0.09x |
| stress_attribute_heavy_homogenous |  | 6.323 ms (112.2 MiB/s) | 7.672 ms (90.9 MiB/s) | 1.885 ms (544.0 MiB/s) | 0.30x |
| stress_boundary_heavy |  | 3.372 ms (654.8 MiB/s) | 803.390 us (2742.9 MiB/s) | 2.909 ms (472.7 MiB/s) | 0.86x |
| stress_geometry_heavy |  | 2.558 ms (620.2 MiB/s) | 1.024 ms (1544.9 MiB/s) | 2.116 ms (490.7 MiB/s) | 0.83x |
| stress_hierarchy_heavy |  | 2.897 ms (502.8 MiB/s) | 1.159 ms (1251.4 MiB/s) | 2.448 ms (511.1 MiB/s) | 0.85x |
| stress_resource_heavy |  | 2.737 ms (449.3 MiB/s) | 976.575 us (1249.7 MiB/s) | 1.908 ms (491.5 MiB/s) | 0.70x |
| stress_vertex_heavy |  | 10.865 ms (633.2 MiB/s) | 2.080 ms (3304.7 MiB/s) | 9.094 ms (455.4 MiB/s) | 0.84x |
