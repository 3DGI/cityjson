# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | cityjson-json | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | owned 29.704 ms (192.9 MiB/s); borrowed 27.843 ms (205.8 MiB/s) | 18.584 ms (308.3 MiB/s) | 1.60x |
| io_3dbag_cityjson_cluster_4x |  | owned 107.437 ms (186.8 MiB/s); borrowed 102.258 ms (196.2 MiB/s) | 69.012 ms (290.8 MiB/s) | 1.56x |
| io_basisvoorziening_3d_cityjson |  | owned 600.842 ms (282.4 MiB/s); borrowed 541.350 ms (313.5 MiB/s) | 675.975 ms (251.1 MiB/s) | 0.89x |
| stress_appearance_and_validation |  | owned 179.886 us (185.5 MiB/s); borrowed 182.679 us (182.6 MiB/s) | 171.263 us (194.8 MiB/s) | 1.05x |
| stress_attribute_tree |  | owned 100.181 us (190.8 MiB/s); borrowed 80.945 us (236.2 MiB/s) | 72.354 us (264.2 MiB/s) | 1.38x |
| stress_composite_value |  | owned 115.138 us (218.0 MiB/s); borrowed 103.835 us (241.7 MiB/s) | 108.997 us (230.3 MiB/s) | 1.06x |
| stress_deep_boundary |  | owned 78.849 us (303.6 MiB/s); borrowed 78.707 us (304.1 MiB/s) | 117.677 us (203.4 MiB/s) | 0.67x |
| stress_geometry_flattening |  | owned 246.786 us (348.0 MiB/s); borrowed 234.742 us (365.9 MiB/s) | 407.791 us (210.6 MiB/s) | 0.61x |
| stress_relation_graph |  | owned 134.308 us (287.1 MiB/s); borrowed 131.637 us (293.0 MiB/s) | 157.823 us (244.3 MiB/s) | 0.85x |
| stress_vertex_transform |  | owned 60.403 us (333.9 MiB/s); borrowed 55.154 us (365.7 MiB/s) | 82.605 us (244.2 MiB/s) | 0.73x |

### Write Benchmarks

| Case | Description | cityjson-json | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | as_json_to_value 17.063 ms (330.3 MiB/s); to_string 9.126 ms (617.5 MiB/s); to_string_validated 8.657 ms (650.9 MiB/s) | 7.284 ms (773.6 MiB/s) | 1.25x |
| io_3dbag_cityjson_cluster_4x |  | as_json_to_value 64.120 ms (304.6 MiB/s); to_string 41.283 ms (473.1 MiB/s); to_string_validated 41.128 ms (474.9 MiB/s) | 33.556 ms (582.1 MiB/s) | 1.23x |
| io_basisvoorziening_3d_cityjson |  | as_json_to_value 439.825 ms (383.4 MiB/s); to_string 251.346 ms (670.9 MiB/s); to_string_validated 251.796 ms (669.8 MiB/s) | 254.301 ms (663.2 MiB/s) | 0.99x |
| stress_appearance_and_validation |  | as_json_to_value 78.430 us (425.2 MiB/s); to_string 52.881 us (630.6 MiB/s); to_string_validated 52.995 us (629.3 MiB/s) | 46.863 us (711.6 MiB/s) | 1.13x |
| stress_attribute_tree |  | as_json_to_value 53.579 us (356.4 MiB/s); to_string 18.683 us (1022.2 MiB/s); to_string_validated 19.107 us (999.5 MiB/s) | 15.556 us (1227.7 MiB/s) | 1.20x |
| stress_composite_value |  | as_json_to_value 65.359 us (383.8 MiB/s); to_string 29.254 us (857.6 MiB/s); to_string_validated 29.265 us (857.2 MiB/s) | 25.340 us (990.0 MiB/s) | 1.15x |
| stress_deep_boundary |  | as_json_to_value 49.227 us (485.8 MiB/s); to_string 32.287 us (740.6 MiB/s); to_string_validated 31.875 us (750.2 MiB/s) | 31.080 us (769.4 MiB/s) | 1.04x |
| stress_geometry_flattening |  | as_json_to_value 176.247 us (487.1 MiB/s); to_string 115.773 us (741.6 MiB/s); to_string_validated 116.005 us (740.1 MiB/s) | 108.555 us (790.9 MiB/s) | 1.07x |
| stress_relation_graph |  | as_json_to_value 88.144 us (437.2 MiB/s); to_string 51.916 us (742.3 MiB/s); to_string_validated 54.359 us (708.9 MiB/s) | 41.308 us (932.9 MiB/s) | 1.26x |
| stress_vertex_transform |  | as_json_to_value 38.862 us (518.6 MiB/s); to_string 26.240 us (768.1 MiB/s); to_string_validated 26.260 us (767.5 MiB/s) | 22.913 us (879.6 MiB/s) | 1.15x |
