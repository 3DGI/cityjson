import json
import subprocess
from dataclasses import dataclass, field, asdict
from pathlib import Path


@dataclass
class CityObject:
    type: str = "Building"


@dataclass
class Boundary:
    inner: list[int] = field(default_factory=list)


@dataclass
class CityModel:
    version: str = "2.0"
    CityObjects: dict[str, CityObject] = field(default_factory=dict)
    geometry: list[Boundary] = field(default_factory=list)
    attributes: dict[str, str] = field(default_factory=dict)


def data_directory() -> Path:
    """Get the Cargo workspace root from the Cargo metadata"""
    res = subprocess.run(["cargo", "metadata", "--format-version", "1"], capture_output=True)
    metadata = json.loads(res.stdout.decode("utf-8"))
    return Path(metadata["workspace_root"]) / "tests" / "data"


def generate_objects(data_directory: Path):
    cm = CityModel(CityObjects={f"NL.IMBAG.Pand.0000000000000000-{i}": CityObject() for i in range(10000)})
    with data_directory.joinpath("objects.city.json").open("w") as fo:
        json.dump(asdict(cm), fo)


def generate_geometry(data_directory: Path):
    cm = CityModel(geometry=[Boundary(inner=[i for i in range(300)]) for j in range(10000)])
    with data_directory.joinpath("geometries.city.json").open("w") as fo:
        json.dump(asdict(cm), fo)


def generate_attributes(data_directory: Path):
    cm = CityModel(attributes={f"some_long_attribute-{i}": f"value-{i}" for i in range(100000)})
    with data_directory.joinpath("attributes.city.json").open("w") as fo:
        json.dump(asdict(cm), fo)


def generate_strings(data_directory: Path):
    pass


def generate_number_array(data_directory: Path):
    pass


if __name__ == "__main__":
    data_dir = data_directory()
    data_dir.mkdir(exist_ok=True)
    generate_attributes(data_dir)
    generate_geometry(data_dir)
    generate_objects(data_dir)
    generate_number_array(data_dir)
    generate_strings(data_dir)
