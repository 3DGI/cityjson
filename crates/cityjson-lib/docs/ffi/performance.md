# FFI Performance Visibility

The FFI benchmark suite compares the raw ABI against the Rust, Python, and C++
wrappers on the same fixtures.

## Run It

```bash
just ffi bench quick
```

Use `full` instead of `quick` when you want a longer run:

```bash
just ffi bench full
```

## What It Measures

- `parse`
- `serialize`
- `cityobject_ids`
- `geometry_types`
- `append`

Each benchmark is run against a small fixture and a medium fixture.

## Outputs

The runner writes JSON snapshots under `target/ffi-bench/results/`:

- `rust.json`
- `python.json`
- `cpp.json`

These are local artifacts, not committed history.

## How To Use It

Look for places where the wrapper layer is meaningfully slower than the raw ABI,
especially on bulk inspection and append. If those gaps remain large after the
current bulk reads, that is the signal to add more bulk APIs on the next pass.
