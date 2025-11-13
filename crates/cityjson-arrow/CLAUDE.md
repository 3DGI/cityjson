# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CityArrow is a Rust library that converts between CityJSON models and Apache Arrow format. It provides bidirectional conversion capabilities, allowing CityJSON data to be stored and processed using Arrow's columnar format for better performance and interoperability.

## Key Commands

- **Build**: `cargo build`
- **Test**: `cargo test`
- **Test specific test**: `cargo test <testname>`
- **Check/lint**: `cargo check`
- **Format**: `cargo fmt`
- **Clippy**: `cargo clippy`

## Architecture

The codebase is organized around a clear separation of concerns:

### Core Modules
- **`lib.rs`**: Defines `CityModelArrowParts` struct and main conversion entry points
- **`reader.rs`**: Handles reading Arrow IPC files back into CityJSON models using `IPCBufferDecoder`
- **`writer.rs`**: Handles writing CityJSON models to Arrow format (IPC, Parquet, JSON) with `FileManifest` metadata
- **`error.rs`**: Custom error types and result handling
- **`conversion.rs`**: Entry point for conversion modules

### Conversion Modules (`src/conversion/`)
Each module handles conversion of specific CityJSON components to/from Arrow:
- **`geometry.rs`**: Converts geometries with support for boundaries, LoD, materials, textures
- **`cityobjects.rs`**: Handles city objects and their properties
- **`vertices.rs`**: Manages vertex data and coordinates
- **`attributes.rs`**: Processes object attributes and properties
- **`metadata.rs`**: Handles model metadata including geographical extent
- **`semantics.rs`**: Manages semantic information
- **`transform.rs`**: Handles coordinate transformations
- **`common.rs`**: Shared utilities for conversions

### Key Data Structures
- **`CityModelArrowParts`**: Central struct containing all Arrow RecordBatch components of a CityJSON model
- **`FileManifest`**: Metadata describing which components are present in Arrow files
- **`IPCBufferDecoder`**: Low-level Arrow IPC file decoder wrapper

## Dependencies

- **External CityJSON library**: Uses `cityjson = { path = "../cityjson-rs"}` - a local dependency
- **Arrow ecosystem**: `arrow`, `arrow-json`, `parquet` for columnar data operations
- **Serialization**: `nanoserde` for JSON manifest files
- **Memory mapping**: `memmap2` for efficient file reading

## Testing

Tests are located in `tests/` directory. The project uses standard Rust testing with `cargo test`. Test data would typically be in `tests/data/` but the directory is currently empty.

## Output Formats

The library supports multiple output formats:
- Arrow IPC files (`.arrow`)
- Parquet files (`.parquet`)
- Line-delimited JSON (`.ndjson`)
- Directory-based storage with manifest files

## Development Notes

- Uses Rust 2024 edition
- Leverages `lazy_static!` for shared field definitions in geometry conversions
- Heavy use of generics for `StringStorage` trait to support different string representations
- Error handling through custom `Result<T>` type aliased to `std::result::Result<T, Error>`