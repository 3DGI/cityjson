# Benchmark Plot Summary

- Description: `cityarrow optimize decode,encode`
- Timestamp: `2026-04-04T20:42:15Z`
- Mode: `full`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 236.8 MiB/s | 0.74x (175.7 MiB/s) | 0.66x (157.1 MiB/s) | 0.94x (222.8 MiB/s) | 0.95x (225.4 MiB/s) |
| 3DBAG cluster 4x | 194.5 MiB/s | 0.78x (151.8 MiB/s) | 0.71x (138.3 MiB/s) | 1.01x (197.1 MiB/s) | 1.03x (199.4 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cityjson_lib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 806.3 MiB/s | 0.85x (687.3 MiB/s) | 0.85x (684.0 MiB/s) | 0.15x (121.8 MiB/s) | 0.13x (106.2 MiB/s) |
| 3DBAG cluster 4x | 562.4 MiB/s | 0.89x (500.7 MiB/s) | 0.89x (498.3 MiB/s) | 0.21x (120.5 MiB/s) | 0.21x (119.1 MiB/s) |
