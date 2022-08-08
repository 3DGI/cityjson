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

The benchmark is run in the jupyter notebook [benchmark.ipynb](https://github.com/balazsdukai/cjlib/blob/master/experiments/benchmark.ipynb).
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

## Commit 96f67d82 – 2022-01-04

Implemented the deserialization of Geometry types into their specific structures, instead of using the generic `serde_json::Value`.
This reduced the memory use of *dereference* by about 300Mb on the *32cz1_04* triangulated model, so that it has the same memory footprint as *direct-json* on this file, 7.8x the file size, 4.6GB.

## Commit e24772fb – 2022-01-27

Implemented the deserialization for the *vertex-index* architecture (in `vertex_index.rs`).
It only uses 2.1x the file size with is a very significant reduction compared to the other approaches.
It doesn't do anything special, just parses the file into a CityJSON data structure, which is an exact copy of the schema.

## Commit 95f01517 – 2022-01-27 – Memory map

Implemented that the input files are memory mapped instead of reading them with from a buffered reader.
The important parts for memmapping with serde are the `Mmap::map` unsafe method and that serde reads from slice.

```rust
let file = File::open(path_in).expect("Couldn't read CityJSON file");
let mmap = unsafe { memmap2::Mmap::map(&file) }.unwrap();
let cm: CityModel = serde_json::from_slice(&mmap).expect("Couldn't deserialize into CityModel");
```

However, the memmap actually increased the memory footprint from the previous implementation at `e24772fb` by about a 100%. 
For instance in case of the vertex-index architecture the memory footprint went from 2.1x to 3.1x.

## Commit da646456 – 2022-05-02 – ijson

Implemented the [ijson](https://crates.io/crates/ijson) library for the direct-json architecture, as a direct replacement of `serde_json`.
In case of the of the direct-json architecture, using `ijson` indeed leads to a lower memory footprint, at around 6.5x of the file size.
This makes it an interesting option for deserialization, esp. because in case of the *32cz1_04* model, the reduction from `serde_json` to `ijson` is about 1.8x, which means 1GB(!) of allocated memory.
This also means that with `json`, the direct-json architecture uses 1.8x less memory than the dereference architecture.
However, the vertex-index architecture's memory usage is still only about 2.1x the file size and this architecture at least gives the CityJSON specific data structures.

However, I also incorporated the `ijson::IString` and `ijson::IArray` types into parts of the vertex-index architecture and it shows a different picture.
Using `ijson::IString` leads to a lower memory usage, which I expect will be more significant when the city model attributes are also read.
On the other hand, using `ijson::IArray` leads to a significantly higher memory usage compared to a `Vec` of a specific type, eg `Vec<i64>`.

## Streaming array deserialization

Implemented a test for counting the number of vertices without reading the whole file into memory.
It is in the `streaming-array` crate.
The memory profiler show very promising results for this, as the peak memory use stays way below the file size.
Tried `memmap`-ing again, which has an even lower footprint in general and is faster. 
However, it shows a strange high peak when the file is opened and when is closed. I'm not sure yet why that happens.

## Memory-mapped files

*"A memory-mapped file is a segment of virtual memory that has been assigned a direct byte-for-byte correlation with some portion of a file or file-like resource. This resource is typically a file that is physically present on disk, but can also be a device, shared memory object, or other resource that the operating system can reference through a file descriptor. Once present, this correlation between the file and the memory space permits applications to treat the mapped portion as if it were primary memory."* [Ref](https://en.wikipedia.org/wiki/Memory-mapped_file)

**Benefits**

- increased I/O, esp. in large files
- lazy loading, so only load the portion of the file that is needed

**Drawbacks**

- more page faults, and Linux has a cap on the nr. of cores handling page faults, which on fast systems (eg NVME) can be a real concern, which impacts the scalability of the application
- if the file is modified or goes out of process that results in Undefined Behaviour (with `memmap2`)

**Types of memory mapping**

- *persisted*: large files on disk
- *non-persisted*: inter-process communication (IPC)

### Strategies

I've been thinking that it could be interesting to memmap the file and locate the *vertices* array and the *transform* object.
They would be deserialized first (and brought into main memory).
Subsequently, the *CityObjects* can be deserialized and their geometries dereferenced by using the *vertices*.
However, locating the *vertices* in the memmapped file seems like a brittle and cumbersome approach, because what if there is a custom attribute, `vertices: some value`? It would easily give a false positve in seeking the byte-array of the file.

Alternatively, in an ideal case this would be possible:

1) Memory-map the file, so that it is not in read into memory.
2) Deserialize the memory-map byte-array into an indexed-json structure using zero-copy deserialization, so that the deserialized structures are just references of the byte-array slices, which are themselves references of the file on disk.
3) Parse the indexed-json structure into the target CityJSON structure by doing a full-copy. 

The strategy above, in theory, allows to parse a CityJSON file into a dereferenced CityJSON structure without replicating the file contents more than once in memory.
That one replication is the target CityJSON structure itself.
But the intermediary steps are executed fully on the memory-mapped file only.

However, it turns out that I cannot just reference a memory buffer when deserializing types other than `&str` and `&[u8]`.
The types `&str` and `&[u8]` are just streams of bytes, therefore they can be deserialized as references into the buffer.
But this requires that these Rust types are laid out exactly as they exist in the buffer I'm deserializing.
While this is true for the two types above, it is not true for other types like `f64` for instance.
Especially that a text format like JSON only know a single *number* type, but what does that number represent in Rust?
Is it `u32` or `u64`?
Since the incoming layout does not match the target layout, some kind of parsing needs to happen from such a format and *someone needs to own the resulting data*.

They types `Vec`, `HashMap` and `String` require allocations.

The [zerovec](https://docs.rs/zerovec/0.6.0/zerovec/) package does provide zero-copy vector abstractions, except that in case of text formats (*json*) the referenced data will be owned (copied). 

### Type sizes

I think it is reasonable to use an `i32` for the transformed vertices.
An `i32` can store signed integers in the range from *-2147483648* to *2147483647*.
I haven't checked if after transformation this is still within the range of the world CRS stuff, but seems sufficient.

## Commit fb50fd77 – 2022-05-04 – Boundary vectors with capacity

Creating the geometry boundary vectors `Vec::with_capacity()` makes a significant reduction in the peak memory usage.
Creating the vector *with capacity* reserves for the vector so that *capacity* number of elements can be added to it [without reallocation](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
Since at the point of creating the final boundary vectors I already know their exact size from the deserialized data, I can create the target vectors with exact capacity.

## Commit 8d82ab55 – 2022-05-04 – Alternative memory allocators

I tried an alternative allocator, [tikv-jemallocator](https://crates.io/crates/tikv-jemallocator), just because it was recommended in the [Rust Performance book](https://nnethercote.github.io/perf-book/heap-allocations.html#using-an-alternative-allocator).
However, there was a slight increase in memory use.
Also, probably much greater improvements can be achieved by changes in the parsing/deserialize code itself before looking into alternative allocators.

## Commit 83af8196 – Revisit the streaming deserialize with two passes over the data

A major milestone.
I think I nailed down the parsing strategy.
It is done in two passes over the data.
First, everything is deserialized except the CityObjects.
In this pass, I memory-map the file, because that is the fastest and even though serde pulls the whole content in the memory, it still skips the CityObjects.
This is not a problem, because the vertices and the rest of the properties are not too large.
This gives me the *vertices* in memory.
In the second pass, I go through the CityObjects and for each CityObject I first deserialize it into an indexed-structure, but then in the same iteration I parse it to the dereferenced structure.
So like this I avoid having to store the whole CityObjects twice in memory.
With `serde::de::DeserializeSeed` I can pass in external data, a 'seed', to the deserializer, which is the vertices array in my case.
It is in the `custom-serde` crate that demonstrates how this works and also my [question on the Rust users forum](https://users.rust-lang.org/t/serde-memory-efficient-parsing-strategy-for-object-that-depends-on-other-object/75287). 
With this strategy I reduced the peak memory usage by ~2x compared to, which is 6.2x --> 4.3x on the 3D BAG, and 7.8x --> 5.4x on the 3D Basisbestand.

Considering that the dereferenced data structure itself takes up about 3.3x (3D BAG) and 4.9x (triangulated 3D Basisbestand) memory on the heap, I am happy with these results.
Heap memory allocation is estimated with [datasize](https://github.com/casperlabs/datasize-rs).

But 5.4x of file size is still quite a lot, especially when considering that attribute data is not even included in the benchmarks yet.
And especially when considering that the plain vertex-index architecture has only 2-2.5x peak memory use.
With 5.4x of file size, the 3D Basisbestand requires 3.2GB RAM, thus loading four of them to merge or get a subset of the four corners needs 12.8GB, so it needs a machine with at least 16GB RAM.

Memory-mapping the file in also in the second pass significantly reduces the parsing time.
In case of the 3D Basisbestaand, 14s --> 8s.
However, it increases the peak memory use by exactly 1x compared to a BuffReader, since the whole file is pulled into memory because of the `deserialize::from_slice` (I think).

## Revisiting the data structure

For a while now, I was focused on getting an memory-efficient deserialization strategy.
Since the beginning I had the idea that probably a multi-pass approach would work the best for the *dereference* architecture.
A multi-pass approach allows me to deserialize the vertices in the first pass and then deserialize and flush the CityObjects one-by-one, by using the in-memory vertices.
It is because for the *dereference* approach I need to deserialize the CityObject into some intermediary structure.
Commit `83af8196` is the culmination of my efforts in getting the deserialization right.

Thus, my deserialization strategy is as efficient as I can get it I think, yet the memory footprint is till pretty high, up to 5.4x the file size.
So the next period is dedicated to optimizing the data structure.
Beginning with understanding [why does my data structure allocate so much memory?](https://users.rust-lang.org/t/why-does-my-data-structure-allocate-so-much-memory/75819?u=balazsdukai).
From the fantastic replies to my question I learned that,
+ I need to also include the `std::mem::size_of_val()` in the structure size calculation [link](https://users.rust-lang.org/t/why-does-my-data-structure-allocate-so-much-memory/75819/2?u=balazsdukai)
+ the `Option` takes just as much space as its largest variant (`Some`) [link](https://users.rust-lang.org/t/why-does-my-data-structure-allocate-so-much-memory/75819/4?u=balazsdukai)
+ The `Box`, `Rc` and `Arc` types can provide memory-optimizations [link](https://users.rust-lang.org/t/why-does-my-data-structure-allocate-so-much-memory/75819/6?u=balazsdukai)
+ there are several possibilities for reducing the data structure size through deduplication [link](https://users.rust-lang.org/t/why-does-my-data-structure-allocate-so-much-memory/75819/9?u=balazsdukai)
+ my suspicion that an Entity Component System could be the way to go is confirmed [link](https://users.rust-lang.org/t/why-does-my-data-structure-allocate-so-much-memory/75819/13?u=balazsdukai)

Before I would jump into optimizing the data structure, I have to review my goals, as in, what operations should the data structure support?

+ Store the complete data that can be represented by the CityJSON specs, including,
  + Geometry templates,
  + CityJSONFeatures,
  + Extensions.
+ Allow the creation of FFI-s in 
  + C++, 
  + Python,
  + WASM.
+ Create new CityModels from scratch.
+ Modify an existing CityModel.
  + Modify root property values (eg. `version`).
  + Modify/add/remove `Metadata`.
  + Modify CityObjects in an existing CityModel.
    + Modify the geometry.
      + Modify the boundary
        + Change the coordinates of a vertex.
        + Add a vertex.
        + Remove a vertex.
      + Modify the `texture/material`.
        + Change the texture/material of a surface.
        + Add a new texture/material of a surface.
        + Remove the texture/material of a surface.
      + Modify the `semantics`.
        + Change the semantics value of a surface.
        + Add new semantics to a surface.
        + Remove the semantics from a surface.
    + Modify the attributes.
      + Change the attribute value.
      + Add a new attribute.
      + Remove an attribute.
    + Add a new CityObject.
    + Drop a CityObject.
+ Drop a CityModel.

After thinking more about it, I realized that the *vertex-index* architecture is not suitable for supporting all the operations.
At least, using this architecture makes it overly complicated to drop vertices and geometries without leaking memory.

The tricky part with the vertex-index architecture is the removal of Geometries from CityObjects.
When I remove a Geometry, it is not enough to simply drop the Geometry object.
I need to also remove all the vertices from the vertex buffer that are not needed anymore.
As in, they are not referenced by any other Geometry object.
If I keep the unnecessary vertices, then I'm leaking memory.

### Commit 7b7a4eab – 2022-05-16
Implemented the most naive approach, which in fact doesn't work. See the crate `vertex-index`.

```rust
    fn drop_geometry(&mut self, i: usize) {
        if self.geometries.len() - 1 < i {
            println!("geometry index out of bounds")
        } else {
            let geom_removed = self.geometries.remove(i);
            let mut vtx_to_keep: Vec<usize> = Vec::new();
            for g in &self.geometries {
                for v in &g.boundary {
                    if geom_removed.boundary.contains(v) {
                        vtx_to_keep.push(v.clone())
                    }
                }
            }
            for v in &geom_removed.boundary {
                if !vtx_to_keep.contains(v) {
                    self.vertices.remove(*v);
                }
            }
        }
    }
```

There are two major issues with this function.
1. When dropping a geometry, I need to iterate over each other geometry and each of their boundaries.
2. The vertex removal is not even working, since Vec::remove() shifts all remaining elements to the left, which messes up the vertex-indices in all the remaining Geometries. In a naive approach for solving this would require iterating over boundaries and shifting the vertex-indices for each removed vertex, which is crazy.

Additionally, the size increase that comes from the vertex duplication is negligable compared to the duplication of some structures (eg. Material).
Additionally additionally, the memory benefit of *vertex-index* architecture only appears when parsing an already compressed/deduplicated CityJSON file.
When creating a new citymodel, the vertices are just added to the model as-is, duplicates and all and the duplicates are only removed in a post-process, if at all.

Long story short, I continue with the *dereference* architecture and focus on making it as efficient as possible.

## Floating point precision

A single precision number is enough to [point out Waldo in a global geographic CRS](https://xkcd.com/2170), since it 
can represent up to [7 decimal digits](https://en.wikipedia.org/wiki/Single-precision_floating-point_format).
In a projected, metric CRS we only need 3 decimal digits.