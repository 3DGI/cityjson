# Generated Benchmark Profiles

This directory does not store generated fixtures. It stores the benchmark
profile catalog used by the `cjfake`-backed read and write suites.

The benchmark inputs are generated deterministically at benchmark time from the
profiles in [`manifest.json`](/home/balazs/Development/serde_cityjson/tests/data/generated/manifest.json)
and the per-case profile files in
[`profiles/`](/home/balazs/Development/serde_cityjson/tests/data/generated/profiles).

## Purpose

The profiles are designed around the current deserializer and serializer hot
paths rather than around the shape of any single real dataset.

The main isolates are:

- geometry flattening
- root vertex transform and import
- recursive attribute conversion
- relation resolution
- deep boundary parsing
- serializer validation and appearance output

## Case Set

- `3DBAG` and `3D Basisvoorziening` are real-world regression datasets.
- `geometry_flattening_best_case` is the best-case geometry stream.
- `vertex_transform_stress` isolates vertex import and transform work.
- `attribute_tree_worst_case` stresses recursive attribute conversion.
- `relation_graph_worst_case` stresses parent/child resolution.
- `deep_boundary_stress` stresses boundary parsing and stored geometry.
- `composite_value_favorable_worst_case` mixes the normalization costs.
- `appearance_and_validation_stress` is the write-only serializer stress case.

## Benchmark Workflow

1. `just bench-read`
2. `just bench-write`
3. `just bench-report`
4. Paste the generated summary into the main README benchmark section

The benchmark binaries also emit suite metadata into `benches/results/` so the
reporting script can compute throughput from the measured timing data.
