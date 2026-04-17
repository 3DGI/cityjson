# Release Roadmap

This page is the historical record for how `cityjson-lib` was narrowed to a
publishable core.

## Current State

- `master` is publishable
- the public crate ships JSON, ops, and model boundaries only
- Arrow and Parquet work lives on the `arrow-transport` branch

## What This Page Is For

- background on the release split
- a reminder that transport work is separate
- a record of the earlier packaging and docs cleanup

## What Changed

- the root crate now packages and verifies cleanly
- the workspace no longer exposes transport features from the publishable crate
- the public docs now describe the core API instead of the transport experiments

## Next Release Chores

1. bump the version
2. publish the crate
3. tag the release
