# FFI Performance Analysis

This note explains why the current end-to-end non-Rust benchmark results are so
much worse than the Rust baseline.

The short answer is: the problem is not primarily the raw ABI crossing cost.
The current numbers mostly capture wrapper-side work layered around the ABI.

## What The Benchmark Is Measuring

The benchmark compares:

- direct Rust API calls
- Python over the shared C ABI
- C++ over the shared C ABI
- wasm through the wasm wrapper plus the Node host boundary

That means the reported slowdown includes:

- parsing and serialization work
- extra validation and probing
- allocation and ownership transfer
- byte copying
- string encoding and decoding
- host-language object construction

It is therefore incorrect to read the current ratios as "an FFI call costs 10x
to 30x more than a Rust call".

## Main Causes

### 1. Parse Entry Points Probe Before They Parse

The shared C ABI parse path does an explicit probe before the full parse.

In [exports.rs](/home/balazs/Development/cityjson-lib/ffi/core/src/exports.rs#L735),
`cj_model_parse_document_bytes`:

- calls `cityjson_lib::json::probe(input)`
- validates root kind and version
- then calls `cityjson_lib::json::from_slice(input)`

The same shape exists for feature parsing.

By contrast, the Rust benchmark baseline for `summary` and `roundtrip` goes
straight to `CityModel::from_slice(...)` in
[main.rs](/home/balazs/Development/cityjson-lib-benchmarks/drivers/rust/src/main.rs#L162).

That means every FFI parse currently pays for an extra front pass over the
input before the real parse starts.

### 2. The ABI Shape Forces Bulk Copying

The low-level ABI is intentionally simple and stable:

- bytes in
- opaque handle out
- bytes back out for serialization

That shape is reasonable, but it means serialized payloads are copied into
ABI-owned buffers.

In [handle.rs](/home/balazs/Development/cityjson-lib/ffi/core/src/handle.rs#L32),
`bytes_from_vec` turns a Rust `Vec<u8>` into owned ABI memory.

The consumer then copies those bytes again into host-language memory before
freeing the ABI buffer:

- C++ copies in [cityjson_lib.hpp](/home/balazs/Development/cityjson-lib/ffi/cpp/include/cityjson_lib/cityjson_lib.hpp#L119)
- Python copies in [_ffi.py](/home/balazs/Development/cityjson-lib/ffi/python/src/cityjson_lib/_ffi.py#L471)

So the path is not "serialize once and hand out a borrow". It is "serialize,
allocate, copy out, free".

### 3. Python Adds Multiple Expensive Copies

Python is the worst case because the current `ctypes` bridge adds several extra
copies and conversions.

Input copy:

- `_data_pointer` uses `from_buffer_copy` in
  [_ffi.py](/home/balazs/Development/cityjson-lib/ffi/python/src/cityjson_lib/_ffi.py#L420)

Output copy:

- `_take_bytes` reconstructs the payload one element at a time in
  [_ffi.py](/home/balazs/Development/cityjson-lib/ffi/python/src/cityjson_lib/_ffi.py#L471)

String conversion:

- `serialize_document()` returns decoded `str` in
  [__init__.py](/home/balazs/Development/cityjson-lib/ffi/python/src/cityjson_lib/__init__.py#L299)

Benchmark conversion back to bytes:

- the benchmark immediately `.encode("utf-8")`s that string in
  [benchmark.py](/home/balazs/Development/cityjson-lib-benchmarks/drivers/python/benchmark.py#L167)

That means the roundtrip path effectively does:

- Rust bytes
- ABI buffer
- Python bytes
- Python string
- Python bytes again

This is the main reason the Python wrapper is dramatically slower than both
Rust and wasm.

### 4. C++ Is Better Than Python, But Still Pays Ownership Churn

C++ avoids Python interpreter overhead, but it still pays unnecessary copying on
large serialized outputs.

`serialize_document()` returns `std::string` in
[cityjson_lib.hpp](/home/balazs/Development/cityjson-lib/ffi/cpp/include/cityjson_lib/cityjson_lib.hpp#L378),
which copies the ABI buffer into a C++ string.

The benchmark then copies that string again into `output_payload` in
[main.cpp](/home/balazs/Development/cityjson-lib-benchmarks/drivers/cpp/main.cpp#L435).

C++ also pays the same explicit `probe + parse` cost that the other shared-ABI
targets pay.

So the C++ result does not mean the ABI crossing alone is roughly `7x` to
`18x`. It means the full wrapper path, including buffer ownership churn, is
that much slower.

### 5. The Wasm Result Shows The Boundary Itself Is Not The Disaster

The wasm benchmark is much closer to Rust than Python or C++ in the current
run.

This is important because it shows the general idea of crossing a non-Rust
boundary is not itself the dominant problem.

Even the wasm benchmark still performs wrapper-side work:

- Rust serializes summary data to JSON strings in
  [lib.rs](/home/balazs/Development/cityjson-lib-benchmarks/drivers/wasm/src/lib.rs#L55)
- Node parses those JSON strings in
  [run_wasm_benchmark.cjs](/home/balazs/Development/cityjson-lib-benchmarks/drivers/wasm/run_wasm_benchmark.cjs#L62)

Despite that, the wasm overhead stays far below Python and below the current
C++ wrapper on the large end-to-end cases.

That is a strong signal that the real problem is wrapper design, not merely
"FFI exists".

## Conclusion

The current FFI performance is poor because the bindings prioritize a narrow,
stable, easy-to-own ABI over zero-copy throughput.

The most important contributors are:

- redundant `probe` before `parse`
- bytes-in / bytes-out ownership transfers for large payloads
- extra copies in the C++ wrapper
- severe copy and text-conversion overhead in the Python `ctypes` wrapper
- benchmark adapter work above the raw library call

The current benchmark is therefore doing exactly what it should do: it is
showing the cost of the real public wrapper stacks, not just the theoretical
cost of a foreign call instruction.

## Highest-Value Follow-Ups

If the goal is to materially reduce the measured overhead, the best next steps
are:

1. add a trusted parse fast path that skips the explicit `probe` when the caller
   does not need it
2. expose serialized document output as raw bytes in non-Rust bindings instead
   of eagerly converting to host-language text
3. replace the Python `ctypes` bulk-copy paths with contiguous buffer handling
   instead of per-element reconstruction
4. add a dedicated microbenchmark for the low-level C ABI so the raw boundary
   cost can be separated from wrapper API cost
