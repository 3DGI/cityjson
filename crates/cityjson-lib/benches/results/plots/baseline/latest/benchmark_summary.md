# Benchmark Plot Summary

- Description: `baseline`
- Timestamp: `2026-04-01T21:21:02Z`
- Mode: `full`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 225.0 MiB/s | 0.76x (171.2 MiB/s) | 0.67x (151.0 MiB/s) | 0.68x (153.0 MiB/s) | 0.62x (140.5 MiB/s) |
| 3DBAG cluster 4x | 186.7 MiB/s | 0.81x (151.2 MiB/s) | 0.74x (138.0 MiB/s) | 0.70x (131.3 MiB/s) | 0.67x (125.0 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 695.2 MiB/s | 0.90x (623.9 MiB/s) | 0.85x (594.3 MiB/s) | 0.15x (100.9 MiB/s) | 0.12x (85.7 MiB/s) |
| 3DBAG cluster 4x | 552.4 MiB/s | 0.87x (478.7 MiB/s) | 0.86x (473.9 MiB/s) | 0.15x (85.3 MiB/s) | 0.13x (73.8 MiB/s) |
