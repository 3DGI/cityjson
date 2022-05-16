# Testing a vertex-index architecture

The tricky part with the vertex-index architecture is the removal of Geometries from CityObjects.
When I remove a Geometry, it is not enough to simply drop the Geometry object.
I need to also remove all the vertices from the vertex buffer that are not needed anymore.
As in, they are not referenced by any other Geometry object.
If I keep the unnecessary vertices, then I'm leaking memory.

## Commit 7b7a4eab – 2022-05-16
Implemented the most naive approach, which in fact doesn't work.

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