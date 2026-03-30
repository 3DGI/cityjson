# Backend Performance Investigation Results

## Run Contract

Date: 2026-03-30

Command:

```bash
cargo run --release --bin investigate-read-performance
```

The investigation binary uses the same shared deterministic workload builder as
the Criterion harness in
[/home/balazs/Development/cjindex/src/realistic_workload.rs](/home/balazs/Development/cjindex/src/realistic_workload.rs).

It measures three views of the workload:

1. direct index lookup only
2. indexed byte reads only
3. full `CityIndex` end-to-end reads

It also inspects the rebuilt SQLite indices so the report can explain byte
volume, working-set size, and `CityJSON` shared-vertices cache reuse.

## Dataset

The investigation runs against the prepared corpus under the bench root:

- `CJINDEX_BENCH_ROOT` or the default root `./tests/data`

The prepared corpus is produced by the reproducible 3DBAG prep pipeline against
the pinned `v20250903` tile index. The prep manifest under the output root
records the exact tile list, counts, and checksums. The figures referenced in
this report correspond to the manifest present on the run date above.

## Corrected Comparison Basis

The most important result from this investigation was not a timing number. It
was a semantic mismatch.

Before this change, regular `CityJSON` indexed every `CityObject`, including
children, while feature-files and `NDJSON` indexed feature packages. That meant
the old benchmark comparison was not measuring the same thing.

After fixing that:

- `CityJSON` now indexes 227,044 feature packages, matching feature-files and
  `NDJSON`
- regular `CityJSON get` reconstructs full root-plus-descendant feature
  packages
- the previous `CityJSON get` advantage disappears

## Workload Coverage

`get` batch shape:

- 1,000 reads
- 2,003 returned `CityObject`s
- all 191 `CityJSON` / `NDJSON` source tiles touched
- exactly 191 first-touch shared-vertices cache misses for `CityJSON`
- 809 later shared-vertices cache hits for `CityJSON`

Rotating bbox ring shape:

- 191 deterministic tile-local bboxes in the full sweep
- all 191 `CityJSON` / `NDJSON` source tiles touched in the sweep
- each measured Criterion batch uses the next 10 bboxes from that ring

The first 10-bbox benchmark batch from that ring touches:

- 7,927 feature hits
- 15,886 returned `CityObject`s
- 43 `CityJSON` / `NDJSON` source tiles

## Byte Volume

### `get`

| Backend | Avg Primary Bytes / Read | Total Primary Bytes / 1,000 Reads |
| --- | ---: | ---: |
| feature-files | `6,339.4` | `6,339,422` |
| `CityJSON` | `5,439.7` | `5,439,671` |
| `NDJSON` | `6,040.8` | `6,040,818` |

`CityJSON` still reads the smallest primary payload on `get`, but not by a huge
margin anymore once full feature-package member ranges are included.

### 10-bbox benchmark batch

| Backend | Avg Primary Bytes / Hit | Total Primary Bytes / Batch |
| --- | ---: | ---: |
| feature-files | `6,998.3` | `55,475,832` |
| `CityJSON` | `5,841.3` | `46,304,131` |
| `NDJSON` | `6,618.4` | `52,464,094` |

`CityJSON` again reads fewer primary bytes, and it also pays shared-vertices
cost on only 43 first touches in this batch.

## Staged Timings

### `get`

| Backend | Lookup Only | Read Only | Full | Estimated Remaining | Full Per Feature | Full Per `CityObject` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| feature-files | `2.660 ms` | `1.192 ms` | `86.114 ms` | `82.262 ms` | `86.114 us` | `42.992 us` |
| `CityJSON` | `2.904 ms` | `1.882 ms` | `89.734 ms` | `84.948 ms` | `89.734 us` | `44.799 us` |
| `NDJSON` | `2.629 ms` | `1.002 ms` | `84.131 ms` | `80.500 ms` | `84.131 us` | `42.002 us` |

### 10-bbox benchmark batch

| Backend | Lookup Only | Read Only | Full | Estimated Remaining | Full Per Feature | Full Per `CityObject` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| feature-files | `11.710 ms` | `11.354 ms` | `719.768 ms` | `696.704 ms` | `90.799 us` | `45.308 us` |
| `CityJSON` | `14.235 ms` | `15.096 ms` | `768.581 ms` | `739.250 ms` | `96.957 us` | `48.381 us` |
| `NDJSON` | `10.833 ms` | `8.305 ms` | `707.893 ms` | `688.755 ms` | `89.301 us` | `44.560 us` |

The "estimated remaining" column is `full - lookup - read_only`. It is not
pure parser time, but it is a good approximation for the work that remains once
the index lookup and explicit byte reads are stripped away. In practice, that
bucket is dominated by metadata handling, JSON decode, vertex localization, and
`CityModel` reconstruction.

## Conclusions

1. The old `CityJSON get` lead was not a durable backend advantage.
   It was largely a semantic artifact of reconstructing individual
   `CityObjects` instead of full feature packages.

2. Lookup cost is not the differentiator.
   All three backends spend only a few milliseconds of a 1,000-read batch on
   direct index lookup.

3. Explicit byte-read cost is also not the main differentiator.
   It remains a small fraction of the full batch even on bbox queries.

4. The dominant backend gap is in the residual decode / reconstruction work.
   Once the easy parts are removed, `CityJSON` still carries the largest
   remaining batch cost on both `get` and bbox reads.

5. The same ranking survives per-`CityObject` normalization.
   The current medians are `get`: feature-files `42.992 us`, `CityJSON`
   `44.799 us`, `NDJSON` `42.002 us`; `query`: feature-files `45.308 us`,
   `CityJSON` `48.381 us`, `NDJSON` `44.560 us`.

6. `CityJSON` still benefits from shared-state reuse.
   The `get` batch hits all 191 tiles but still converts 809 of 1,000 reads into
   shared-vertices cache hits after the first touch per tile.

7. The benchmark now has meaningful corpus spread.
   `get` covers all 191 tiles in one batch, and `query` / `query_iter` sweep the
   full 191-tile bbox ring over time instead of hammering a single hot tile.

## Practical Reading

In simple terms:

- `NDJSON` is currently the cleanest overall winner
- feature-files is very close to `NDJSON`
- `CityJSON` is no longer broken, but it still pays more reconstruction cost
  than the other two layouts
- that remains true even when normalized by returned `CityObject`, not just by
  feature package or by bbox
- the next optimization target is no longer raw indexed I/O
- the next optimization target is `CityJSON` feature-package reconstruction
