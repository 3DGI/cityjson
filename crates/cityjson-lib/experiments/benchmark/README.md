# Benchmarking different architectures and implementations

The goal is to gain insight into the implications on performance of different approaches for working with CityJSON files.
There are three approaches in consideration.
1. The *direct-json*. This approach simply deserializes the json string with a json library (eg. `serde_json` or Python's standard `json`) and operates on the deserialized json values as-is.
2. The *vertex-index*. This approach adds a layer of abstraction over the deserialized json values by providing an API that helps with common operations. It retains the global/local vertex list and the  in-memory geometry structures index into this vertex list. This approach is commonly applied in applications that need to store and access meshes efficiently, such as rendering and game engines.
3. The *dereference*. This approach adds a layer of abstraction over the deserialized json values by providing an API that help with common operations. It replaces the vertex indices in the geometry boundaries by their coordinates. This is approach is widely used in GIS applications and is similar to the Simple Features specification.

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

## Focus on geometry and semantics

The benchmark is focusing on the geometry and semantics of the city model.
Attributes don't require special structures, they are stored as-is, irrespective of the chosen *approach*.
Therefore, they are not included in the benchmark.
Since appearances are stored with the same logic as semantics, it is sufficient to assess semantic operations in order to gain insight into the performance of appearance operations too.

## Use cases

There are four use cases that are considered common and used for benchmarking speed.

1. **deserialize**: Parse a CityJSON into a city model representation.
2. **serialize**: Serialize a city model into CityJSON.
3. **geometry**: Get the boundary coordinates of each surface.
4. **semantics**: Get the boundary coordinates of a given semantic surface.
5. **create**: Create a new city model from scratch.

# Acknowledgement

