# Parallel Indexing Plan

## Goal

Add parallel indexing support without changing the CityJSON output format or the existing dataset-first CLI shape.

## Constraints

- Keep the current sidecar index model.
- Preserve deterministic feature ids and query results.
- Do not regress the existing single-worker path.
- Keep the implementation compatible with the current `cjindex` commands.

## Approach

1. Split indexing work into independent source/file shards.
2. Parse and scan shards in parallel.
3. Merge shard output into a single SQLite write phase.
4. Keep query and reconstruction behavior unchanged.

## Validation

- Compare single-worker and multi-worker index output.
- Check source counts, feature counts, CityObject counts, and representative feature reconstruction.
- Verify bbox query hit counts for a fixed set of datasets.

