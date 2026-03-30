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
- all 191 `CityJSON` / `NDJSON` source tiles touched
- exactly 191 first-touch shared-vertices cache misses for `CityJSON`
- 809 later shared-vertices cache hits for `CityJSON`

Rotating bbox ring shape:

- 191 deterministic tile-local bboxes in the full sweep
- all 191 `CityJSON` / `NDJSON` source tiles touched in the sweep
- each measured Criterion batch uses the next 10 bboxes from that ring

The first 10-bbox benchmark batch from that ring touches:

- 7,927 feature hits
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

| Backend | Lookup Only | Read Only | Full | Estimated Remaining |
| --- | ---: | ---: | ---: | ---: |
| feature-files | `2.638 ms` | `1.199 ms` | `85.318 ms` | `81.481 ms` |
| `CityJSON` | `2.905 ms` | `1.824 ms` | `88.416 ms` | `83.686 ms` |
| `NDJSON` | `2.670 ms` | `992.478 us` | `84.448 ms` | `80.786 ms` |

### 10-bbox benchmark batch

| Backend | Lookup Only | Read Only | Full | Estimated Remaining |
| --- | ---: | ---: | ---: | ---: |
| feature-files | `11.592 ms` | `11.159 ms` | `707.188 ms` | `684.437 ms` |
| `CityJSON` | `14.068 ms` | `14.861 ms` | `747.442 ms` | `718.513 ms` |
| `NDJSON` | `10.836 ms` | `8.237 ms` | `693.624 ms` | `674.550 ms` |

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

5. `CityJSON` still benefits from shared-state reuse.
   The `get` batch hits all 191 tiles but still converts 809 of 1,000 reads into
   shared-vertices cache hits after the first touch per tile.

6. The benchmark now has meaningful corpus spread.
   `get` covers all 191 tiles in one batch, and `query` / `query_iter` sweep the
   full 191-tile bbox ring over time instead of hammering a single hot tile.

## Practical Reading

In simple terms:

- `NDJSON` is currently the cleanest overall winner
- feature-files is very close to `NDJSON`
- `CityJSON` is no longer broken, but it still pays more reconstruction cost
  than the other two layouts
- the next optimization target is no longer raw indexed I/O
- the next optimization target is `CityJSON` feature-package reconstruction
