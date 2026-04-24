# Contributing

Thanks for your interest in the project.

Before you spend real time on a patch, please open an issue to discuss the
change.

## Basic guidelines

- **Keep PRs small and focused.** One topic per PR. If it's sprawling, I
  probably won't be able to review it in a reasonable time.
- **Match the style and conventions already in the crate you're editing.**
  The workspace is internally consistent on purpose.
- **New behaviour needs tests.** Bug fixes need a regression test that
  fails before the fix. Public API changes need doc updates.
- **`just ci` must pass** — that runs formatting, clippy, build, tests, and
  docs. If you're touching `cityjson-lib` or `cityjson-index`,
  `just test-python` should pass too.
- **If you remove or merge tests, examples, or benchmarks, say why in the
  PR.**

## Use of AI

LLM-assisted PRs are welcome for bug fixes, docs, test coverage, and
mechanical refactors. For anything larger — new features, architectural
changes, public API — please discuss it in an issue first, regardless of
whether the implementation will be hand-written or AI-assisted.

Parts of this workspace were themselves developed with AI assistance
(design exploration, refactors, test and benchmark scaffolding). Correctness
is guarded by a curated test suite and benchmarks that exercise the full
CityJSON 2.0 specification. That line of defence applies to your
contributions too: if the tests don't cover what you changed, extend them.

Please don't submit PRs that are clearly unreviewed LLM output (bogus
references, invented APIs, plausible-looking but wrong logic). Run the
tests locally first.

## Tooling and workflow detail

`docs/development.md` is the full contract: toolchain versions, MSRV,
Cargo metadata conventions, clippy/rustfmt/test flags, Python packaging,
justfile recipes, release flow. Start there if you're setting up a dev
environment or proposing a change that touches build config.

## Licensing

Contributions are dual-licensed under MIT or Apache-2.0, at the user's
option. The authoritative answer is always the `license` field in the
crate's `Cargo.toml` — if a crate ever relaxes to a single license, that's
where it'll be recorded.

Unless you say otherwise, any contribution you intentionally submit for
inclusion in this workspace is licensed under the same terms as the crate
you're contributing to, with no additional terms or conditions.
