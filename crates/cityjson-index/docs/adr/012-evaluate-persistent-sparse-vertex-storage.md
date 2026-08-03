# ADR 012: Evaluate Persistent Sparse Vertex Storage

- Status: Proposed
- Date: 2026-08-03

## Context

Regular CityJSON sources store one shared `vertices` array for every
`CityObject` in the source. The normalized index stores byte ranges for
individual `CityObject` fragments, but package reconstruction still reads and
parses the complete source-level vertices array. `CityJsonBackend` then retains
the resulting `Vec<[i64; 3]>` in an unbounded process-local cache.

The [Tyler profiling results][tyler-profile] establish both consequences on the
182-tile Groningen corpus:

- the one-worker run retained all 182 source arrays;
- the corpus contains 111,713,328 source vertices;
- one complete set of source arrays requested 3,862,953,984 bytes of vector
  capacity;
- 24 worker-local indexes retained 1,039 source copies and 22.44 GiB of vertex
  capacity;
- dropping those caches released most of the resident memory.

The source corpus is 8,996,367,140 bytes. Its current schema-v2 sidecar is
638,263,296 bytes, and the 182 JSON `vertices` fragments occupy 2,928,367,534
bytes. Loading a package must instead allocate only the vertices referenced by
that package, or by the explicit batch containing it.

This ADR evaluates three persistent representations that make a source vertex
addressable by its original zero-based CityJSON index. It deliberately does not
select one yet. The choice will be made after running the experiments below and
adding their results to this record.

The public concurrent-read architecture is related but separate. Connection
pooling, shared reader state, and making `CityIndex` cloneable, `Send`, and
`Sync` will be specified in a follow-up ADR after the storage measurements are
available. Keeping that decision separate lets this experiment attribute
storage and reconstruction costs without also changing connection ownership.

## Common Requirements

Every candidate must satisfy the following requirements:

- coordinates remain exact signed 64-bit integers;
- the original CityJSON vertex index remains the lookup key;
- a single-package read never materializes a complete source vertex array;
- `read_packages` may deduplicate vertices within that call, but no candidate
  may retain an unbounded decoded-vertex cache after the call;
- malformed or truncated persistent data produces an error rather than a
  partial model;
- empty source arrays require no chunk rows;
- source freshness checks continue to protect any representation that depends
  on source byte positions;
- schema-v2 sidecars are marked as requiring a full reindex rather than being
  populated by an in-place migration;
- candidate sidecars use distinct paths and record the selected strategy in
  their profiling provenance.

All integer fields inside binary blobs use little-endian byte order. The DDL
below shows the candidate-specific addition to the existing normalized schema.
The existing `sources`, `packages`, `cityobjects`, relationship, membership,
and RTree tables remain unchanged during the experiment.

## Option A: Packed Coordinate Chunks

### Representation

Store every source coordinate once as three consecutive little-endian `i64`
values. Split each source into chunks of at most 16,384 vertices so no SQLite
BLOB operation or allocation depends on total source size.

```sql
CREATE TABLE source_vertex_chunks (
    id             INTEGER PRIMARY KEY,
    source_id      INTEGER NOT NULL
                         REFERENCES sources(id) ON DELETE CASCADE,
    chunk_ordinal  INTEGER NOT NULL CHECK (chunk_ordinal >= 0),
    first_vertex   INTEGER NOT NULL CHECK (first_vertex >= 0),
    vertex_count   INTEGER NOT NULL
                         CHECK (vertex_count BETWEEN 1 AND 16384),
    payload        BLOB NOT NULL,
    UNIQUE (source_id, chunk_ordinal),
    UNIQUE (source_id, first_vertex),
    CHECK (first_vertex = chunk_ordinal * 16384),
    CHECK (length(payload) = vertex_count * 24)
);
```

The row is found from `vertex_index / 16384`. The byte position within its BLOB
is `(vertex_index % 16384) * 24`. SQLite incremental BLOB I/O reads only the
requested records. Adjacent indices can be coalesced into one positional read.

### Expected properties

This is the simplest binary read path. Coordinates are parsed once during
reindexing, reads use constant-time arithmetic, and no general-purpose decoder
is needed. The cost is a second, nearly full-size representation of the source
vertices.

For Groningen-182, the exact coordinate payload is:

```text
111,713,328 vertices * 24 bytes = 2,681,119,872 bytes (2.497 GiB)
```

Adding that payload to the current sidecar gives a pre-overhead estimate of
3.091 GiB, about 36.9% of the source corpus and 5.2 times the current sidecar.

## Option B: Compact JSON Offsets

### Representation

Keep coordinates only in the authoritative CityJSON source. Persist the start
of each JSON vertex value as a chunk-relative `u32`. A final sentinel bounds the
last value in the chunk. Each chunk covers at most 16,384 vertices.

```sql
CREATE TABLE source_vertex_offset_chunks (
    id                  INTEGER PRIMARY KEY,
    source_id           INTEGER NOT NULL
                              REFERENCES sources(id) ON DELETE CASCADE,
    chunk_ordinal       INTEGER NOT NULL CHECK (chunk_ordinal >= 0),
    first_vertex        INTEGER NOT NULL CHECK (first_vertex >= 0),
    vertex_count        INTEGER NOT NULL
                              CHECK (vertex_count BETWEEN 1 AND 16384),
    source_base_offset  INTEGER NOT NULL CHECK (source_base_offset >= 0),
    offsets             BLOB NOT NULL,
    UNIQUE (source_id, chunk_ordinal),
    UNIQUE (source_id, first_vertex),
    CHECK (first_vertex = chunk_ordinal * 16384),
    CHECK (length(offsets) = (vertex_count + 1) * 4)
);
```

The `offsets` BLOB contains monotonically increasing little-endian `u32`
values. Entry zero is zero. Entries `0..vertex_count` locate vertex starts
relative to `source_base_offset`; the final entry ends the last searchable
range. Reindexing rejects a chunk whose source span exceeds `u32::MAX`.

For every requested vertex, the reader loads the corresponding start and next
offset. It finds the exact end of the JSON value within that bounded slice,
then parses one `[i64; 3]`. Consecutive requested indices are coalesced into a
single source-file read before their individual values are parsed.

### Expected properties

This option minimizes duplicated coordinate data and keeps SQLite writes small.
It adds source-file seeks, bounded JSON scanning, and integer parsing to every
package reconstruction. Repeated packages and repeated corpus scans repeat
that work. The kernel page cache is shared across readers, but parsed
coordinates are not.

The Groningen offset payload is bounded by approximately 426.16 MiB, including
one sentinel per chunk. Adding it to the current sidecar gives a pre-overhead
estimate of 1.011 GiB, about 12.1% of the source corpus and 1.7 times the current
sidecar. A simpler absolute-`u64` offset array would instead add approximately
852.3 MiB and is not part of the experiment.

## Option C: Frame-of-Reference Bit Packing

### Representation

Store exact coordinates in 16,384-vertex SQLite superchunks. Divide each
superchunk into independently encoded subblocks of at most 128 vertices. For
each axis in a subblock, store the minimum signed coordinate and the number of
bits required for the unsigned difference from that minimum.

```sql
CREATE TABLE source_vertex_superchunks (
    id             INTEGER PRIMARY KEY,
    source_id      INTEGER NOT NULL
                         REFERENCES sources(id) ON DELETE CASCADE,
    chunk_ordinal  INTEGER NOT NULL CHECK (chunk_ordinal >= 0),
    first_vertex   INTEGER NOT NULL CHECK (first_vertex >= 0),
    vertex_count   INTEGER NOT NULL
                         CHECK (vertex_count BETWEEN 1 AND 16384),
    header         BLOB NOT NULL,
    payload        BLOB NOT NULL,
    UNIQUE (source_id, chunk_ordinal),
    UNIQUE (source_id, first_vertex),
    CHECK (first_vertex = chunk_ordinal * 16384),
    CHECK (
        length(header) = ((vertex_count + 127) / 128) * 27
    )
);
```

Each 27-byte subblock descriptor contains, in order:

1. `min_x`, `min_y`, and `min_z` as three little-endian `i64` values;
2. `bits_x`, `bits_y`, and `bits_z` as three `u8` values in the range `0..=64`.

The payload stores X, Y, and Z differences in that order for every vertex.
Fields and bytes use least-significant-bit-first order. Every subblock starts on
a byte boundary, so padding is limited to seven bits per subblock. A zero-width
axis has the same value for every vertex and consumes no payload bits.

Encoding and decoding use checked `i128` subtraction and addition. This
supports the complete `i64` coordinate domain: the difference between the
minimum and maximum signed values fits in `u64`, even when it does not fit in
`i64`. Open-time and read-time validation checks header widths, the computed
payload length, chunk bounds, and decoded coordinate conversion.

The reader calculates the superchunk and subblock from the original vertex
index, reads the small header, and uses SQLite incremental BLOB I/O for only
the touched encoded subblock. Batch reconstruction reads each touched subblock
once and decodes only the indices requested from it. No decompressed subblock
survives the batch.

### Measured size model

A read-only scan of all 111,713,328 Groningen vertices measured the following
frame-of-reference payloads before per-subblock headers:

| Subblock size | Average bits/vertex | Coordinate payload |
|---:|---:|---:|
| 32 | 55.42 | 737.99 MiB |
| 64 | 56.79 | 756.26 MiB |
| 128 | 57.90 | 771.05 MiB |
| 256 | 58.88 | 784.07 MiB |
| 1,024 | 60.73 | 808.76 MiB |

At 128 vertices, compact headers add approximately 22.48 MiB and byte alignment
adds no more than 0.73 MiB. The estimated vertex storage is therefore about
794.3 MiB. Adding it to the current sidecar gives a pre-overhead estimate of
approximately 1.370 GiB, about 16.3% of the source corpus and 2.3 times the
current sidecar.

The corpus coordinate extrema used to validate the calculation are
`[0, 0, 0]` and `[76,541,370, 74,012,340, 214,748,607,000]`. The large Z value
is one reason the representation must not narrow coordinates globally.

## Shared Experimental Implementation

The experiment will define an internal `VertexStore` boundary used only by the
benchmark branch. Its three implementations must share package preparation and
model assembly code so the measured variable is persistent vertex lookup, not
different JSON reconstruction logic.

`read_packages` becomes the representative batch path for all candidates:

1. load and validate all indexed CityObject fragments for the requested
   packages;
2. collect, sort, and deduplicate `(source_id, vertex_index)` requirements for
   the complete input batch;
3. ask the selected vertex store for those exact coordinates;
4. remap package boundaries and assemble results in original request order;
5. drop all batch-local coordinate and encoded buffers before returning.

`read_package` invokes the same path with a one-item slice. The experiment must
not add a persistent decoded-coordinate cache to any option. Candidate strategy
names and sidecar paths are benchmark inputs and are recorded in every result
artifact; they are not proposed public API.

## Experiment 1: Correctness and Storage

Build one fresh candidate sidecar per representation from the same immutable
Groningen-182 sources. Before timing comparisons:

- run the existing regular-CityJSON reconstruction and schema tests against
  every candidate;
- verify source, package, CityObject, membership, and relationship counts are
  identical;
- compare reconstructed package bytes or deterministic SHA-256 digests in
  package-record order;
- directly compare first, last, subblock-boundary, chunk-boundary, and selected
  extrema vertices from every source with their source JSON values;
- reject a candidate on any coordinate, remap, relationship, or package-output
  mismatch.

For each successful sidecar, record:

- complete sidecar byte size;
- byte size by table from SQLite `dbstat`;
- source vertex count and encoded payload bytes;
- chunk, superchunk, or offset-chunk row count;
- observed bytes per source vertex.

## Experiment 2: Reindex Cost

Run three fresh-process reindexes for every candidate with four indexing
workers. Four workers are the best measured throughput/memory balance in the
current Tyler campaign. Use the same 28 GiB cgroup limit and disable swap, as
specified by [ADR 011](011-tiered-tyler-profiling.md).

Record median and range for:

- elapsed reindex time;
- package and vertex throughput;
- process peak RSS and cgroup peak;
- bytes read from sources and bytes written to the sidecar;
- final sidecar size.

The retained comparison sidecar for each option is created outside all read
experiments. A measured read process must fail rather than create, delete, or
rebuild its sidecar.

## Experiment 3: Read Latency and Batching

Create a deterministic, source-stratified sample covering all 182 sources and
10,000 package references. Include packages that touch the beginning and end of
vertex chunks and packages from the largest source files.

Measure in fresh processes:

- one-worker singleton `read_package` calls;
- one-worker `read_packages` calls in batches of 2,048;
- a first pass and an immediately repeated pass, reported separately rather
  than labelled as controlled cold and warm cache states.

Record package latency distributions, package throughput, peak memory,
requested and unique vertex counts, persistent bytes read, touched chunk or
subblock counts, and source JSON bytes read. Do not drop the host filesystem
cache or require root privileges; rotate candidate order between repetitions
to reduce cache-order bias.

## Experiment 4: Tyler Full-Corpus Materialization

Update the Tyler simulation to pass each existing 2,048-package Rayon chunk to
one `read_packages` call. Run the complete 707,239-package materialization at
1, 4, and 24 workers for every candidate. Use three native repetitions per
cell, a fresh process for every repetition, and the existing 28 GiB cgroup
containment and 100 ms memory sampling.

Rotate candidate order between repetitions. Record median and range for:

- materialization time and packages per second;
- total pipeline time;
- process peak RSS and cgroup peak;
- requested, unique, and returned vertex counts;
- persistent vertex bytes read;
- chunks or subblocks touched;
- source JSON bytes read;
- retained vertex-store heap after materialization.

The initial bake-off uses native measurements only. `perf stat`, Heaptrack,
Cachegrind, and Massif are follow-up diagnostics when native results cannot
explain a material difference; they are not prerequisites for comparing every
candidate.

## Decision Process

Correctness and bounded memory are mandatory. Any candidate that returns a
different package or retains source-sized decoded arrays is rejected.

The ADR intentionally defines no automatic weighting between sidecar size,
reindex cost, singleton latency, batch throughput, and worker scaling. After
the result tables are populated, the maintainers will review the complete
measurements and amend this section with the selected representation and its
rationale. The selected design then receives its own implementation and
compatibility plan; the two unselected experimental implementations are
removed.

## Expected Consequences of the Experiment

- The persistent-format decision will be supported by exact corpus storage and
  end-to-end read measurements rather than estimates alone.
- All candidates will exercise one common batch reconstruction path, which is
  reusable by the later concurrent API work.
- The experiment requires temporary code for three storage implementations and
  three incompatible candidate sidecars.
- No candidate format or benchmark selector becomes public API merely because
  it participates in the bake-off.

[tyler-profile]: ../../../../docs/benchmarks/tyler-profile-results-2026-08-03.md
