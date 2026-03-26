"""Generate plots of relative performance differences for the 'speed' and 'datasize' benchmarks."""
# /// script
# requires-python = ">=3.12"
# dependencies = ["matplotlib"]
# ///
import json
import re
import subprocess
from pathlib import Path

import matplotlib.lines as mlines
import matplotlib.pyplot as plt


def cargo_target_directory() -> Path:
    """Get the Cargo target directory from the Cargo metadata"""
    res = subprocess.run(["cargo", "metadata", "--format-version", "1"], capture_output=True)
    metadata = json.loads(res.stdout.decode("utf-8"))
    return Path(metadata["target_directory"])


GROUPS = ("3DBAG", "3D Basisvoorziening")
OUTPUT_FIGS_DIR = cargo_target_directory().parent.joinpath("benches", "results")
OUTPUT_FIGS_DIR.mkdir(exist_ok=True, parents=True)
MARKER = "d"
MARKERSIZE = 60


def compare_criterion():
    # The serde_json::Value in the baseline for comparison that we compare serde_cityjson to
    benchmark_id_baseline = "serde_json::Value"
    benchmark_ids = ("serde_cityjson/owned", "serde_cityjson/borrowed")
    criterion_dir = cargo_target_directory().joinpath("criterion")
    # Contains the benchmark results. Note that the schema of this file is not stable.
    estimates_filename = "estimates.json"

    # {bench_id: [(group, relative_speed), ...]}
    results: dict[str, list[tuple[str, float]]] = {}
    for group in GROUPS:
        group_dir = criterion_dir.joinpath(group)
        # Get the latest run of the baseline benchmark (serde_json::Value)
        bench_baseline_dir_name = re.sub(r'[^a-zA-Z0-9 \n.]', '_', benchmark_id_baseline)
        bench_baseline_new_estimates_file = group_dir.joinpath(bench_baseline_dir_name, "new", estimates_filename)
        with bench_baseline_new_estimates_file.open("r") as fo:
            estimates_baseline = json.load(fo)
        for bench_id in benchmark_ids:
            bench_dir_name = re.sub(r'[^a-zA-Z0-9 \n.]', '_', bench_id)
            # The latest benchmark run
            bench_new_estimates_file = group_dir.joinpath(bench_dir_name, "new", estimates_filename)
            # borrowed mode is absent for datasets with JSON-escaped strings
            if not bench_new_estimates_file.exists():
                continue
            with bench_new_estimates_file.open("r") as fo:
                estimates = json.load(fo)
            speed_compared_to_baseline = estimates["mean"]["point_estimate"] / estimates_baseline["mean"][
                "point_estimate"]
            results.setdefault(bench_id, []).append((group, speed_compared_to_baseline))

    plt.style.use('seaborn-v0_8-muted')
    fig, ax = plt.subplots(figsize=(10, 3))
    for bench_id, points in results.items():
        groups, speeds = zip(*points)
        ax.scatter(speeds, groups, marker=MARKER, s=MARKERSIZE, label=bench_id)
    ax.vlines(x=1, ymin=0, ymax=1, transform=ax.get_xaxis_transform(), colors="r")
    red_line = mlines.Line2D([], [], color='red', label="serde_json::Value")
    series_handles, _ = ax.get_legend_handles_labels()
    ax.legend(handles=[red_line] + series_handles)
    ax.set_xlim(left=0.0)
    ax.grid(visible=True, which="major", axis="x")
    ax.set_title(f"Relative execution time of serde_cityjson compared to {benchmark_id_baseline}")
    ax.set_xlabel("Factor of execution time relative to serde_json::Value (>1 = slower)")
    plt.tight_layout()
    filepath = OUTPUT_FIGS_DIR.joinpath("speed_relative.png")
    plt.savefig(filepath)
    print(f"Saved {filepath}")


def compare_datasize():
    # The serde_json::Value in the baseline for comparison that we compare serde_cityjson to

    datasize_dir = cargo_target_directory().joinpath("serde_cityjson_datasize")
    # Contains the benchmark results. Note that the schema of this file is not stable.
    estimates_filename = "datasizes.json"

    datasizes_total_labels = [("serde_cityjson_total", {"label": "serde_cityjson", "color": "blue"}),
                              ("serde_value_total", {"label": "serde_json::Value", "color": "orange"}), ]

    plt.style.use('seaborn-v0_8-muted')

    # Plot relative datasizes
    fig, ax = plt.subplots(figsize=(10, 3))
    handles = []
    for group in GROUPS:
        group_dir = datasize_dir.joinpath(group)
        for bench_id in group_dir.iterdir():
            with bench_id.joinpath("new", estimates_filename).open("r") as fo:
                datasize = json.load(fo)
                for total_label, plot_cfg in datasizes_total_labels:
                    size_compared_to_baseline = datasize[total_label] / datasize["json"]
                    pt = ax.scatter(size_compared_to_baseline, group, color=plot_cfg["color"], label=plot_cfg["label"],
                                    marker=MARKER, s=MARKERSIZE)
                    handles.append(pt)

            plot_member_sizes(bench_id, datasize, group)
    # Scatter legend
    # see https://stackoverflow.com/a/13589144
    handles, labels = plt.gca().get_legend_handles_labels()
    by_label = dict(zip(labels, handles))
    legend1 = plt.legend(by_label.values(), by_label.keys(), loc="lower left")
    ax.add_artist(legend1)
    # mark the JSON string size
    ax.vlines(x=1, ymin=0, ymax=1, transform=ax.get_xaxis_transform(), colors="r")
    red_line = mlines.Line2D([], [], color='red', label="JSON string")
    ax.legend(handles=[red_line], loc="upper left")
    # Layout and annotation
    ax.grid(visible=True, which="major", axis="x")
    ax.set_title(f"Relative size of serde_cityjson structures compared to the JSON string")
    ax.set_xlabel("Factor of size relative to the size of the JSON string")
    plt.subplots_adjust(left=0.2, bottom=0.2)
    filepath = OUTPUT_FIGS_DIR.joinpath("datasize_relative.png")
    plt.savefig(filepath)
    print(f"Saved {filepath}")


def plot_member_sizes(bench_id, datasize, group):
    # Plot the size of members for the serde_cityjson structs
    fig, ax = plt.subplots(figsize=(10, 5))
    for member, size in datasize["serde_cityjson_citymodel"].items():
        if member == "cityobjects":
            for co_member in ["total_coid", "total_geometry", "total_attributes",
                              "total_geographical_extent", "total_children_id", "total_parents_id"]:
                size = datasize["serde_cityjson_citymodel"]["cityobjects"][co_member]
                size_pct = round(size / datasize["serde_cityjson_total"] * 100.0, 2)
                co_member_name = f"CityObject.{co_member.replace('total_', '')}"
                _ = ax.barh(co_member_name, size_pct)
        else:
            size_pct = round(size / datasize["serde_cityjson_total"] * 100.0, 2)
            _ = ax.barh(member, size_pct)
    ax.invert_yaxis()
    ax.set_xticks(list(range(0, 100, 10)))
    ax.set_xlim(left=0.0, right=100.0)
    ax.grid(visible=True, which="major", axis="x")
    fig.suptitle(f"Relative size of members compared to the total CityModel size", fontsize=14)
    ax.set_title(f"{group}/{bench_id.name}")
    ax.set_xlabel("Size of CityModel member relative to the total size [%]")
    plt.subplots_adjust(left=0.4, bottom=0.2)
    filepath = OUTPUT_FIGS_DIR.joinpath(f"datasize_members_{group}_{bench_id.name}.png")
    plt.savefig(filepath)
    plt.close()
    print(f"Saved {filepath}")


if __name__ == "__main__":
    compare_criterion()
    compare_datasize()
