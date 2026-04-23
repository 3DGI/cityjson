"""Create a benchmark index for local CityJSON benchmark inputs."""
# /// script
# requires-python = ">=3.12"
# ///

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT_PATH = REPO_ROOT / "target" / "bench-local" / "benchmark-index.json"
JSON_SUFFIX = ".json"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "inputs",
        nargs="+",
        help="One or more CityJSON files or directories containing CityJSON files",
    )
    parser.add_argument(
        "--output",
        default=str(DEFAULT_OUTPUT_PATH),
        help="Output path for the generated benchmark index",
    )
    return parser.parse_args()


def collect_input_files(raw_inputs: list[str]) -> list[Path]:
    input_files: list[Path] = []
    for raw_input in raw_inputs:
        input_path = Path(raw_input)
        if not input_path.exists():
            raise SystemExit(f"benchmark input does not exist: {input_path}")
        if input_path.is_file():
            input_files.append(input_path.resolve())
            continue
        if not input_path.is_dir():
            raise SystemExit(f"benchmark input is neither a file nor a directory: {input_path}")
        input_files.extend(iter_directory_files(input_path))

    unique_files = sorted({path.resolve() for path in input_files}, key=lambda path: str(path))
    if not unique_files:
        raise SystemExit("no JSON files found in the supplied benchmark inputs")
    return unique_files


def iter_directory_files(directory: Path) -> list[Path]:
    return sorted(
        (
            path.resolve()
            for path in directory.rglob(f"*{JSON_SUFFIX}")
            if path.is_file()
        ),
        key=lambda path: str(path),
    )


def case_identifier(path: Path, used_ids: set[str]) -> str:
    stem = path.name.removesuffix(".city.json").removesuffix(".json")
    identifier = re.sub(r"[^0-9A-Za-z]+", "_", stem).strip("_").lower()
    if not identifier:
        identifier = "case"

    candidate = identifier
    suffix = 2
    while candidate in used_ids:
        candidate = f"{identifier}_{suffix}"
        suffix += 1

    used_ids.add(candidate)
    return candidate


def display_path(path: Path) -> str:
    current_directory = Path.cwd()
    if path.is_relative_to(current_directory):
        return str(path.relative_to(current_directory))
    return str(path)


def benchmark_case(path: Path, used_ids: set[str]) -> dict[str, object]:
    return {
        "id": case_identifier(path, used_ids),
        "layer": "local",
        "representation": "cityjson",
        "description": display_path(path),
        "artifacts": [
            {
                "representation": "cityjson",
                "path": str(path),
            }
        ],
    }


def write_index(cases: list[dict[str, object]], output_path: Path) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "generated_cases": [],
        "other_cases": cases,
    }
    output_path.write_text(
        json.dumps(payload, indent=2) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    args = parse_args()
    input_files = collect_input_files(args.inputs)
    used_ids: set[str] = set()
    cases = [benchmark_case(path, used_ids) for path in input_files]
    output_path = Path(args.output)
    if not output_path.is_absolute():
        output_path = (Path.cwd() / output_path).resolve()
    write_index(cases, output_path)
    print(f"Wrote {len(cases)} benchmark cases to {output_path}")


if __name__ == "__main__":
    main()
