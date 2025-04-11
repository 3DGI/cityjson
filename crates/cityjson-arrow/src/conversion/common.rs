use arrow::buffer::Buffer;
use cityjson::prelude::*;
use std::mem;

/// # Safety
///
/// This function is unsafe because it relies on the caller ensuring that
/// `VertexIndex<T>` is `#[repr(transparent)]` over `T` and that `T` is a primitive
/// type compatible with Arrow's Buffer expectations (like u16, u32, u64).
/// It takes ownership of the input vector's allocation.
unsafe fn vec_vertexindex_to_primitive_vec<T>(vec: Vec<VertexIndex<T>>) -> (Vec<T>, usize)
where
    T: VertexRef + Sized, // Sized is usually implied but good to be explicit
{
    let len = vec.len();
    let cap = vec.capacity();
    let ptr = vec.as_ptr(); // Get pointer to the start

    // Prevent Rust from dropping the original Vec, so we can take ownership of its buffer
    mem::forget(vec);

    // Reconstruct the Vec using the same memory but typed as Vec<T>
    // This relies on VertexIndex<T> having the exact same layout as T.
    let primitive_vec = Vec::from_raw_parts(ptr as *mut T, len, cap);

    (primitive_vec, len)
}

/// Creates an Arrow Buffer from a Vec<VertexIndex<T>> without copying element data.
/// Takes ownership of the input Vec's allocation.
///
/// # Safety
/// Relies on the safety guarantees of vec_vertexindex_to_primitive_vec.
pub unsafe fn vec_vertexindex_to_arrow_buffer<T>(vec: Vec<VertexIndex<T>>) -> (Buffer, usize)
where
    T: VertexRef + arrow::datatypes::ArrowPrimitiveType + arrow::datatypes::ArrowNativeType, // Ensure T is Arrow-compatible
{
    let (primitive_vec, len) = vec_vertexindex_to_primitive_vec(vec);
    // Buffer::from_vec takes ownership, preventing a copy here
    (Buffer::from_vec(primitive_vec), len)
}
