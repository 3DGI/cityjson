# An architecture based on a vertex buffer

Maybe a global vertex list is not such a bad idea within the application either.
What advantages would it bring?

1) reduced space
2) topology? but that doesn't come by default either

Although, it would complicate things.
Possibly, overcomplicate things.
For instance, what if I erase a vertex?
If I cannot erase a vertex, then when can I remove the vertices from the memory?
How do CAD software do this? Or 3D graphics software like Blender? Or games?

A [SlotMap](https://docs.rs/slotmap/1.0.6/slotmap/) could be very useful for building a vertex buffer based library.

Okay, game engines use an *indexed triangle list*, also called a *vertex buffer* (DirectX)
or *vertex array* (OpenGL), just as OBJ and CityJSON[1].
Games often store quite a lot of metadata with each vertex, so repeating this data in a triangle list wastes memory.
It also wastes GPU bandwidth, because a duplicated vertex will be transformed and lit multiple times.

*"So in a 3D rendered world, everything seen will start as a collection of vertices and texture maps. They are collated into memory buffers that link together -- a __vertex buffer__ contains the information about the vertices; an __index buffer__ tells us how the vertices connect to form shapes; a __resource buffer__ contains the textures and portions of memory set aside to be used later in the rendering process; a __command buffer__ the list of instructions of what to do with it all."*[3]

Blender uses a non-manifold boundary representation called *BMesh*[2]. 
This it uses a global vertex list too. 

## References

+ [1]: Gregory, J. (2018). Game engine architecture. Taylor and Francis, CRC Press.
+ [2]: Blender. (2020). The BMesh Structure. https://wiki.blender.org/wiki/Source/Modeling/BMesh/Design. Accessed on 2021-11-15.
+ [3]: Evanson, N. (2019). 3D Game Rendering. https://www.techspot.com/article/1851-3d-game-rendering-explained/. Accessed on 2021-11-15.
