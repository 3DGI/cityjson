# Tyler pipeline profiling results — 2026-08-03

## Summary

The 182-tile Groningen matrix completed at 1, 4, and 24 workers under a
28 GiB cgroup limit. All 18 native and `perf stat` runs completed without an
OOM. Native medians show that 24 workers make feature materialization about
10 times faster than one worker, but duplicate the same 182 source-vertex
arrays into approximately 1,039 worker-local cache entries.

| Workers | Total time | Materialization | Process peak RSS | Cgroup peak | Cached source copies | Vertex capacity |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 138.97 s | 119.51 s | 3.10 GiB | 9.89 GiB | 182 | 3.60 GiB |
| 4 | 47.37 s | 28.07 s | 7.01 GiB | 7.52 GiB | 411 | 8.39 GiB |
| 24 | 29.15 s | 11.96 s | 18.63 GiB | 19.89 GiB | 1,039 | 22.44 GiB |

The requested vertex-buffer capacity can exceed RSS because reserved `Vec`
capacity is not necessarily backed by resident physical pages. It remains the
right measure of the heap capacity owned by the caches; cgroup memory and RSS
remain the deployment-cost measures.

The campaign and profiling policy are defined in
[ADR 011](../../crates/cityjson-index/docs/adr/011-tiered-tyler-profiling.md).
Preparation and sidecar construction happened outside every measured process.

## Worker scaling

Each value below is the median of three native runs over 707,239 packages.

| Workers | Total speedup | Materialization speedup | Materialization throughput |
|---:|---:|---:|---:|
| 1 | 1.00x | 1.00x | 5,918 packages/s |
| 4 | 2.93x | 4.26x | 25,187 packages/s |
| 24 | 4.77x | 9.99x | 59,115 packages/s |

The serial extent pass limits end-to-end scaling. Representative `perf stat`
runs used about 0.9, 2.7, and 8.3 CPU cores on average at 1, 4, and 24 workers.
The 24-worker run retired about 2.27 instructions per cycle, down from roughly
3.2 at 1 and 4 workers, and incurred about 601,000 context switches. Adding
workers beyond this point is therefore unlikely to produce proportional speed
gains even before memory is considered.

The three native repetitions were stable:

| Workers | Total-time range | Cgroup-peak range |
|---:|---:|---:|
| 1 | 138.77–141.89 s | 9.83–10.42 GiB |
| 4 | 47.35–47.46 s | 7.40–7.52 GiB |
| 24 | 28.45–29.42 s | 19.87–19.93 GiB |

## Direct cache evidence

Immediately after materialization, the benchmark queried every Rayon worker's
thread-local `CityIndex`. It then dropped those indexes while keeping the
worker pool alive and recorded another RSS checkpoint.

| Workers | Unique sources | Cached source copies | Duplication | Vertex capacity | RSS before drop | RSS after drop |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 182 | 182 | 1.00x | 3.60 GiB | 3.09 GiB | 0.26 GiB |
| 4 | 182 | 411 | 2.26x | 8.39 GiB | 7.01 GiB | 1.98 GiB |
| 24 | 182 | 1,039 | 5.71x | 22.44 GiB | 18.59 GiB | 4.88 GiB |

This establishes the main mechanism directly: a source is cached once in each
worker that happens to process one of its packages. At 24 workers, the workload
retained almost six copies of each source on average. The cache is unbounded,
so those copies live until their worker-local `CityIndex` is dropped.

Dropping the caches released most resident memory, but the remaining RSS also
grew with worker count. That remainder includes the package-reference vector,
SQLite state, Rayon stacks, allocator arenas and metadata, and pages retained
by the allocator. It is not all fragmentation.

## Heaptrack allocation attribution

The staged full-corpus Heaptrack run completed at one worker with an 8.60 GiB
cgroup peak. The conservative four-worker projection was 34.39 GiB, above the
25.2 GiB safety threshold, so the campaign did not risk the 30 GiB host. It ran
the defined 24-tile 1/4/24 fallback instead.

| Tiles | Workers | Peak requested heap | Vertex-cache call path | Exact vertex capacity | Process peak RSS | RSS released by cache drop |
|---:|---:|---:|---:|---:|---:|---:|
| 182 | 1 | 3.78 GiB | 3.57 GiB | 3.60 GiB | 3.12 GiB | 2.86 GiB |
| 24 | 1 | 0.37 GiB | 0.35 GiB | 0.34 GiB | 0.33 GiB | 0.26 GiB |
| 24 | 4 | 0.80 GiB | 0.77 GiB | 0.76 GiB | 0.71 GiB | 0.57 GiB |
| 24 | 24 | 0.97 GiB | 0.87 GiB | 0.89 GiB | 1.29 GiB | 0.54 GiB |

The Heaptrack category and the direct cache counter agree closely. For the
full-corpus row, allocations whose stack includes `load_shared_vertices` or
`parse_vertices_fragment` account for 3.57 GiB, while the benchmark reports
3.60 GiB of retained vertex-buffer capacity. The remaining peak requested heap
is approximately:

- 103 MiB from package reconstruction;
- 114 MiB from the preloaded package-reference vector;
- 4 MiB from SQLite;
- less than 1 MiB from Rayon/runtime and uncategorized allocations.

This makes the vertex caches the dominant live allocation, not reconstructed
`CityModel` values or SQLite. Reconstructed models are dropped after every
package and contribute allocation traffic, but little retained peak memory.

### Allocator fragmentation and non-resident capacity

Heaptrack reports requested allocation sizes, whereas RSS reports resident
pages. At one and four workers, requested live heap exceeded RSS by 35–710 MiB.
That is expected when large vector capacities contain untouched or reclaimed
pages; it means an allocator-fragmentation bound cannot be computed from their
difference.

For the reduced 24-worker run, process peak RSS exceeded requested live heap by
342 MB. This is an upper bound, not a pure fragmentation measurement: it also
contains allocator metadata and arenas, stacks, anonymous mappings, and
Heaptrack overhead. After the cache drop, 805 MB RSS remained, compared with
80 MB at one worker and 152 MB at four workers. The 24-worker allocator/runtime
residual is therefore material, but it is secondary to the 0.89 GiB of directly
measured cache capacity in that reduced run and the 22.44 GiB seen in the full
native workload.

## Cgroup and reclaim interpretation

Each run used a transient systemd cgroup v2 service with `MemoryMax=28G`,
`MemorySwapMax=0`, 100 ms memory sampling, and authoritative
`memory.events.oom_kill` classification. All 26 campaign runs reported zero
OOM kills.

The cgroup peak includes the benchmark, profiler processes, file cache, and
charged kernel memory. This is why Heaptrack's full-corpus cgroup peak is much
higher than the benchmark process RSS: the Heaptrack interpreter and trace
buffers are charged to the same limit. Instrumented cgroup peaks must not be
compared directly with native application peaks.

The earlier corrected native run showed file cache being reclaimed as
anonymous heap grew: about 1.78 million pages were scanned and reclaimed,
almost entirely by `kswapd`, with only 21 major faults and low pressure stalls.
That evidence remains consistent with the new matrix. The cache counters now
identify the anonymous allocations that caused the pressure.

## Conclusions

- Worker-local unbounded vertex caches are the primary retained-memory cost.
  The 24-worker native run retained about 22.44 GiB of requested vertex-buffer
  capacity across 1,039 source copies.
- Allocator/runtime overhead becomes visible at 24 workers, but Heaptrack and
  cache-drop checkpoints show that it is not the primary cause.
- Four workers are the best measured balance in this matrix: 2.93x total
  speedup over one worker with a 7.52 GiB median cgroup peak. Twenty-four
  workers improve total time by another 1.63x but raise the median peak to
  19.89 GiB.
- The optimization should prevent the same source vertices from being cached
  independently by multiple workers, or bound/evict worker-local caches. It
  should then repeat this exact matrix to confirm both lower memory and
  unchanged or improved throughput.

## Reproducibility

The campaign was invoked from `crates/cityjson-index` with:

```sh
just profile-index-campaign "vertex-cache allocation baseline" 28G
```

The campaign manifest is:

```text
target/profiling/cityjson-index/campaigns/20260803T091154Z-tyler-matrix-8da45c70/campaign.json
```

The full-corpus Heaptrack artifact is:

```text
target/profiling/cityjson-index/20260803T093606Z-heaptrack-w1-t182-bffb7ac4
```

Every artifact records commit `279c34c`, the profiling-tool version, dirty
worktree status, and a SHA-256 digest of the tracked diff plus status. The
binary included the changes described here, but the campaign was intentionally
run before committing them; the provenance therefore records a dirty worktree
rather than claiming commit-exact results. Generated profiler artifacts remain
outside Git.
