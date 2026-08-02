# cityjson-index FFI Core

The C ABI exposes CityObject and package APIs. `cjx_index_package_source_paths` returns an owned UTF-8 byte array in package-ref order; release it with `cjx_bytes_array_free`. `cjx_index_read_filtered_packages`
reconstructs package refs and applies a typed `cjx_package_filter_t`. The function
returns an owned `cjx_filtered_package_t` array. Call
`cjx_filtered_packages_free(packages, count)` exactly once for a returned array.
