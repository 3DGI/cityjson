# Development

Tests and benchmarks depend on a local checkout of the
[`cityjson-corpus`](https://github.com/cityjson/cityjson-corpus) repository.
Configure that checkout with `CITYJSON_JSON_SHARED_CORPUS_ROOT`, or point
directly at an index file with `CITYJSON_JSON_CORRECTNESS_INDEX` or
`CITYJSON_JSON_BENCHMARK_INDEX`.

## Setup

```bash
export CITYJSON_JSON_SHARED_CORPUS_ROOT=/path/to/cityjson-corpus
just ci
```

If you do not want to set a shared root, you can point directly at the manifest
files instead:

```bash
export CITYJSON_JSON_CORRECTNESS_INDEX=/path/to/cityjson-corpus/artifacts/correctness-index.json
export CITYJSON_JSON_BENCHMARK_INDEX=/path/to/cityjson-corpus/artifacts/benchmark-index.json
```

Relative index paths are resolved against `CITYJSON_JSON_SHARED_CORPUS_ROOT`.

## Running Tests

```bash
just test
```

The corpus-backed correctness tests read fixture IDs from the configured
correctness index.

## Running Benchmarks

```bash
just bench-read
just bench-write
just bench-report
```

The benchmark suite reads the shared benchmark index and the artifacts listed in
each workload's `artifacts[]` array.

To benchmark your own CityJSON file or a directory of CityJSON files without
depending on the shared corpus, run:

```bash
just bench-local /path/to/cityjson-or-directory
```

That command generates a temporary benchmark index under
`target/bench-local/benchmark-index.json` and runs the existing read and write
suites against those inputs.

The benchmarks use Criterion. Read throughput is based on input bytes; write
throughput is based on output bytes. The shared benchmark suite can refresh the
main README benchmark table with:

```bash
just bench
```

That README table is generated from current Criterion output and should not be
edited by hand.
