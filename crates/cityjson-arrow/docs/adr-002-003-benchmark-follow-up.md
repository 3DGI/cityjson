# ADR 2 And ADR 3 Benchmark Follow-Up

This note records the first end-to-end benchmark run after the ADR 3 public
surface refactor and defines the split benchmark matrix needed to drive the
next optimization slice.

The source run is the `cjlib` campaign:

- description: `cityarrow refactor e085b91e`
- timestamp: `2026-04-02T09:08:03Z`
- baseline comparison: `2026-04-01T21:21:02Z`
- source data: `cjlib/bench_results/history.csv`

## Executive Summary

The refactor did improve the native paths.

For `cityarrow`:

| Path | Baseline | Refactor | Delta |
| --- | --- | --- | --- |
| tile read | `37.46 ms` | `29.03 ms` | `-22.5%` |
| cluster read | `152.79 ms` | `119.12 ms` | `-22.0%` |
| tile write | `56.81 ms` | `47.45 ms` | `-16.5%` |
| cluster write | `235.26 ms` | `197.16 ms` | `-16.2%` |

For `cityparquet`:

| Path | Baseline | Refactor | Delta |
| --- | --- | --- | --- |
| tile read | `40.78 ms` | `29.11 ms` | `-28.6%` |
| cluster read | `160.57 ms` | `118.18 ms` | `-26.4%` |
| tile write | `66.86 ms` | `46.83 ms` | `-30.0%` |
| cluster write | `271.72 ms` | `197.35 ms` | `-27.4%` |

That means the refactor is not a no-op. It removed enough overhead for the
native read paths to cross over `serde_cityjson` on both pinned cases.

The refactor did not, however, deliver the full performance story implied by
ADR 2 and ADR 3:

- read allocation totals remained effectively unchanged
- write remains much slower than the shared-model JSON path
- the current benchmark suite still cannot localize the remaining cost

The practical conclusion is:

- ADR 2 was directionally correct that transport/package implementation was
  part of the benchmark shortfall
- ADR 3 changed the public boundary and removed the old directory contract
- the implementation still does not realize the intended stream-first and
  lazy-package execution model, so the remaining gap is unsurprising

## Why The Plot Summary Understates The Change

The generated plot summary in `cjlib` is normalized against the same-run
`serde_json::Value` baseline.

That is the correct denominator for cross-format end-to-end plots, but it is
not the right lens for judging whether the `cityarrow` refactor itself moved
anything. In the refactor run the JSON baseline also became faster, so ratios
such as `0.15x -> 0.15x` can hide real absolute improvements.

For refactor evaluation, the comparison that matters is:

- new `cityarrow` vs old `cityarrow`
- new `cityparquet` vs old `cityparquet`
- new native path vs current `serde_cityjson`

On that basis, the read side did move materially.

## What The Current Implementation Still Does

The current code no longer exposes the old package-directory API, but the hot
path still materializes the same internal structures.

- `ModelEncoder::encode` still does `OwnedCityModel -> encode_parts ->
  write_model_stream`
- `ModelDecoder::decode` still does `read_model_stream -> decode_parts ->
  OwnedCityModel`
- `PackageWriter::write_file` still does `OwnedCityModel -> encode_parts ->
  package file`
- `PackageReader::read_file` still does `package file -> parts ->
  decode_parts -> OwnedCityModel`

The implementation is therefore still dominated by whole-model materialization:

- the stream encoder first builds full canonical parts
- the stream writer buffers per-table payloads before writing the final output
- the stream reader reads the full source into memory before table decode
- the package reader still deserializes all tables and builds full parts before
  model reconstruction

So the public architecture is now aligned with ADR 3, but the execution model
is still much closer to the ADR 1 and ADR 2 baseline than to the intended end
state.

## Updated Reading Of ADR 2

This run strengthens ADR 2 rather than weakening it.

ADR 2 argued that the benchmark gap should be treated as an implementation
problem in transport conversion and package I/O, not as proof that the shared
`OwnedCityModel` boundary is wrong.

The post-refactor numbers support that reading:

- read performance improved enough to erase the previous `cityarrow` read loss
  against `serde_cityjson`
- `cityparquet` moved even more, which indicates that reducing format and
  container overhead matters
- unchanged allocation totals strongly suggest that the remaining gap is now
  concentrated in conversion and materialization work rather than in the new
  public API shape itself

What ADR 2 still lacks is the benchmark decomposition it explicitly called for.

## Updated Reading Of ADR 3

This run shows that ADR 3 is only partially realized today.

Delivered:

- live Arrow stream and persistent package are now separate public surfaces
- the old directory-oriented package contract is gone from the supported API
- the benchmark corpus and downstream wrapper now reflect the new transport
  boundary cleanly

Not yet delivered:

- incremental stream decode
- stream encode without whole-parts materialization
- package read that localizes work before full table materialization
- bound-column import/export as the dominant steady-state implementation model

That distinction matters. The refactor proves that the cut line itself was not
harmful. It does not yet prove that the intended ADR 3 execution model is in
place.

## Exact Split Benchmarks To Add

The next benchmark slice should separate conversion, transport, and end-to-end
cost using one pinned fixture set.

Use these two cases only:

- `io_3dbag_cityjson`
- `io_3dbag_cityjson_cluster_4x`

Prepare these artifacts once per case outside the measured loop:

- `OwnedCityModel`
- canonical parts from `cityarrow::internal::encode_parts`
- live stream bytes from `ModelEncoder`
- package file from `PackageWriter`

### Headline End-To-End Benchmarks

These remain the product metrics.

| Benchmark id | Operation | Answers |
| --- | --- | --- |
| `stream_write_model` | `ModelEncoder.encode(&model, writer)` | current live export cost seen by users |
| `stream_read_model` | `ModelDecoder.decode(reader)` | current live import cost seen by users |
| `package_write_model` | `PackageWriter.write_file(path, &model)` | current persistent export cost seen by users |
| `package_read_model` | `PackageReader.read_file(path)` | current persistent import cost seen by users |

### Conversion-Only Benchmarks

These isolate the shared-model boundary work that ADR 2 said might dominate.

| Benchmark id | Operation | Answers |
| --- | --- | --- |
| `convert_encode_parts` | `internal::encode_parts(&model)` | how expensive flattening/export is before any transport serialization |
| `convert_decode_parts` | `internal::decode_parts(&parts)` | how expensive reconstruction/import is before any stream or package I/O |

### Transport-Only Benchmarks

These measure stream/package overhead after conversion has already happened.

To keep the public API clean, these should use doc-hidden internal helpers
rather than reintroducing a public `parts` surface.

| Benchmark id | Operation | Answers |
| --- | --- | --- |
| `stream_write_parts` | canonical parts to live stream bytes | stream framing and Arrow IPC serialization cost without model flattening |
| `stream_read_parts` | live stream bytes to canonical parts | stream parse and Arrow IPC decode cost without model reconstruction |
| `package_write_parts` | canonical parts to package file | container write and table serialization cost without model flattening |
| `package_read_parts` | package file to canonical parts | mmap, index walk, table decode, and concat cost without model reconstruction |
| `package_read_manifest` | `PackageReader.read_manifest(path)` | fixed footer/index overhead before table decode |

### Measurement Rules

Use these rules consistently:

- end-to-end benchmarks report wall-clock time and logical dataset throughput
- conversion-only and transport-only benchmarks report wall-clock time first
- transport-only benchmarks may also report native payload throughput because
  they compare like-for-like encoded bytes
- allocation counters should be recorded for at least:
  `convert_decode_parts`, `stream_read_parts`, and `package_read_parts`
- setup must not parse CityJSON, regenerate parts, or rebuild encoded payloads
  inside the timed loop

`package_write_*` needs one extra rule:

- pre-create the temp directory once and overwrite a fixed path in the timed
  loop

That avoids benchmarking `tempdir()` churn instead of package writing.

## How To Interpret The Split Results

Use these decision rules after the first split run:

- if `convert_decode_parts` is close to `stream_read_model` and
  `package_read_model`, the next work should focus on import reconstruction
- if `stream_read_parts` is still large, the live stream path is still too
  eager and not yet behaving like ADR 3's intended streaming boundary
- if `package_read_manifest` is tiny but `package_read_parts` is large, fixed
  container overhead is no longer the main problem; table decode and batch
  concatenation are
- if `convert_encode_parts` dominates both write paths, export traversal and
  row construction are the next optimization target
- if `package_write_parts` is much slower than `stream_write_parts`, the
  persistent container layer still has avoidable serialization overhead

## Recommended Benchmark Harness Shape

Add the split suite in `cityarrow` itself rather than keeping it only in
`cjlib`.

The clean shape is:

- one Criterion target for end-to-end public API benchmarks
- one Criterion target for split transport and conversion benchmarks
- one lightweight profiling binary or script for heap and cache follow-up on
  the hottest split benchmarks

The benchmark-only helper surface should be limited to doc-hidden internal
bridges such as:

- `internal::write_stream_parts`
- `internal::read_stream_parts`
- `internal::write_package_parts`
- `internal::read_package_parts`

That keeps the user-facing API aligned with ADR 3 while still making the
benchmark decomposition executable.

## Immediate Optimization Priority

Based on the current code and the unchanged allocation profile, the most likely
next dominant cost is `decode_parts`, followed by `encode_parts`.

That suggests this order:

1. add the split benchmarks above
2. confirm whether `convert_decode_parts` dominates read time
3. if confirmed, rewrite import around bound columns and span-based traversal
4. then revisit export row construction and package/stream buffering
