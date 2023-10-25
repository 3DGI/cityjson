# Contributors guide

## Naming conventions

### The `I`-prefix

The capital `I` prefix in signatures is an abbreviation for *Indexed*, such as `IGeometry` stands for *Indexed Geometry*.
*Indexed* refers to the indexed boundary representation, where only the vertex indices are stored in the geometry boundary instead of the coordinates.

Some boundary representations have the same depth.
To collectively refer to boundaries of the same depth, the *aggregate* prefix is used.
The boundary-aggregations of the same depth are:

- `multisurface`, `compositesurface`, `shell` --> `aggregatesurface`
- `multisolid`, `compositesolid` --> `aggregatesolid`

### Boundary and Geometry

CityJSON uses the term *Geometry* to refer to a geometry object with semantics and appearances, and uses the term *Boundary* to refer to the shape of the geometry object.
In 2D GIS we use the term "geometry" or "geometry boundary" to refer to what we call *Boundary* in CityJSON and in cjlib.

For the sake of consistency, cjlib adopts the geometry/boundary terminology and thus there is `PointBoundary` and `LineStringBoundary`.

# Engineering choices

## Equality, Ordering and Hashing of Boundaries

*Boundary* objects do not implement equality, because equality comparison of points is done with different tolerances, based on the use case.
The tolerance has a meaning, such as the maximum allowed 3D distance from the point.
Since [Eq](https://doc.rust-lang.org/std/cmp/trait.Eq.html) and [PartialEq](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html) does not allow a user provided tolerance value, they are not implemented.

*Boundary* objects do not implement ordering ([Ord](https://doc.rust-lang.org/std/cmp/trait.Ord.html) and [PartialOrd](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html)), because there is a multitude of spatial ordering schemes, such as Morton-order or Hilbert-order.
One might be better for some applications than the other, therefore the ordering of boundaries is left to the user.

*Boundary* objects do not implement hashing ([Hash](https://doc.rust-lang.org/std/hash/trait.Hash.html)), because the `f64` type is used for storing the coordinates and `f64` does not implement `Hash`.


## Coordinate precision and representation

*McShaffry* (pp. 447)[^1] discusses what size of a real-world area can be represented by 32bit float coordinates.
They say in Table 14.2 that if the unit of measurement is meters, the precision is 1mm, then the maximum range of values is 167000 meters.

*"Graphics chips (GPUs) always perform their math with 32-bit or 16-bit floats, the CPU/FPU is also usually faster when working in single-precision, and SIMD vector instructions operate on 128-bit registers that contain four 32-bit floats each. Hence, most games tend to stick to single-precision floating-point math."*  (pp. 139)[^2]

*"The vertices of a model are typically stored in object space, a coordinate system that is local to the particular model and used only by that model. The position and orientation of each model are often stored in world space, a global coordinate system that ties all the object spaces together."* (pp. 5)[^3]

[^1]: <div class="csl-entry">McShaffry, M. (2013). <span style="font-style: italic">Game coding complete</span>. Course Technology, Cengage Learning.</div>
[^2]: <div class="csl-entry">Gregory, J. (2018). <span style="font-style: italic">Game engine architecture</span>. Taylor and Francis, CRC Press.</div>
[^3]: <div class="csl-entry">Lengyel, E. (2012). <span style="font-style: italic">Mathematics for 3D game programming and computer graphics</span>. Course Technology PTR.</div>
