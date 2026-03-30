# Plan for Explaining Backend Performance Differences in `cjindex`

## Goal

Produce a reproducible explanation for the current steady-state performance
differences between:

- feature-files
- regular `CityJSON`
- `NDJSON`

The explanation needs to be more concrete than "one backend feels faster". It
should identify which parts of the read path are materially different and how
those differences show up in the current realistic workloads.

## Questions To Answer

For the existing realistic `get` and bbox workloads, we want exact answers to
these questions:

1. how much of the time is spent on index lookup versus byte reads versus the
   full end-to-end `CityModel` path?
2. how many bytes does each backend actually read for the measured workload?
3. how large is the working set in terms of source files touched?
4. how much shared-state reuse does `CityJSON` get from its per-source shared
   vertices cache?
5. are the current differences explained mainly by I/O volume, by cache reuse,
   or by parse / reconstruction cost?

## Non-Goals

- cache-cold benchmarking
- hardware-independent absolute latency claims
- a general production profiling framework
- changing the existing benchmark contract again

This work should explain the current benchmark numbers, not replace them.

## Measurement Strategy

We will keep using the existing realistic workload shape:

- `get`: 1,000 deterministic pseudo-random IDs
- `query` / `query_iter`: 10 deterministic real bbox queries

The investigation will add a reproducible analysis tool that measures three
separate views of the same workload:

1. index lookup only
2. indexed byte reads only
3. full `CityIndex` end-to-end reads

The analysis tool will also inspect the built SQLite indices so it can report:

- average and total indexed span lengths per backend
- unique source files touched by each workload
- `CityJSON` shared vertices span sizes
- exact `CityJSON` cache hit / miss counts implied by the workload order
- total result counts for the bbox workload

## Implementation Plan

### Phase 1: Share the realistic workload builder

Move the deterministic workload construction out of the Criterion harness into
a reusable crate module so both the benches and the investigation tool use the
same workload definition.

That shared module should provide:

- prepared dataset resolution
- deterministic `get` ID selection
- deterministic bbox selection
- canonical feature record collection

### Phase 2: Add a dedicated investigation binary

Add a `src/bin/...` tool that:

1. prepares or resolves the fixture datasets
2. builds fresh indices for all three layouts
3. reconstructs the realistic workload from the shared module
4. gathers workload-shape statistics directly from the index database
5. runs timed measurement rounds for:
   - lookup-only batches
   - read-only batches
   - full `get` batches
   - full bbox-query batches

The timing runner should warm the process first, then record several measured
rounds and report stable batch medians instead of one noisy run.

### Phase 3: Keep the read-only measurements honest

The read-only path should mimic the real backend data access pattern closely
enough to make the residual meaningful:

- feature-files / `NDJSON`: indexed range reads for each feature
- `CityJSON`: indexed object-fragment reads plus one shared-vertices read on
  first touch per source, with the cache retained across rounds

This will let us compare:

- lookup-only cost
- bytes-and-cache cost
- remaining parse / reconstruction cost

without changing the production hot path.

### Phase 4: Document the results

Write a new results document that includes:

- exact commands used
- the measured stage timings
- byte-volume and working-set summaries
- the explanation for why `CityJSON get` is ahead of `NDJSON get`
- the explanation for why `CityJSON` still trails slightly on bbox queries

If the previous ADR needs clarification based on the new evidence, update it on
trunk instead of creating contradictory documentation.

## Verification Plan

Before trusting the investigation:

1. run `cargo test --all-features`
2. run `cargo test --release --tests`
3. run `cargo check --benches`
4. run the new investigation binary and record its output
5. rerun the full release benchmark suite:
   - `cargo bench --bench cityjson`
   - `cargo bench --bench feature_files --bench ndjson`

## Expected Outcome

At the end of this work, we should be able to say, with numbers:

- how much of the backend gap is lookup overhead
- how much is explained by byte volume
- how much `CityJSON` gains from shared-vertices reuse
- whether the remaining gap is best treated as parse / reconstruction work

That is the level of detail needed to decide what the next optimization target
should be.
