# Benchmark Plot Summary

- Description: `cityarrow refactor 9f3d51e`
- Timestamp: `2026-04-02T12:45:56Z`
- Mode: `full`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 229.5 MiB/s | 0.75x (172.4 MiB/s) | 0.67x (153.8 MiB/s) | 0.87x (200.4 MiB/s) | 0.88x (201.7 MiB/s) |
| 3DBAG cluster 4x | 193.3 MiB/s | 0.78x (149.8 MiB/s) | 0.71x (137.6 MiB/s) | 0.90x (174.0 MiB/s) | 0.91x (175.5 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 736.8 MiB/s | 0.88x (646.3 MiB/s) | 0.89x (656.7 MiB/s) | 0.13x (96.2 MiB/s) | 0.13x (98.6 MiB/s) |
| 3DBAG cluster 4x | 560.4 MiB/s | 0.85x (474.8 MiB/s) | 0.86x (481.0 MiB/s) | 0.17x (95.1 MiB/s) | 0.17x (95.8 MiB/s) |
