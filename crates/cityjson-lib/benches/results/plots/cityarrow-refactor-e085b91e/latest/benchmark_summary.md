# Benchmark Plot Summary

- Description: `cityarrow refactor e085b91e`
- Timestamp: `2026-04-02T09:08:03Z`
- Mode: `full`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 239.7 MiB/s | 0.74x (176.6 MiB/s) | 0.66x (157.3 MiB/s) | 0.82x (197.4 MiB/s) | 0.82x (196.9 MiB/s) |
| 3DBAG cluster 4x | 197.1 MiB/s | 0.78x (154.0 MiB/s) | 0.71x (140.4 MiB/s) | 0.85x (168.5 MiB/s) | 0.86x (169.8 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 816.2 MiB/s | 0.86x (700.8 MiB/s) | 0.85x (697.7 MiB/s) | 0.15x (120.8 MiB/s) | 0.15x (122.4 MiB/s) |
| 3DBAG cluster 4x | 559.3 MiB/s | 0.89x (499.5 MiB/s) | 0.89x (495.8 MiB/s) | 0.18x (101.8 MiB/s) | 0.18x (101.7 MiB/s) |
