# Tyler profiling results - JSON offsets - 2026-08-04

## Summary

This campaign measured the JSON-offset candidate's actual
'vertex-store-bakeoff tyler-materialization' executable against the Tyler
memory baseline from [2026-08-03](tyler-profile-results-2026-08-03.md).

Candidate commit: '1f33165e5481455074ff007f7dff4b8d948e4287'. Shared harness:
'bc23d7e'. Corpus: the same 182-tile Groningen corpus with 707,239 packages.
Every measured process used a transient cgroup with MemoryMax=28G,
MemorySwapMax=0, a 100 ms sampler, and a 3600-second runtime limit.

All 26 requested runs completed. Authoritative memory.events.oom_kill was zero
in every run, cgroup pswpin/pswpout were zero in every run, the full-corpus
package count was 707,239, and the full-corpus model digest was stable across
worker counts, repetitions, and full-corpus profiler runs.

The direct result is unfavorable for materialization time: JSON offsets took
about 2.05x-2.28x the baseline materialization time. Process peak RSS was
lower at one and four workers, but 8.1% higher at 24 workers; cgroup peaks
were lower at one and four workers and 4.7% higher at 24 workers.

## Direct native comparison

The authoritative candidate timing is TylerResult.result.elapsed_ns. Process
RSS is the final bake-off profile output's process-lifetime VmHWM; cgroup peak
is cgroup v2 memory.peak.

| Workers | Baseline materialization | JSON-offset materialization | Delta | Delta % | Baseline process RSS | JSON-offset process RSS | Baseline cgroup peak | JSON-offset cgroup peak |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 119.51 s | 246.63 s | +127.12 s | +106.4% | 3.10 GiB | 2.32 GiB | 9.89 GiB | 8.95 GiB |
| 4 | 28.07 s | 63.95 s | +35.88 s | +127.8% | 7.01 GiB | 5.63 GiB | 7.52 GiB | 5.64 GiB |
| 24 | 11.96 s | 24.52 s | +12.56 s | +105.0% | 18.63 GiB | 20.13 GiB | 19.89 GiB | 20.82 GiB |

Candidate native materialization ranges across three repetitions:

| Workers | Materialization range | Cgroup peak range |
|---:|---:|---:|
| 1 | 245.396-247.560 s | 7.07-9.20 GiB |
| 4 | 63.949-63.955 s | 5.59-5.64 GiB |
| 24 | 24.469-24.554 s | 20.73-21.29 GiB |

Candidate total_pipeline_elapsed_ns medians were 258.93 s, 72.22 s, and
37.51 s for 1, 4, and 24 workers. This is setup plus candidate
materialization only: it does not execute the baseline extent/grid stages and
must not be compared with baseline end-to-end Tyler time.

## Candidate telemetry and storage

Full-corpus telemetry was identical across native and perf stat runs:

- package count: 707,239;
- model digest: 'sha256:7dc19f3033c0c7b8198114f5de9fec177baaba32a14b5aa5265b6d0de11de38c';
- requested vertices: 125,352,085;
- unique and returned vertices: 118,325,159;
- persistent SQLite bytes read: 856,194,164;
- source JSON bytes read: 3,102,905,763;
- touched units: 15,476;
- retained decoded bytes: 0.

| Prepared corpus | Source bytes | Packages | JSON-offset sidecar | Candidate payload |
|---:|---:|---:|---:|---:|
| 182 tiles | 8,996,367,140 | 707,239 | 1,154,154,496 | 446,880,960 |
| 24 tiles | 878,634,981 | 71,140 | 114,188,288 | 43,519,368 |
| 1 tile | 20,686,295 | 1,400 | 2,572,288 | 1,072,480 |

All sidecars were built before measurement, opened read-only for validation, had
the json-offsets marker at schema version 3, and had schema_state version 2
with needs_reindex=0. The full prepared source byte count matches baseline.
The baseline sidecar was 698,347,520 bytes; the JSON-offset sidecar was
1,154,154,496 bytes (+65.3%). Candidate payload size has no baseline cache
equivalent.

Cached source copies, vertex-vector capacity, and cache-drop RSS are not
applicable to JSON offsets. Candidate retention evidence is
retained_decoded_bytes=0, the process profile output, and cgroup measurements.

## Perf stat

Median full-corpus counters over three repetitions:

| Workers | Task-clock | Cycles | Instructions | Branch misses | Cache misses | Context switches |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 239,179 ms | 1.325e12 | 4.074e12 | 8.928e9 | 5.161e9 | 194,027 |
| 4 | 254,763 ms | 1.382e12 | 4.090e12 | 8.923e9 | 5.097e9 | 71,924 |
| 24 | 532,368 ms | 2.657e12 | 4.317e12 | 9.460e9 | 8.499e9 | 5,350,164 |

Complete CSVs, including page faults, major faults, cache references, and
branches, are in each perf-stat-w*-r*-t182 run directory.

## Heaptrack and reduced profilers

Heaptrack measured full corpus at worker 1. Its conservative four-worker
projection was 38,185,664,512 bytes (35.57 GiB), above the 90% safety
threshold of 25.2 GiB, so worker 4 and worker 24 used the prescribed 24-tile
fallback. Reduced Heaptrack peaks are not compared with full-corpus native
peaks.

| Tiles | Workers | Peak requested live heap | Package reconstruction | Other | SQLite | Vertex-cache category | Profile process peak RSS | Cgroup peak |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 182 | 1 | 1.62 GiB | 1.23 GiB | 0.37 GiB | 24.6 MiB | 0 | 2.33 GiB | 8.89 GiB |
| 24 | 1 | 0.59 GiB | 0.46 GiB | 0.11 GiB | 8.8 MiB | 0 | 0.71 GiB | 1.78 GiB |
| 24 | 4 | 0.96 GiB | 0.67 GiB | 0.28 GiB | 15.8 MiB | 0 | 2.22 GiB | 2.44 GiB |
| 24 | 24 | 4.77 GiB | 3.95 GiB | 0.76 GiB | 67.2 MiB | 0 | 7.21 GiB | 7.66 GiB |

Heaptrack's process RSS column uses the bake-off profile output because the
candidate writes results to files and does not emit baseline stage events.
Raw Heaptrack analyses retain requested/live allocation summaries and category
breakdowns. perf record completed for workers 1, 4, and 24 on 24 tiles;
Cachegrind completed for worker 1 on one tile.

## Provenance and raw artifacts

The original candidate checkout at '/tmp/cityjson-vertex-store-offsets'
remained clean at the requested commit. An isolated build overlay at
'/tmp/cityjson-vertex-store-offsets-profile' contained only the optional
profile-output hook in experiments/vertex-store-bakeoff.rs; BakeoffResult and
existing invocations were unchanged. The manifest records this overlay as
dirty with diff SHA-256
'f2bfeb737853c9d97fdd4e2f81590ef7f8339770849557ad1fd901af83e22db1'.

The campaign command was:

~~~sh
python3 crates/cityjson-index/tools/profile_vertex_store_campaign.py --candidate-worktree /tmp/cityjson-vertex-store-offsets-profile --candidate-commit 1f33165e5481455074ff007f7dff4b8d948e4287 --harness-commit bc23d7e --corpus-identity groningen-182-local-2026-08-04 --description "JSON offsets versus Tyler memory baseline" --memory-max 28G --corpus target/benchmarks/groningen-182/cityjson --work-root target/benchmarks/cityjson-index-vertex-store-profile --output-root target/profiling/cityjson-index-vertex-store --sidecar-182 target/benchmarks/cityjson-index-vertex-store-profile/json-offsets-182.sqlite --sidecar-24 target/benchmarks/cityjson-index-vertex-store-profile/json-offsets-24.sqlite --sidecar-1 target/benchmarks/cityjson-index-vertex-store-profile/json-offsets-1.sqlite
~~~

Every measured command is an explicit argv vector in its run metadata.json;
no shell-expanded candidate command was used. The systemd wrapper recorded
MemoryAccounting=yes, MemoryMax=28G, MemorySwapMax=0, and RuntimeMaxSec=3600.

Campaign manifest:

~~~text
target/profiling/cityjson-index-vertex-store/campaigns/20260804T045453Z-json-offsets-11d73be0/campaign.json
~~~

Raw artifacts:

~~~text
target/profiling/cityjson-index-vertex-store/campaigns/20260804T045453Z-json-offsets-11d73be0/runs/
~~~

The baseline reference remains
[tyler-profile-results-2026-08-03.md](tyler-profile-results-2026-08-03.md).

## Outcomes and limitations

There were zero unexplained failures, zero cgroup OOM kills, zero measured
cgroup swap-in/out counters, and stable digests within each prepared corpus:

- 182 tiles: one digest, 'sha256:7dc19f3033c0c7b8198114f5de9fec177baaba32a14b5aa5265b6d0de11de38c';
- 24 tiles: one reduced-corpus digest, 'sha256:fc4f98d58955d1346a3282f52094bdcae21de23afb67a5bc90823c042cbb7ce2';
- 1 tile: one reduced-corpus digest, 'sha256:2b6f52e579e503c3ee15299aac323700fab013b22994803549d2d7c78fd9cbe6'.

The host had pre-existing global swap usage, but every measured service had
MemorySwapMax=0 and cgroup swap counters remained zero. This report treats
cgroup counters, not host-global swap accounting, as the authority.

The candidate does not reproduce baseline cache-drop checkpoints or
cache-specific counters. It reads a prebuilt JSON-offset sidecar, whereas
baseline reads its own prepared SQLite layout. Sidecar construction and all
corpus preparation were outside measured processes. Heaptrack cgroup includes
its interpreter and trace buffers, so instrumented cgroup peaks are compared
only within the corresponding profiler tier.

## Conclusion

JSON offsets eliminate decoded-vertex retention in the measured telemetry, but
on this corpus they are substantially slower to materialize than Tyler at every
worker count. At 24 workers they also do not reduce process RSS or cgroup peak.
This is not a drop-in performance or memory improvement against the baseline;
further work would need to recover the 2.05x-2.28x materialization-time loss
while preserving zero retained decoded bytes.
