# Benchmark Summary

Generated from Criterion results.

### Read Benchmarks

| Case | Description | serde_cityjson | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | owned 29.290 ms (195.6 MiB/s); borrowed 27.761 ms (206.4 MiB/s) | 17.865 ms (320.7 MiB/s) | 1.64x |
| stress_appearance_and_validation |  | owned 4.443 us (322.0 MiB/s); borrowed 4.445 us (321.8 MiB/s) | 4.065 us (351.9 MiB/s) | 1.09x |
| stress_attribute_tree |  | owned 6.827 us (274.8 MiB/s); borrowed 6.639 us (282.5 MiB/s) | 6.033 us (310.9 MiB/s) | 1.13x |
| stress_composite_value |  | owned 5.857 us (329.2 MiB/s); borrowed 5.432 us (355.0 MiB/s) | 5.734 us (336.3 MiB/s) | 1.02x |
| stress_deep_boundary |  | owned 5.153 us (338.9 MiB/s); borrowed 4.895 us (356.8 MiB/s) | 5.269 us (331.4 MiB/s) | 0.98x |
| stress_geometry_flattening |  | owned 4.357 us (323.8 MiB/s); borrowed 4.180 us (337.5 MiB/s) | 4.284 us (329.2 MiB/s) | 1.02x |
| stress_relation_graph |  | owned 5.202 us (310.3 MiB/s); borrowed 4.766 us (338.8 MiB/s) | 4.908 us (328.9 MiB/s) | 1.06x |
| stress_vertex_transform |  | owned 5.204 us (310.6 MiB/s); borrowed 4.861 us (332.5 MiB/s) | 4.890 us (330.6 MiB/s) | 1.06x |

### Write Benchmarks

| Case | Description | serde_cityjson | serde_json::to_string | Factor |
| --- | --- | --- | --- | --- |
| io_3dbag_cityjson |  | as_json_to_value 16.134 ms (349.3 MiB/s); to_string 8.087 ms (696.8 MiB/s); to_string_validated 8.262 ms (682.1 MiB/s) | 7.433 ms (758.1 MiB/s) | 1.09x |
| stress_appearance_and_validation |  | as_json_to_value 3.029 us (471.9 MiB/s); to_string 1.697 us (842.5 MiB/s); to_string_validated 1.688 us (847.1 MiB/s) | 1.371 us (1043.0 MiB/s) | 1.24x |
| stress_attribute_tree |  | as_json_to_value 4.377 us (428.6 MiB/s); to_string 2.191 us (856.1 MiB/s); to_string_validated 2.208 us (849.6 MiB/s) | 2.086 us (899.3 MiB/s) | 1.05x |
| stress_composite_value |  | as_json_to_value 4.317 us (446.6 MiB/s); to_string 2.126 us (907.2 MiB/s); to_string_validated 2.128 us (906.1 MiB/s) | 2.038 us (946.4 MiB/s) | 1.04x |
| stress_deep_boundary |  | as_json_to_value 4.062 us (429.8 MiB/s); to_string 2.119 us (824.0 MiB/s); to_string_validated 2.093 us (834.2 MiB/s) | 2.066 us (845.3 MiB/s) | 1.03x |
| stress_geometry_flattening |  | as_json_to_value 3.261 us (432.5 MiB/s); to_string 1.576 us (894.7 MiB/s); to_string_validated 1.569 us (899.2 MiB/s) | 1.441 us (978.8 MiB/s) | 1.09x |
| stress_relation_graph |  | as_json_to_value 3.696 us (436.8 MiB/s); to_string 1.797 us (898.4 MiB/s); to_string_validated 1.790 us (901.9 MiB/s) | 1.630 us (990.6 MiB/s) | 1.10x |
| stress_vertex_transform |  | as_json_to_value 3.680 us (439.3 MiB/s); to_string 1.749 us (924.0 MiB/s); to_string_validated 1.752 us (922.6 MiB/s) | 1.683 us (960.5 MiB/s) | 1.04x |
