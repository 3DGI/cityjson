# Benchmark Plot Summary

- Description: `cityarrow v3 laptop`
- Timestamp: `2026-04-03T19:53:00Z`
- Mode: `all`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cjlib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 168.0 MiB/s | 0.80x (133.8 MiB/s) | 0.73x (122.9 MiB/s) | 1.05x (176.2 MiB/s) | 1.05x (176.1 MiB/s) |
| 3DBAG cluster 4x | 137.0 MiB/s | 0.90x (123.6 MiB/s) | 0.83x (114.0 MiB/s) | 1.22x (167.6 MiB/s) | 1.23x (168.5 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cjlib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 648.1 MiB/s | 0.91x (587.6 MiB/s) | 0.91x (588.4 MiB/s) | 0.20x (131.2 MiB/s) | 0.20x (131.0 MiB/s) |
| 3DBAG cluster 4x | 601.8 MiB/s | 0.90x (540.4 MiB/s) | 0.89x (536.2 MiB/s) | 0.18x (105.3 MiB/s) | 0.18x (105.6 MiB/s) |
