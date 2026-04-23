# Benchmark Plot Summary

- Description: `cityarrow v3 workstation`
- Timestamp: `2026-04-04T19:37:31Z`
- Mode: `full`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 238.9 MiB/s | 0.74x (175.7 MiB/s) | 0.66x (157.5 MiB/s) | 0.93x (221.7 MiB/s) | 0.92x (220.6 MiB/s) |
| 3DBAG cluster 4x | 195.3 MiB/s | 0.79x (154.3 MiB/s) | 0.72x (140.4 MiB/s) | 1.00x (195.6 MiB/s) | 1.02x (198.5 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 772.4 MiB/s | 0.91x (703.1 MiB/s) | 0.91x (705.0 MiB/s) | 0.15x (119.1 MiB/s) | 0.16x (124.3 MiB/s) |
| 3DBAG cluster 4x | 565.1 MiB/s | 0.88x (498.5 MiB/s) | 0.87x (490.4 MiB/s) | 0.20x (114.9 MiB/s) | 0.20x (114.4 MiB/s) |
