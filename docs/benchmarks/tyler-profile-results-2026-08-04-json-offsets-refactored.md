# Tyler profiling results — refactored JSON offsets — 2026-08-04

## Summary

This candidate-only campaign measured the refactored JSON-offset vertex store
at commit `b8695134ffacea3521e897f95f0cc5e6a6af3327` against the same Groningen
182-tile corpus used by the previous JSON-offset report. The baseline was not
rerun. Shared harness commit: `bc23d7e`. Corpus identity:
`groningen-182-local-2026-08-04-json-offset-refactor`.

All 26 requested candidate runs completed: nine native timing runs, nine
perf-stat runs, four Heaptrack runs, three perf-record runs, and one Cachegrind
run. Each process used a transient cgroup with `MemoryMax=28G`,
`MemorySwapMax=0`, a 100 ms sampler, and a 3600-second runtime limit. There
were no cgroup OOM kills or cgroup swap-in/out events.

The refactor improved candidate-only native materialization time versus the
previous JSON-offset candidate at every worker count: about 19.0% faster at
one worker, 17.9% faster at four workers, and 15.0% faster at 24 workers.
Against the existing Tyler memory baseline, the refactored candidate remains 67.3%,
87.0%, and 74.2% slower at one, four, and 24 workers respectively. Process
RSS and cgroup peak are lower than baseline at one and four workers, while
both are slightly higher at 24 workers.

## Native candidate timing and memory

Values are medians over three full-corpus repetitions. Materialization time is
the candidate result's `elapsed_ns`; process RSS is the final profile
snapshot's process-lifetime `VmHWM`; cgroup peak is cgroup v2
`memory.peak`.

| Workers | Materialization | Process RSS | Cgroup peak |
|---:|---:|---:|---:|
| 1 | 199.88 s | 2.26 GiB | 7.43 GiB |
| 4 | 52.50 s | 5.52 GiB | 5.52 GiB |
| 24 | 20.85 s | 19.20 GiB | 21.59 GiB |

## Comparison with the existing Tyler memory baseline

The baseline values below are taken from
[tyler-profile-results-2026-08-03.md](tyler-profile-results-2026-08-03.md),
which was the baseline used by the earlier JSON-offset report. The baseline
was not rerun for this campaign. Materialization is the directly comparable
native stage; total_pipeline_elapsed_ns includes candidate setup but does not
execute the baseline extent/grid stages, so total time is not compared here.

| Workers | Baseline materialization | Refactored JSON-offset materialization | Delta | Delta % | Baseline process RSS | Refactored process RSS | Baseline cgroup peak | Refactored cgroup peak |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 119.51 s | 199.88 s | +80.37 s | +67.3% | 3.10 GiB | 2.26 GiB | 9.89 GiB | 7.43 GiB |
| 4 | 28.07 s | 52.50 s | +24.43 s | +87.0% | 7.01 GiB | 5.52 GiB | 7.52 GiB | 5.52 GiB |
| 24 | 11.96 s | 20.85 s | +8.89 s | +74.2% | 18.63 GiB | 19.20 GiB | 19.89 GiB | 21.59 GiB |

The refactored candidate therefore reduces process RSS by about 27.1% at one
worker and 21.3% at four workers, but increases it by about 3.1% at 24
workers. Cgroup peak falls by about 24.9% and 26.6% at one and four workers,
then rises by about 8.5% at 24 workers. The memory baseline retained 182, 411,
and 1,039 cached source copies and reported 3.60, 8.39, and 22.44 GiB of
vertex capacity at one, four, and 24 workers; the JSON-offset candidate
reports zero retained decoded bytes instead of a vertex-cache capacity.

Compared with the previous candidate report:

| Workers | Previous time | Refactored time | Delta | Delta % |
|---:|---:|---:|---:|---:|
| 1 | 246.63 s | 199.88 s | −46.75 s | −19.0% |
| 4 | 63.95 s | 52.50 s | −11.45 s | −17.9% |
| 24 | 24.52 s | 20.85 s | −3.67 s | −15.0% |

The full-corpus package count was 707,239 and the model digest was stable:
`sha256:7dc19f3033c0c7b8198114f5de9fec177baaba32a14b5aa5265b6d0de11de38c`.

## Candidate telemetry and storage

Full-corpus telemetry was identical across the native and perf-stat runs:

- requested vertices: 125,352,085;
- unique and returned vertices: 118,325,159;
- persistent SQLite bytes read: 856,194,164;
- source JSON bytes read: 6,130,907,078;
- source file opens: 182;
- metadata loads: 1;
- touched units: 15,476; and
- retained decoded bytes: 0.

| Prepared corpus | Source bytes | Packages | JSON-offset sidecar | Candidate payload |
|---:|---:|---:|---:|---:|
| 182 tiles | 8,996,367,140 | 707,239 | 1,178,931,200 | 446,880,960 |
| 24 tiles | 878,634,981 | 71,140 | 116,981,760 | 43,519,368 |
| 1 tile | 20,686,295 | 1,400 | 2,637,824 | 1,072,480 |

The sidecars were built before measurement, opened read-only for validation,
and had marker schema version 3 with `schema_state` version 2 and
`needs_reindex=0`.

## Perf stat

Median full-corpus counters over three repetitions:

| Workers | Task-clock | Cycles | Instructions | Branch misses | Cache misses | Context switches |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 195,070 ms | 1.078e12 | 3.622e12 | 3.203e9 | 4.390e9 | 116,300 |
| 4 | 210,642 ms | 1.140e12 | 3.634e12 | 3.200e9 | 4.413e9 | 75,615 |
| 24 | 486,454 ms | 2.430e12 | 3.818e12 | 3.645e9 | 7.821e9 | 5,328,241 |

Complete counters, including page faults, major faults, cache references, and
branches, are retained in each `perf-stat-w*-r*-t182` run directory.

## Heaptrack and reduced profilers

The full-corpus worker-1 Heaptrack run completed. Its projected worker-4
cgroup peak was 32,050,888,704 bytes, above the 90% safety threshold of
25.2 GiB, so the prescribed 24-tile fallback was used for workers 1, 4, and
24. Reduced Heaptrack peaks are not compared with full-corpus native peaks.

| Tiles | Workers | Peak requested heap | Package reconstruction | Other | SQLite | Process RSS | Cgroup peak |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 182 | 1 | 1.57 GiB | 1.23 GiB | 0.31 GiB | 24.7 MiB | 2.28 GiB | 7.46 GiB |
| 24 | 1 | 0.56 GiB | 0.46 GiB | 0.09 GiB | 8.8 MiB | 0.68 GiB | 1.73 GiB |
| 24 | 4 | 1.02 GiB | 0.67 GiB | 0.33 GiB | 15.6 MiB | 2.17 GiB | 2.37 GiB |
| 24 | 24 | 4.61 GiB | 3.82 GiB | 0.73 GiB | 67.2 MiB | 6.86 GiB | 7.30 GiB |

Perf record completed for workers 1, 4, and 24 on 24 tiles. Cachegrind
completed for worker 1 on one tile.

## Provenance and raw artifacts

The profiling-only detached overlay was based on the clean refactored commit
and contained only the optional memory-snapshot hook needed by the supervisor.
Its working-tree diff SHA-256 was
`a1ac5c5ce58cbfdc8b926cb8e682f4274fac0b48c15e13b6a50048300db82118`.

Campaign manifest:

`target/profiling/cityjson-index-vertex-store-refactored/campaigns/20260804T094921Z-json-offsets-6c40a563/campaign.json`

Raw artifacts:

`target/profiling/cityjson-index-vertex-store-refactored/campaigns/20260804T094921Z-json-offsets-6c40a563/runs/`

The previous candidate-only report remains
[tyler-profile-results-2026-08-04-json-offsets.md](tyler-profile-results-2026-08-04-json-offsets.md),
and the existing Tyler memory baseline is documented in
[tyler-profile-results-2026-08-03.md](tyler-profile-results-2026-08-03.md).
Neither the baseline nor the previous candidate was rerun for this campaign.
