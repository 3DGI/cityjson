# Engineering choices

## Coordinate precision and representation

*McShaffry* (pp. 447)[^1] discusses what size of a real-world area can be represented by 32bit float coordinates.
They say in Table 14.2 that if the unit of measurement is meters, the precision is 1mm, then the maximum range of values is 167000 meters.

*"Graphics chips (GPUs) always perform their math with 32-bit or 16-bit floats, the CPU/FPU is also usually faster when working in single-precision, and SIMD vector instructions operate on 128-bit registers that contain four 32-bit floats each. Hence, most games tend to stick to single-precision floating-point math."*  (pp. 139)[^2]

*"The vertices of a model are typically stored in object space, a coordinate system that is local to the particular model and used only by that model. The position and orientation of each model are often stored in world space, a global coordinate system that ties all the object spaces together."* (pp. 5)[^3]

[^1]: <div class="csl-entry">McShaffry, M. (2013). <span style="font-style: italic">Game coding complete</span>. Course Technology, Cengage Learning.</div>
[^2]: <div class="csl-entry">Gregory, J. (2018). <span style="font-style: italic">Game engine architecture</span>. Taylor and Francis, CRC Press.</div>
[^3]: <div class="csl-entry">Lengyel, E. (2012). <span style="font-style: italic">Mathematics for 3D game programming and computer graphics</span>. Course Technology PTR.</div>
