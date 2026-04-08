# Benchmark Plot Summary

- Description: `cityarrow optimize v0.5.1`
- Timestamp: `2026-04-08T12:12:16Z`
- Mode: `full`
- Metric: relative speed using a common logical dataset-size denominator relative to `serde_json::Value` (`>1` means faster)

## Read

| Case | Baseline | serde_cityjson | cjlib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 234.9 MiB/s | 0.73x (171.6 MiB/s) | 0.65x (153.6 MiB/s) | 1.62x (380.3 MiB/s) | 1.57x (368.2 MiB/s) |
| 3DBAG cluster 4x | 194.3 MiB/s | 0.78x (152.2 MiB/s) | 0.72x (139.2 MiB/s) | 1.60x (311.3 MiB/s) | 1.59x (308.4 MiB/s) |

## Write

| Case | Baseline | serde_cityjson | cjlib::json | cityarrow | cityparquet |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 794.7 MiB/s | 0.84x (668.4 MiB/s) | 0.85x (678.5 MiB/s) | 0.39x (309.9 MiB/s) | 0.39x (310.2 MiB/s) |
| 3DBAG cluster 4x | 560.1 MiB/s | 0.89x (497.4 MiB/s) | 0.88x (491.2 MiB/s) | 0.34x (189.7 MiB/s) | 0.35x (194.2 MiB/s) |
