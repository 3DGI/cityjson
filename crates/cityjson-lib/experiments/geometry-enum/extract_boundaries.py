import json
from pathlib import Path

if __name__ == "__main__":
    with Path("../data/32cz1_04.json").open("r") as fo:
        cm = json.load(fo)

    Path("benches/data").mkdir(exist_ok=True)

    boundaries = [geom["boundaries"] for coid, co in cm["CityObjects"].items() for geom in co["geometry"] if
                  geom["type"] == "MultiSurface"]
    print(f"boundaries len {len(boundaries)}")
    with Path("benches/data/boundaries.json").open("w") as fw:
        json.dump(boundaries, fw)

    with Path("benches/data/vertices.json").open("w") as fw:
        json.dump(cm["vertices"], fw)

    cityobjects = {
        coid: {
            "geometry": [{
                "type": geom["type"],
                "boundaries": geom["boundaries"],
                "lod": geom["lod"]
            }, ]
        } for coid, co in cm["CityObjects"].items() for geom in co["geometry"] if geom["type"] == "MultiSurface"
    }
    with Path("benches/data/all.json").open("w") as fw:
        json.dump({"vertices": cm["vertices"], "cityobjects": cityobjects}, fw)
