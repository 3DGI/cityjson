# ADR 013: Refactor `CityIndex` for Concurrent Reads

- Status: Proposed
- Date: 2026-08-03

## Context

`CityIndex` currently owns one `rusqlite::Connection` and one storage backend.
`rusqlite::Connection` is `Send` but not `Sync`, so a `CityIndex` cannot be
shared by concurrent readers. A multithreaded caller must open an independent
index in every worker or keep one in thread-local storage.

The Tyler pipeline follows that pattern. Its 24-worker profile opened one
`CityIndex` per participating Rayon worker. Because each regular-CityJSON
backend also owned an unbounded source-vertices cache, the workers retained
1,039 source-array copies and 22.44 GiB of vertex capacity. The
[profiling results][tyler-profile] establish that cache duplication is the
dominant retained-memory cost.

[ADR 012](012-evaluate-persistent-sparse-vertex-storage.md) evaluates
persistent sparse vertex representations that remove the source-sized decoded
cache. That solves the largest allocation mechanism, but independently opened
indexes would still duplicate SQLite connections, metadata caches, backend
state, configuration, and lifecycle handling. More importantly, the public API
would still force every concurrent caller to invent its own thread-local
connection management.

`cityjson-index` needs one explicit concurrency model for read operations. It
must bound SQLite resources, reuse connections, make batching efficient, and
coordinate in-process reindexing without coupling the crate to Rayon or any
other scheduling runtime.

## Decision

Refactor `CityIndex` into a cheap clone over shared inner state. The public type
will implement `Clone`, `Send`, and `Sync` and will internally own:

- the immutable `StorageLayout` and storage backend;
- the sidecar path;
- a bounded pool of read-only SQLite connections;
- one shared source-metadata cache;
- an in-process read/reindex coordination gate.

Callers share or clone one `CityIndex` and invoke its existing read methods
directly from their workers. `cityjson-index` supplies connection management,
but it does not create threads, choose a scheduler, or expose a parallel
iterator.

```rust
use rayon::prelude::*;

use cityjson_index::{CityIndex, CityIndexOptions};

let index = CityIndex::open_with_options(
    layout,
    &index_path,
    CityIndexOptions::default(),
)?;

package_refs
    .par_chunks(2_048)
    .try_for_each(|page| index.read_packages(page).map(|_| ()))?;
# Ok::<(), cityjson_lib::Error>(())
```

The batch APIs remain the preferred concurrency unit. One `read_packages` call
checks out one connection for the complete batch, performs all associated
queries and persistent vertex reads, and returns the connection when the call
finishes. A caller must not loop over `read_package` when it already has a
batch.

## Public Rust API

Add a configuration type with validated, non-zero pool capacity:

```rust
use std::num::NonZeroU32;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CityIndexOptions {
    pub max_read_connections: NonZeroU32,
    pub connection_timeout: Duration,
}
```

`CityIndexOptions::default()` uses
`std::thread::available_parallelism()`, falling back to one connection when the
parallelism cannot be determined, and a 30-second checkout timeout. The default
does not inspect Rayon or any application-specific environment variable.

Add the configurable constructor while retaining the current convenience:

```rust
impl CityIndex {
    pub fn open(layout: StorageLayout, index_path: &Path) -> Result<Self>;

    pub fn open_with_options(
        layout: StorageLayout,
        index_path: &Path,
        options: CityIndexOptions,
    ) -> Result<Self>;

    pub fn reindex(&self) -> Result<()>;
}
```

`open` delegates to `open_with_options` with defaults. Changing `reindex` from
`&mut self` to `&self` reflects that exclusivity is enforced by shared runtime
state rather than by one Rust binding. Existing calls through `&mut CityIndex`
continue to compile, but ownership and concurrency semantics intentionally
change.

The existing lookup, paging, query, metadata, count, package-read, and filtered
read method names and result types remain unchanged. No checked-out connection
or pool implementation type becomes public API.

## Connection Pool

Use `r2d2` with a crate-local `rusqlite` connection manager rather than expose
`r2d2_sqlite` or couple the public API to either crate. The manager opens every
pooled connection with SQLite read-only flags and configures:

- `PRAGMA foreign_keys = ON`;
- `PRAGMA query_only = ON`;
- the configured busy/checkout behavior needed by the pool.

The pool capacity is exactly `max_read_connections`. Exhaustion blocks only up
to `connection_timeout`; timeout and connection-validation failures are mapped
to `cityjson_lib::Error` with an actionable message.

`CityIndex::open_with_options` first uses a short-lived read/write bootstrap
connection to create or validate the sidecar schema, preserving today's
behavior for a missing index. It then creates the read-only pool. Normal read
operations never receive a writable SQLite connection.

Each top-level read operation checks out at most one connection. Public methods
that compose other operations must call connection-scoped private helpers
instead of recursively invoking another public method and checking out a
second connection. This invariant prevents deadlocks when the configured pool
size is one.

Operations that require multiple SQL statements, including package lookup plus
reconstruction, run inside one deferred SQLite read transaction. The operation
therefore observes one coherent sidecar snapshot even if a separately opened
process commits a reindex concurrently.

## Shared State and Caches

The internal shape is conceptually:

```rust
#[derive(Clone)]
pub struct CityIndex {
    inner: Arc<CityIndexInner>,
}

struct CityIndexInner {
    layout: StorageLayout,
    index_path: PathBuf,
    backend: Arc<dyn StorageBackend>,
    readers: Pool<ReadConnectionManager>,
    metadata_cache: Mutex<HashMap<i64, CachedMetadata>>,
    lifecycle: LifecycleGate,
}
```

The exact private type decomposition may differ, but ownership must preserve
these properties:

- all clones use the same connection bound;
- all clones share one metadata cache;
- storage backends are immutable, `Send`, and `Sync`;
- regular-CityJSON vertices come from the persistent store selected after ADR
  012, not from a decoded source-level backend cache;
- file handles and decoded package vertices remain operation- or batch-local;
- dropping one clone does not invalidate work performed through another;
- dropping the final clone closes idle pooled connections and releases shared
  caches.

The metadata cache remains small and stores both parsed `Arc<Meta>` values and
serialized `Arc<[u8]>` values. Population uses double-checked locking or an
equivalent entry mechanism so concurrent misses do not retain duplicate cache
entries.

## Reindex Coordination

Every public read operation acquires the lifecycle gate in shared mode before
checking out a connection and holds it until its read transaction and all
source reconstruction work finish.

`reindex(&self)` acquires the same gate in exclusive mode. The gate must prefer
a waiting writer so a continuous stream of readers cannot starve reindexing.
After existing in-process readers drain, reindexing:

1. opens a dedicated read/write SQLite connection;
2. performs the existing scan and transactional sidecar replacement;
3. commits the complete new index or preserves the previous index on failure;
4. clears the shared metadata cache only after a successful commit;
5. releases the exclusive gate, allowing new reads to observe the new state.

No pooled read connection is repurposed for writes. SQLite transaction and
locking semantics continue to serialize independently opened writers and
provide snapshot behavior to readers in other processes. The in-process gate
coordinates clones of one `CityIndex`; it does not claim to lock source files
or prevent another process from editing the dataset. Freshness and validation
remain responsible for detecting source changes.

An index whose schema state requires reindexing continues to reject package
reads with rebuild guidance. Pooling must not turn a stale or incompatible
sidecar into a partially readable one.

## Failure and Panic Behavior

- A failed checkout returns an error; it does not silently open an unbounded
  fallback connection.
- A read error rolls back its read transaction and returns its connection to
  the pool when the connection remains valid.
- A broken SQLite connection is discarded and may be replaced within the
  configured pool bound.
- A failed or panicking reindex releases the lifecycle gate and leaves the
  previously committed sidecar and metadata cache usable. Cache clearing occurs
  only after commit.
- Mutex or gate poisoning is converted consistently with the crate's existing
  error policy; it must not create an unsafe partially initialized handle.
- Results preserve current deterministic record ordering, input ordering, and
  duplicate-reference behavior regardless of scheduling.

## CLI, C, and Python Scope

The CLI uses the redesigned `CityIndex` internally but does not gain worker or
pool flags in this change. Its commands and output remain unchanged.

The C ABI and Python API retain their current constructors and operations. They
benefit from the shared internal implementation, but this ADR does not add
cross-language pool configuration or promise that concurrent calls racing with
handle destruction are safe. The Rust API is stabilized first; explicit FFI
concurrency and lifetime contracts require separate review.

## Alternatives Considered

### Keep Thread-Local `CityIndex` Values

Persistent sparse vertices would remove the largest duplicated cache, but each
application would still own connection creation, bounds, errors, and cleanup.
The primary public type would remain unusable as shared concurrent state.

### Add a Separate `CityIndexPool`

A separate administrative `CityIndex` and pooled reader facade would preserve
more of the current ownership model. It would also duplicate the read surface,
force callers to choose between two index types, and leave the main abstraction
architecturally split. A breaking internal redesign produces a smaller and
clearer public API.

### Put One Connection Behind a Mutex

This would make the type `Sync` but serialize every SQL query and package read.
It does not meet the throughput goal and makes long batch reconstruction block
unrelated lookups.

### Expose `rusqlite::Connection` or Pool Guards

Leaking storage implementation types would prevent future pool or database
changes and let callers violate transaction and reindex invariants. Connection
ownership remains private.

### Expose a Rayon `ParallelIterator`

The crate must not choose the caller's scheduler. Page and batch APIs provide
the correct boundaries for Rayon, Tokio blocking pools, scoped threads, and
other concurrency models.

## Testing

Add compile-time and behavioral coverage for:

- `CityIndex: Clone + Send + Sync`;
- concurrent lookups and `read_packages` calls through clones of one index;
- exact output parity between serial and concurrent reads for all three storage
  layouts;
- a pool size of one, including composed APIs, to prove there are no nested
  checkout deadlocks;
- enforcing the configured connection maximum under contention;
- timeout and broken-connection error propagation;
- one shared metadata entry per source across concurrent clones;
- dropping clones while other clones remain active;
- an exclusive reindex waiting for active readers and blocking later readers;
- readers observing either the complete pre-reindex or complete post-reindex
  state, never partially replaced tables;
- failed reindex preserving the previous sidecar and cache;
- deterministic ordering and duplicate alignment under concurrent scheduling;
- unchanged C and Python behavior.

Tests that need deterministic contention use test-only barriers and connection
instrumentation rather than timing sleeps.

## Performance Validation

First establish the winning persistent vertex representation and its baseline
using ADR 012. Then replace the Tyler benchmark's worker-local indexes with one
shared `CityIndex`, and submit every existing 2,048-package Rayon chunk through
one `read_packages` call.

Repeat the native 182-tile matrix at 1, 4, and 24 workers with three fresh
process repetitions per cell under the existing 28 GiB cgroup. Record:

- materialization and total pipeline time;
- package throughput and observed worker count;
- process peak RSS and cgroup peak;
- configured, checked-out, idle, and peak concurrent connection counts;
- checkout wait count and cumulative wait time;
- shared metadata cache entries and bytes;
- persistent vertex-store counters defined by ADR 012.

Compare the results with both the original thread-local profile and the chosen
vertex-store baseline. Correctness, the configured connection bound, and the
absence of worker-local source vertex arrays are mandatory. Throughput and
memory results are presented for review without an automatic acceptance
threshold.

## Consequences

### Positive

- Rust callers can share one index directly across worker threads.
- SQLite connection count and checkout waiting are bounded and observable.
- Batch operations amortize one checkout and one coherent read transaction.
- Metadata and immutable backend state are shared once per logical index.
- Reindexing has explicit in-process exclusion semantics.
- The API remains independent of any scheduling library.

### Negative

- `CityIndex` ownership and `reindex` semantics change intentionally.
- The crate gains a pool dependency and more lifecycle state.
- Every public read path must be refactored around connection-scoped helpers.
- A checked-out connection remains occupied during package source I/O and model
  reconstruction, so pool sizing affects both SQLite and decoding concurrency.
- Cross-process source mutation remains outside the lifecycle gate.

### Neutral tradeoff

The pool bounds complete read operations rather than SQL statements alone.
This holds a connection longer, but gives each operation a coherent snapshot,
avoids repeated checkout, and makes the configured bound describe actual
concurrent reconstruction work.

[tyler-profile]: ../../../../docs/benchmarks/tyler-profile-results-2026-08-03.md
