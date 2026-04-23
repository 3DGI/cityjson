# Benchmark Plot Summary

- Description: `cityarrow v2alpha2 conversion cleanup`
- Timestamp: `2026-04-02T13:46:48Z`
- Mode: `full`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 228.7 MiB/s | 0.75x (171.2 MiB/s) | 0.67x (152.4 MiB/s) | 1.11x (254.0 MiB/s) | 1.10x (252.6 MiB/s) |
| 3DBAG cluster 4x | 193.4 MiB/s | 0.78x (151.6 MiB/s) | 0.71x (137.9 MiB/s) | 1.11x (214.7 MiB/s) | 1.13x (217.6 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 751.3 MiB/s | 0.86x (646.7 MiB/s) | 0.84x (632.2 MiB/s) | 0.14x (101.7 MiB/s) | 0.15x (109.9 MiB/s) |
| 3DBAG cluster 4x | 560.1 MiB/s | 0.87x (486.8 MiB/s) | 0.84x (469.9 MiB/s) | 0.19x (108.0 MiB/s) | 0.19x (107.6 MiB/s) |
