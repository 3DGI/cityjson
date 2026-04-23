#!/usr/bin/env python3
from __future__ import annotations

import argparse
import gc
import os
import json
from pathlib import Path
from statistics import median
from time import perf_counter_ns

REPO_ROOT = Path(__file__).resolve().parents[3]
RELEASE_LIB = REPO_ROOT / "target" / "release" / "libcityjson_lib_ffi_core.so"
if RELEASE_LIB.exists():
    os.environ.setdefault("CITYJSON_LIB_FFI_CORE_LIB", str(RELEASE_LIB))

from cityjson_lib import CityModel, ModelType, Transform, WriteOptions
from cityjson_lib._ffi import FfiLibrary


FIXTURES = (
    ("small", Path(__file__).resolve().parents[3] / "tests" / "data" / "v2_0" / "minimal.city.json"),
    (
        "medium",
        Path(__file__).resolve().parents[3]
        / "tests"
        / "data"
        / "v2_0"
        / "cityjson_fake_complete.city.json",
    ),
)
def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quick", action="store_true", help="use the short benchmark preset")
    return parser.parse_args()


def mode_config(quick: bool) -> dict[str, int | str]:
    if quick:
        return {"label": "quick", "iterations": 250, "repeats": 7, "append_repeats": 11}
    return {"label": "full", "iterations": 1000, "repeats": 9, "append_repeats": 15}


def measure_iterations(iterations: int, repeats: int, action) -> dict[str, float | int]:
    samples: list[int] = []
    for _ in range(repeats):
        started = perf_counter_ns()
        for _ in range(iterations):
            action()
        samples.append(perf_counter_ns() - started)

    elapsed = int(median(samples))
    return {
        "elapsed_ns": elapsed,
        "elapsed_per_iteration_ns": elapsed / iterations,
    }


def measure_append(repeats: int, setup, action, cleanup) -> dict[str, float | int]:
    samples: list[int] = []
    for _ in range(repeats):
        target, source = setup()
        try:
            started = perf_counter_ns()
            action(target, source)
            samples.append(perf_counter_ns() - started)
        finally:
            cleanup(target, source)

    elapsed = int(median(samples))
    return {
        "elapsed_ns": elapsed,
        "elapsed_per_iteration_ns": elapsed,
    }


def result(
    fixture: str,
    mode: str,
    operation: str,
    layer: str,
    iterations: int,
    repeats: int,
    timing: dict[str, float | int],
) -> dict[str, float | int | str]:
    return {
        "fixture": fixture,
        "mode": mode,
        "operation": operation,
        "layer": layer,
        "iterations": iterations,
        "repeats": repeats,
        "elapsed_ns": timing["elapsed_ns"],
        "elapsed_per_iteration_ns": timing["elapsed_per_iteration_ns"],
    }


def benchmark_fixture(ffi: FfiLibrary, fixture: str, payload: bytes, config: dict[str, int | str]) -> list[dict[str, float | int | str]]:
    iterations = int(config["iterations"])
    repeats = int(config["repeats"])
    append_repeats = int(config["append_repeats"])
    mode = str(config["label"])

    results: list[dict[str, float | int | str]] = []

    results.append(
        result(
            fixture,
            mode,
            "parse",
            "wrapper",
            iterations,
            repeats,
            measure_iterations(
                iterations,
                repeats,
                lambda: _parse_wrapper(payload),
            ),
        )
    )
    results.append(
        result(
            fixture,
            mode,
            "parse",
            "abi",
            iterations,
            repeats,
            measure_iterations(iterations, repeats, lambda: _parse_abi(ffi, payload)),
        )
    )

    wrapper_model = CityModel.parse_document_bytes(payload)
    raw_handle = ffi.parse_document(payload)
    serialize_options = WriteOptions(validate_default_themes=False)
    serialize_native = serialize_options.to_native()
    try:
        results.append(
            result(
                fixture,
                mode,
                "serialize",
                "wrapper",
                iterations,
                repeats,
                measure_iterations(
                    iterations,
                    repeats,
                    lambda: wrapper_model.serialize_document_bytes(serialize_options),
                ),
            )
        )
        results.append(
            result(
                fixture,
                mode,
                "serialize",
                "abi",
                iterations,
                repeats,
                measure_iterations(
                    iterations,
                    repeats,
                    lambda: ffi.serialize_document_with_options(raw_handle, serialize_native),
                ),
            )
        )

        results.append(
            result(
                fixture,
                mode,
                "cityobject_ids",
                "wrapper",
                iterations,
                repeats,
                measure_iterations(iterations, repeats, wrapper_model.cityobject_ids),
            )
        )
        results.append(
            result(
                fixture,
                mode,
                "cityobject_ids",
                "abi",
                iterations,
                repeats,
                measure_iterations(iterations, repeats, lambda: ffi.cityobject_ids(raw_handle)),
            )
        )

        results.append(
            result(
                fixture,
                mode,
                "geometry_types",
                "wrapper",
                iterations,
                repeats,
                measure_iterations(iterations, repeats, wrapper_model.geometry_types),
            )
        )
        results.append(
            result(
                fixture,
                mode,
                "geometry_types",
                "abi",
                iterations,
                repeats,
                measure_iterations(iterations, repeats, lambda: ffi.geometry_types(raw_handle)),
            )
        )
    finally:
        wrapper_model.close()
        ffi.free_model(raw_handle)

    return results


def benchmark_append(ffi: FfiLibrary, config: dict[str, int | str]) -> list[dict[str, float | int | str]]:
    payload_source = FIXTURES[0][1].read_bytes()
    fixture = "empty<-small"
    iterations = 1
    repeats = int(config["append_repeats"])
    mode = str(config["label"])
    transform = Transform(scale=(1.0, 1.0, 1.0), translate=(0.0, 0.0, 0.0))

    return [
        result(
            fixture,
            mode,
            "append",
            "wrapper",
            iterations,
            repeats,
            measure_append(
                repeats,
                lambda: _append_wrapper_setup(ffi, payload_source, transform),
                lambda target, source: target.append_model(source),
                lambda target, source: (target.close(), source.close()),
            ),
        ),
        result(
            fixture,
            mode,
            "append",
            "abi",
            iterations,
            repeats,
            measure_append(
                repeats,
                lambda: _append_abi_setup(ffi, payload_source, transform),
                lambda target, source: ffi.append_model(target, source),
                lambda target, source: (ffi.free_model(target), ffi.free_model(source)),
            ),
        ),
    ]


def main() -> None:
    args = parse_args()
    config = mode_config(args.quick)
    gc_was_enabled = gc.isenabled()
    gc.disable()
    try:
        ffi = FfiLibrary.load()
        results = []
        for fixture, path in FIXTURES:
            payload = path.read_bytes()
            results.extend(benchmark_fixture(ffi, fixture, payload, config))
        results.extend(benchmark_append(ffi, config))
    finally:
        if gc_was_enabled:
            gc.enable()

    print(
        json.dumps(
            {
                "language": "python",
                "mode": config["label"],
                "results": results,
            },
            indent=2,
        )
    )


def _parse_wrapper(payload: bytes) -> int:
    model = CityModel.parse_document_bytes(payload)
    try:
        return model.summary().cityobject_count
    finally:
        model.close()


def _parse_abi(ffi: FfiLibrary, payload: bytes) -> int:
    handle = ffi.parse_document(payload)
    try:
        return ffi.summary(handle).cityobject_count
    finally:
        ffi.free_model(handle)


def _append_wrapper_setup(
    ffi: FfiLibrary, payload_source: bytes, transform: Transform
) -> tuple[CityModel, CityModel]:
    target = CityModel.create(model_type=ModelType.CITY_JSON)
    source = CityModel.parse_document_bytes(payload_source)
    target.set_transform(transform)
    source.set_transform(transform)
    for cityobject_id in source.cityobject_ids():
        ffi.clear_cityobject_geometry(source._require_handle(), cityobject_id)
    return target, source


def _append_abi_setup(
    ffi: FfiLibrary, payload_source: bytes, transform: Transform
) -> tuple[int, int]:
    target = ffi.create(ModelType.CITY_JSON)
    source = ffi.parse_document(payload_source)
    ffi.set_transform(target, transform.to_native())
    ffi.set_transform(source, transform.to_native())
    for cityobject_id in ffi.cityobject_ids(source):
        ffi.clear_cityobject_geometry(source, cityobject_id)
    return target, source


if __name__ == "__main__":
    main()
