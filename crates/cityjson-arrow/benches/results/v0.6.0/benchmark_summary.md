# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-arrow | cityjson-json/owned | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | stream_read 9.589 ms (587.9 MiB/s) | 29.943 ms (191.4 MiB/s) | 0.32x |
| io_3dbag_cityjson_cluster_4x |  | stream_read 38.603 ms (524.7 MiB/s) | 108.266 ms (185.3 MiB/s) | 0.36x |
| io_basisvoorziening_3d_cityjson |  | stream_read 275.830 ms (623.8 MiB/s) | 604.554 ms (280.7 MiB/s) | 0.46x |
| stress_attribute_heavy |  | stream_read 4.838 ms (202.4 MiB/s) | 8.528 ms (171.0 MiB/s) | 0.57x |
| stress_boundary_heavy |  | stream_read 648.233 us (3399.4 MiB/s) | 4.335 ms (317.4 MiB/s) | 0.15x |
| stress_geometry_heavy |  | stream_read 1.026 ms (1541.7 MiB/s) | 3.759 ms (276.4 MiB/s) | 0.27x |
| stress_hierarchy_heavy |  | stream_read 1.280 ms (1133.4 MiB/s) | 6.486 ms (193.0 MiB/s) | 0.20x |
| stress_resource_heavy |  | stream_read 1.555 ms (784.8 MiB/s) | 5.891 ms (159.2 MiB/s) | 0.26x |
| stress_vertex_heavy |  | stream_read 1.550 ms (4434.6 MiB/s) | 11.491 ms (360.5 MiB/s) | 0.13x |

### Write Benchmarks

| Case | Description | cityjson-arrow | cityjson-json/to_vec | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | stream_write 25.566 ms (220.5 MiB/s) | 9.056 ms (622.2 MiB/s) | 2.82x |
| io_3dbag_cityjson_cluster_4x |  | stream_write 104.668 ms (193.5 MiB/s) | 44.507 ms (438.8 MiB/s) | 2.35x |
| io_basisvoorziening_3d_cityjson |  | stream_write 766.450 ms (224.5 MiB/s) | 254.514 ms (662.6 MiB/s) | 3.01x |
| stress_attribute_heavy |  | stream_write 12.182 ms (80.4 MiB/s) | 2.262 ms (644.5 MiB/s) | 5.39x |
| stress_boundary_heavy |  | stream_write 787.914 us (2796.8 MiB/s) | 2.868 ms (479.6 MiB/s) | 0.27x |
| stress_geometry_heavy |  | stream_write 1.067 ms (1483.4 MiB/s) | 2.081 ms (499.0 MiB/s) | 0.51x |
| stress_hierarchy_heavy |  | stream_write 1.274 ms (1138.6 MiB/s) | 2.608 ms (479.7 MiB/s) | 0.49x |
| stress_resource_heavy |  | stream_write 1.069 ms (1141.7 MiB/s) | 2.139 ms (438.4 MiB/s) | 0.50x |
| stress_vertex_heavy |  | stream_write 2.677 ms (2567.7 MiB/s) | 8.834 ms (468.8 MiB/s) | 0.30x |
