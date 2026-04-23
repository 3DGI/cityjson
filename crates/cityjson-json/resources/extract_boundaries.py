import json
from pathlib import Path

repo_root = Path(__file__).resolve().parents[1]
input_path = repo_root / "resources" / "data" / "downloaded" / "30gz1_04.json"
output_root = repo_root.parent / "bench_serde_json_nested_array" / "benches" / "data"

with input_path.open("r") as fo:
    cm = json.load(fo)

boundaries = [geom["boundaries"] for coid, co in cm["CityObjects"].items() for geom in co["geometry"] if geom["type"] == "MultiSurface"]
print(f"boundaries len {len(boundaries)}")
with (output_root / "boundaries.json").open("w") as fw:
    json.dump(boundaries, fw)

with (output_root / "vertices.json").open("w") as fw:
    json.dump(cm["vertices"], fw)

cityobjects = {
    coid: {
        "geometry": [{
            "lod": geom["lod"],
            "type": geom["type"],
            "boundaries": geom["boundaries"],
        }, ]
    } for coid, co in cm["CityObjects"].items() for geom in co["geometry"] if geom["type"] == "MultiSurface"
}
with (output_root / "all.json").open("w") as fw:
    json.dump({"vertices": cm["vertices"], "cityobjects": cityobjects}, fw)
