# Benchmark Plot Summary

- Description: `cityarrow optimize v0.5.0`
- Timestamp: `2026-04-07T22:36:17Z`
- Mode: `all`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cjlib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 228.9 MiB/s | 0.74x (169.7 MiB/s) | 0.66x (150.0 MiB/s) | 1.59x (364.4 MiB/s) | 1.57x (358.9 MiB/s) |
| 3DBAG cluster 4x | 192.0 MiB/s | 0.78x (150.5 MiB/s) | 0.71x (136.5 MiB/s) | 1.58x (303.6 MiB/s) | 1.60x (306.8 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cjlib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 726.2 MiB/s | 0.87x (628.7 MiB/s) | 0.86x (623.5 MiB/s) | 0.22x (159.8 MiB/s) | 0.18x (128.1 MiB/s) |
| 3DBAG cluster 4x | 546.6 MiB/s | 0.86x (470.9 MiB/s) | 0.87x (475.6 MiB/s) | 0.30x (162.0 MiB/s) | 0.29x (160.2 MiB/s) |
