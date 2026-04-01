# Clippy Unresolved Cases And Fundamental Follow-Up

## Final Status

`just lint` passes cleanly.

No clippy warnings or errors remained unresolved at the end of this work, so there are no cases that still require an allow-attribute, suppression, or deferred fix.

## Unresolved Cases

None.

## Fundamental Follow-Up Proposals

Even though the lint set is clean, a few structural improvements would make these classes of warnings less likely to return:

1. Extract shared package read/write pipelines into common internal helpers.

The `src/package/*` and `cityparquet/src/package/*` modules had nearly identical control flow and were drifting into the same clippy failures independently.

Fundamental fix:
- Move the staged `core/geometry/semantic/appearance` orchestration into a shared internal abstraction.
- Keep only encoding-specific table IO in the backend crates.

2. Replace ad-hoc numeric narrowing with domain-specific checked conversion utilities.

Several lints came from repeated `usize -> u32/i32` and `f64 -> f32` conversions.

Fundamental fix:
- Keep all boundary/index conversions behind a small checked conversion module.
- Revisit whether color/appearance payloads should be modeled with the same precision on both sides of the conversion boundary to avoid workaround-style narrowing paths.

3. Introduce test fixture builders instead of large monolithic sample constructors.

The test suite accumulated large inline fixture functions that repeatedly triggered `too_many_lines`.

Fundamental fix:
- Add reusable fixture builders for metadata, geometry families, semantics, materials, textures, and package tables.
- Compose fixtures from those builders instead of hand-assembling full models in single functions.

4. Keep optional batch comparisons and byte-buffer utilities centralized.

Some warnings came from local utility patterns (`&Option<T>`, large stack buffers) rather than domain logic.

Fundamental fix:
- Maintain a single test-support API for optional batch assertions and file comparison.
- Prefer heap-backed reusable buffers for large comparisons by default.
