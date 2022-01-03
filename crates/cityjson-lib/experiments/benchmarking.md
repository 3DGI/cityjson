# Benchmarking different architectures and implementations

The goal is to gain insight into the implications on performance of different approaches for working with CityJSON files.

## What to measure?

City models and thus CityJSON files can be several hundreds of megabytes large, containing tens of thousands of CityObjects and millions of vertices.
Therefore, there are two aspects to consider about the performance of a CityJSON library:
1. The **size** of allocated memory with respect to the size of the CityJSON file on disk.
    + The whitespace and newline characters are removed from the CityJSON file, since they are also removed by the deserialization process.
    + The vertices are untransformed, since transformed vertices are not directly usable in applications.
      It is feasible to calculate the vertex coordinates on-demand in applications that read CityJSON data to reduce the memory footprint, for applications that generate, modify, extend city models it is not feasible to internally work with quantized coordinates, since it leads to extra bookkeeping and complexity.
      For instance, what is the transformation matrix for a newly generated city model?
      When updating or extending a city model, new points are created with real coordinates. How to integrate the new points into the original, transformed model and make sure that the transformation matrix is still appropriate for the extent of the updated model?
2. The **speed** of execution of operations with respect to the speed of execution of operations on the raw deserialized json. Thus, the baseline for benchmarking speed is the _direct-json_ approach.

## How to measure?

### Size, speed (approx)

```shell
$ /usr/bin/time -v target/release/benchmark ../data/3dbag_v210908_fd2cee53_5786_bench.city.json -a direct-json -c geometry
```

Ref.:
+ https://users.rust-lang.org/t/how-to-calculate-memory-usage/51240
+ https://stackoverflow.com/questions/774556/peak-memory-usage-of-a-linux-unix-process

### Speed (precise)

Since we are doing file I/O, the benchmark results can be heavily influenced by disk caches and whether they cold or warm.
Thus, for precise run time measurements the benchmarks need to be run on a warm cache.
The `time` command cannot do warmup runs, but [hyperfine](https://github.com/sharkdp/hyperfine) can.

## Focus on geometry and semantics

The benchmark is focusing on the geometry and semantics of the city model.
Attributes don't require special structures, they are stored as-is, irrespective of the chosen *approach*.
Therefore, they are not included in the benchmark.
Since appearances are stored with the same logic as semantics, it is sufficient to assess semantic operations in order to gain insight into the performance of appearance operations too.

## Use cases

There are four use cases that are considered common and used for benchmarking speed.

1. **deserialize**: Parse a CityJSON into a city model representation.
2. **serialize**: Serialize a city model into CityJSON. **NOT IMPLEMENTED**
3. **geometry**: Get the boundary coordinates of each surface.
4. **semantics**: Get the boundary coordinates of a given semantic surface.
5. **create**: Create a new city model from scratch.

## Benchmarked files

All files are upgraded to CityJSON v1.1 and preprocessed with the *prepare* tool.

+ *cluster* – A tiny file containing a couple of LoD2.2 building models (not triangulated) from the 3D BAG.
+ *3dbag_v210908_fd2cee53_5786* – A 3D BAG tile, containing building models (not triangulated) in four levels of detail (LoD0, LoD1.2, LoD1.3, LoD2.2).
+ *32cz1_04* – A [3D Basisbestand Volledig](https://3d.kadaster.nl/basisvoorziening-3d/) tile, containing several CityObject types, all of them with triangulated surfaces. [download link](https://download.pdok.nl/kadaster/basisvoorziening-3d/v1_0/2020/volledig/32cz1_2020_volledig.zip)

# Build and run the tools



Prepare the input cityjson files first so that they comply with the requirements stated above.
Build the `prepare` executable with the nightly release of rust.

```shell
cargo +nightly build --release --bin prepare
```

Run the tool to strip the unnecessary parts from the input files.
The tool will write the result into the same directory as the input (`my_file.city.json`) and append `_bench` to the name
of the input file (`my_file_bench.city.json`).

```shell
prepare my_file.city.json
```

The `benchmark` tool will run the benchmarks.
Build the `benchmark` executable as:

```shell
cargo build --release --bin benchmark
```

The `benchmark` tool takes the selected architecture and use case as argument.
For instance to run the use case *geometry* on the *dereference* architecture:

```shell
benchmark my_file_bench.city.json -a dereference -c geometry
```

# Results

The benchmark is run in the jupyter notebook `experiments/benchmark.ipynb`.
This notebook contains the scripts and detailed results.

## Commit dcd842bd – 2022-01-02

We narrowed down the tests to the *deserialize* case in order to get a simple comparison to loading a CityJSON file with python's standard `json` library.
We  can observe that there is a 6-70x increase in memory footprint compared to the size of the file stored on disk.
The largest increase is in case of *cluster*, which is 70x the file size and occurs with python.
The lowest increase is case of *3dbag_v210908_fd2cee53_5786*, which is 6x the file size and occurs with the *dereference* architecture.
In case of *cluster*, the memory footprint of *direct-json* and *dereference* architectures are similar, about 20x the file size.
In case of *3dbag_v210908_fd2cee53_5786*, the memory footprint of *dereference* is the lowest, about 6x the file size. The two other architectures have a similar memory usage, 10x the file size.
In case of *32cz1_04*, the memory footprint of *direct-json* is the lowest, 7.8x the file size, while *dereference* requires 8.3x the file sizie in memory.
This difference of 0.5x file size means 300Mb.
The python version requires 8.6x memory.

The results indicate that even a simplistic implementation of a *dereference* architecture can operate on a memory footprint that is similar to what is required by the *direct-json* approach.
Thus, convenience does not come on the cost of performance.

Apart from the minimal *cluster* file, the memory footprint of the python implementation is comparable to the rust implementation.
Thus, simply using rust's *serde* library does not provide significant benefits in memory consumption compared to the CPython implementation of the `json` python standard library.

From these results the following questions arise:

1. Why does the memory footprint increase to multiple-fold the file size when deserializing JSON files?

In Rust, the high memory usage is due to the characteristis of the `serde_json::Value` type, as outlined [here](https://github.com/serde-rs/json/issues/635#issue-584766942).

In Python, the high memory usage is due to the decoding of UTF-8 encoded files.
This is explained in detail in [this post](https://stackoverflow.com/a/58080893/3717824).

2. How can we reduce the memory footprint of the *dereference* architecture?

By deserializing the specific boundary types into their own structure instead of the generic `serde_json::Value`.
For the sake of simplicity, the current implementation of the Geometry structure that the CityJSON is deserialized into, is using a generic `serde_json::Value` type for the `Geometry.boundaries`.
This is to avoid implementing a specialized `Deserializer` that can differentiate between the boundary types in the file and deserialize them into their specific structure (`Solid`, `LineString` etc.).
However, this has an overhead in memory use.

By shrinking the vector allocations so that they fit their elements exactly.
Rust's `Vec` grows exponentially.
It allocates 8 elements by default and then doubles when full [Ref](https://doc.rust-lang.org/src/alloc/raw_vec.rs.html#116-127).
Since vectors are used in several structures, this can result in severe memory overallocation.
Alternatively, it could be possible to [process an array of values without buffering into a Vec](https://serde.rs/stream-array.html). Or an [ijson::array::IArray]() could be used instead of vectors.

By [interning](https://en.wikipedia.org/wiki/String_interning) the string values in the CityJSON.
If there are many duplicate values (eg. `lod`, or attribute values), this could save lots of memory.
The [ijson](https://crates.io/crates/ijson) rust library has an `IString` type which is interned.

By iterative (streaming) deserialization, instead of loading the whole file contents into memory.
A generic streaming array deserialization example is [here](https://github.com/serde-rs/json/issues/160#issuecomment-841344394).
Or for instance the [ijson](https://pypi.org/project/ijson/) python library.
