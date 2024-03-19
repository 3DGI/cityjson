import json
from pathlib import Path

with Path("/home/balazs/Development/serde_cityjson/resources/data/downloaded/30gz1_04.json").open("r") as fo:
    cm = json.load(fo)

boundaries = [geom["boundaries"] for coid, co in cm["CityObjects"].items() for geom in co["geometry"] if geom["type"] == "MultiSurface"]
print(f"boundaries len {len(boundaries)}")
with Path("/home/balazs/Development/bench_serde_json_nested_array/benches/data/boundaries.json").open("w") as fw:
    json.dump(boundaries, fw)

with Path("/home/balazs/Development/bench_serde_json_nested_array/benches/data/vertices.json").open("w") as fw:
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
with Path("/home/balazs/Development/bench_serde_json_nested_array/benches/data/all.json").open("w") as fw:
    json.dump({"vertices": cm["vertices"], "cityobjects": cityobjects}, fw)
