# ADR 0008: Wasm32 Blocker And The Wasm64 Path

## Status

Proposed.

## Context

The benchmark work has reached a real upstream limit: `cityjson-rs` currently
hard-fails on non-64-bit targets in `src/backend/default/vertex.rs`
with:

```rust
compile_error!("This crate only supports 64-bit platforms");
```

That failure is useful signal, not just a benchmark inconvenience. It means the
current `cjlib` wasm story is not blocked by wrappers alone; the semantic core
still assumes a 64-bit host in at least one low-level geometry/indexing path.

At the same time, the platform picture has moved in three different ways:

- The WebAssembly project published Wasm 3.0 on September 17, 2025, and that
  release includes 64-bit memories and tables.
- The same WebAssembly update notes that the web embedding still limits a
  64-bit memory to 16 GiB, so the biggest wins from wasm64 are in non-web
  runtimes and unusually large data sets.
- As of March 30, 2026, Rust still documents `wasm64-unknown-unknown` as a
  Tier 3 target, says testing is not run for it, does not ship precompiled
  artifacts for it, and says there are no known toolchains for mixing Rust and
  C on that target.
- Rust also spent 2025 tightening the `wasm32-unknown-unknown` C ABI story.
  That is a reminder that our benchmark target should be the real boundary we
  intend to ship, not an accidental wrapper-specific surrogate.

That means we should not treat wasm64 as a near-term replacement for wasm32, but
we also should not ignore it. For large CityJSON datasets, 64-bit addressing may
eventually become the better fit once the portability work lands.

There is also a `cjlib`-specific reason not to conflate wasm32 portability with
wasm64 support: the shared FFI core already uses `usize`/`uintptr_t` heavily in
its ABI for lengths, indices, and counts. wasm64 is therefore interesting for
ABI alignment inside Rust and C-like layers, but it still does not make the
JavaScript boundary, browser packaging path, or `cityjson-rs` pointer-width
assumptions disappear.

## Decision

`cjlib` should treat wasm portability as a staged effort:

1. Remove the 64-bit platform guard in `cityjson-rs`.
2. Make vertex and boundary storage paths compile and test on wasm32.
3. Keep the wasm-facing `cjlib` adapter narrow and benchmark the real wasm32
   boundary once the core builds there.
4. Evaluate wasm64 only after the wasm32 baseline is green and only if the
   dataset size or memory pressure makes it materially useful.

The first portability pass should focus on the code that currently assumes
`usize`/pointer-width friendliness:

- `vertex.rs`
- any storage or conversion code that converts vertex indices through `usize`
- any bulk-copy or buffer export logic that depends on 64-bit host assumptions
- the `cjlib-ffi-core` ABI surfaces that currently expose pointer-sized index
  and length fields

The type system being parameterizable is an advantage, but it is not enough by
itself. The portability work needs explicit audit and tests around the concrete
index and allocation boundaries.

## Concrete Plan

### Phase 1: wasm32 portability in `cityjson-rs`

- Remove the unconditional `compile_error!` gate.
- Audit `VertexIndex` and related conversion paths for `usize` assumptions.
- Replace non-portable conversion paths with checked conversions or
  pointer-width-neutral storage where necessary.
- Add a wasm32-oriented CI or local verification path for the affected crate
  surface.

### Phase 2: benchmark the real wasm32 boundary

- Repoint the benchmark worktree from the native wasm fallback to a true
  wasm32 artifact.
- Measure parse, summary, and roundtrip costs again so the benchmark captures
  the actual JS or host boundary cost rather than a host-side adapter surrogate.

### Phase 3: decide whether wasm64 is worth a first-class target

- Keep wasm64 as a research and scaling path, not the default compatibility
  target.
- Reassess if current datasets, future generated cases, or browser/runtime
  support make 64-bit memory materially better than wasm32.
- Treat wasm64 as more plausible for non-web runtimes or internal host-side
  adapters than as the first browser target.
- Do not require wasm64 for the first portable wasm32 milestone.

## Consequences

Positive:

- the blocker is documented where the architecture decisions live
- the plan separates core portability work from wrapper work
- wasm64 stays on the table without obscuring the nearer-term wasm32 goal
- benchmark results will eventually reflect the real wasm boundary instead of a
  fallback path

Tradeoffs:

- the first portability pass may touch low-level storage code in `cityjson-rs`
- a later wasm64 path may still require ABI review in `cjlib-ffi-core`, even if
  `cityjson-rs` becomes pointer-width-neutral
- wasm64 remains a moving target and should not be used as the immediate
  production baseline
- the benchmark repo still needs a follow-up once the core can actually compile
  to wasm32

## References

- WebAssembly, "Wasm 3.0 Completed", September 17, 2025:
  https://webassembly.org/news/2025-09-17-wasm-3.0/
- Rust target support, `wasm64-unknown-unknown`, checked on March 30, 2026:
  https://doc.rust-lang.org/beta/rustc/platform-support/wasm64-unknown-unknown.html
- Rust Blog, "C ABI Changes for `wasm32-unknown-unknown`", April 4, 2025:
  https://blog.rust-lang.org/2025/04/04/c-abi-changes-for-wasm32-unknown-unknown/
