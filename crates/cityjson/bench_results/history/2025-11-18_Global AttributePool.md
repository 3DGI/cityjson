# Benchmark Results: 2025-11-18 - Global AttributePool

- **Date**: 2025-11-18 22-16-26
- **Commit**: 58f79ce
- **Rust Version**: 1.91.1
- **System**: Linux 6.14.0-35-generic x86_64

## Criterion Benchmarks

   Compiling cityjson v0.3.0 (/home/balazs/Development/cityjson-rs)
    Finished `bench` profile [optimized + debuginfo] target(s) in 2.27s
     Running unittests src/lib.rs (target/release/deps/cityjson-f8577dec02935d30)

running 149 tests
test backend::default::attributes::tests::test_attribute_container ... ignored
test backend::default::attributes::tests::test_attribute_pool_basic ... ignored
test backend::default::attributes::tests::test_attribute_pool_maps ... ignored
test backend::default::attributes::tests::test_attribute_pool_vectors ... ignored
test backend::default::attributes::tests::test_clear ... ignored
test backend::default::attributes::tests::test_display_implementations ... ignored
test backend::default::attributes::tests::test_get_by_key ... ignored
test backend::default::attributes::tests::test_nested_maps_and_vectors ... ignored
test backend::default::attributes::tests::test_remove_and_reuse ... ignored
test backend::default::boundary::nested_tests::test_empty_nested_conversions ... ignored
test backend::default::boundary::nested_tests::test_multisolid_with_complex_structure ... ignored
test backend::default::boundary::nested_tests::test_nested_multilinestring_with_empty_linestrings ... ignored
test backend::default::boundary::nested_tests::test_nested_multisurface_with_empty_components ... ignored
test backend::default::boundary::nested_tests::test_type_alias_consistency ... ignored
test backend::default::boundary::tests::test_boundary_consistency ... ignored
test backend::default::boundary::tests::test_boundary_counter ... ignored
test backend::default::boundary::tests::test_boundary_type_detection ... ignored
test backend::default::boundary::tests::test_boundary_with_capacity ... ignored
test backend::default::boundary::tests::test_display_boundary_type ... ignored
test backend::default::boundary::tests::test_empty_boundary ... ignored
test backend::default::boundary::tests::test_multi_linestring_conversion ... ignored
test backend::default::boundary::tests::test_multi_point_conversion ... ignored
test backend::default::boundary::tests::test_multi_solid_conversion ... ignored
test backend::default::boundary::tests::test_multi_surface_conversion ... ignored
test backend::default::boundary::tests::test_solid_conversion ... ignored
test backend::default::coordinate::tests::test_flexible_coordinate ... ignored
test backend::default::coordinate::tests::test_quantized_coordinate ... ignored
test backend::default::coordinate::tests::test_real_world_coordinate ... ignored
test backend::default::coordinate::tests::test_uv_coordinate ... ignored
test backend::default::coordinate::tests::test_vertices16_limit ... ignored
test backend::default::coordinate::tests::test_vertices32_capacity ... ignored
test backend::default::coordinate::tests::test_vertices_default ... ignored
test backend::default::coordinate::tests::test_vertices_indexing ... ignored
test backend::default::coordinate::tests::test_vertices_methods ... ignored
test backend::default::extension::tests::test_extension ... ignored
test backend::default::extension::tests::test_extensions_add_get ... ignored
test backend::default::extension::tests::test_extensions_iteration ... ignored
test backend::default::extension::tests::test_extensions_remove_empty ... ignored
test backend::default::geometry::tests::test_geometry_template_and_instance ... ignored
test backend::default::geometry::tests::test_multilinestring ... ignored
test backend::default::geometry::tests::test_multipoint_with_add_point ... ignored
test backend::default::geometry::tests::test_multipoint_with_add_vertex ... ignored
test backend::default::geometry::tests::test_multipoint_with_mixed_adds ... ignored
test backend::default::geometry::tests::test_multipoint_with_semantics ... ignored
test backend::default::geometry::tests::test_multisolid ... ignored
test backend::default::geometry::tests::test_multisurface ... ignored
test backend::default::geometry::tests::test_solid ... ignored
test backend::default::transform::test::display ... ignored
test backend::default::vertex::tests::test_hash_and_equality ... ignored
test backend::default::vertex::tests::test_integer_to_vertex_index_conversion ... ignored
test backend::default::vertex::tests::test_to_usize_conversion ... ignored
test backend::default::vertex::tests::test_vertex_coordinate ... ignored
test backend::default::vertex::tests::test_vertex_index_conversion ... ignored
test backend::default::vertex::tests::test_vertex_index_creation ... ignored
test backend::default::vertex::tests::test_vertex_index_from_u32 ... ignored
test backend::default::vertex::tests::test_vertex_index_helpers ... ignored
test backend::default::vertex::tests::test_vertex_index_overflow - should panic ... ignored
test backend::default::vertex::tests::test_vertex_index_vec_trait ... ignored
test backend::default::vertex::tests::test_vertex_indices_sequence ... ignored
test cityjson::tests::test_geometry_type_equality ... ignored
test cityjson::tests::test_lod_ordering ... ignored
test resources::mapping::tests::test_accessors ... ignored
test resources::mapping::tests::test_is_empty ... ignored
test resources::mapping::tests::test_material_map_type_alias ... ignored
test resources::mapping::tests::test_semantic_map_type_alias ... ignored
test resources::mapping::tests::test_semantic_or_material_map_check_type ... ignored
test resources::mapping::textures::tests::test_accessor_methods ... ignored
test resources::mapping::textures::tests::test_texture_map_creation ... ignored
test resources::mapping::textures::tests::test_texture_map_hierarchy ... ignored
test resources::mapping::textures::tests::test_texture_map_population ... ignored
test resources::pool::tests::basic_operations::test_add_and_get ... ignored
test resources::pool::tests::basic_operations::test_find ... ignored
test resources::pool::tests::basic_operations::test_get_mut ... ignored
test resources::pool::tests::basic_operations::test_invalid_id ... ignored
test resources::pool::tests::basic_operations::test_len ... ignored
test resources::pool::tests::basic_operations::test_remove ... ignored
test resources::pool::tests::boundary_conditions::test_generation_overflow_prevention ... ignored
test resources::pool::tests::boundary_conditions::test_generation_wraparound ... ignored
test resources::pool::tests::boundary_conditions::test_is_valid_edge_cases ... ignored
test resources::pool::tests::boundary_conditions::test_multiple_generation_overflows ... ignored
test resources::pool::tests::clear_tests::test_add_after_clear ... ignored
test resources::pool::tests::clear_tests::test_clear_basic ... ignored
test resources::pool::tests::clear_tests::test_clear_empty_pool ... ignored
test resources::pool::tests::clear_tests::test_clear_with_reuse ... ignored
test resources::pool::tests::concurrency_and_performance::test_concurrent_access ... ignored
test resources::pool::tests::concurrency_and_performance::test_performance ... ignored
test resources::pool::tests::initialization::test_new_pool ... ignored
test resources::pool::tests::initialization::test_with_capacity ... ignored
test resources::pool::tests::iter_mut_tests::iteration_mutable ... ignored
test resources::pool::tests::iter_mut_tests::test_iter_mut_collects_all_valid_resources ... ignored
test resources::pool::tests::iter_mut_tests::test_iter_mut_on_empty_pool ... ignored
test resources::pool::tests::iter_mut_tests::test_iter_mut_with_custom_types ... ignored
test resources::pool::tests::iteration::test_iter ... ignored
test resources::pool::tests::iteration::test_iter_empty_pool ... ignored
test resources::pool::tests::iteration::test_iter_with_all_removed ... ignored
test resources::pool::tests::memory_safety::test_memory_leaks ... ignored
test resources::pool::tests::memory_safety::test_resource_lifetime ... ignored
test resources::pool::tests::resource_id::test_conversion ... ignored
test resources::pool::tests::resource_management::test_generation_increment ... ignored
test resources::pool::tests::resource_management::test_multiple_removals_and_additions ... ignored
test resources::pool::tests::resource_management::test_reuse_freed_slot ... ignored
test v1_0::appearance::material::tests::test_material_equality ... ignored
test v1_0::appearance::texture::tests::test_texture_equality ... ignored
test v1_0::cityobject::tests_cityobjects_container::test_attribute_filtering ... ignored
test v1_0::cityobject::tests_cityobjects_container::test_basic_operations ... ignored
test v1_0::cityobject::tests_cityobjects_container::test_bulk_operations ... ignored
test v1_0::cityobject::tests_cityobjects_container::test_filtering ... ignored
test v1_0::geometry::semantic::tests::test_semantic_attributes ... ignored
test v1_0::geometry::semantic::tests::test_semantic_children ... ignored
test v1_0::geometry::semantic::tests::test_semantic_creation ... ignored
test v1_0::geometry::semantic::tests::test_semantic_display ... ignored
test v1_0::geometry::semantic::tests::test_semantic_equality ... ignored
test v1_0::geometry::semantic::tests::test_semantic_parent ... ignored
test v1_0::geometry::semantic::tests::test_semantic_type_extension ... ignored
test v1_1::appearance::material::tests::test_material_equality ... ignored
test v1_1::appearance::texture::tests::test_texture_equality ... ignored
test v1_1::cityobject::tests_cityobjects_container::test_attribute_filtering ... ignored
test v1_1::cityobject::tests_cityobjects_container::test_basic_operations ... ignored
test v1_1::cityobject::tests_cityobjects_container::test_bulk_operations ... ignored
test v1_1::cityobject::tests_cityobjects_container::test_filtering ... ignored
test v1_1::geometry::semantic::tests::test_semantic_attributes ... ignored
test v1_1::geometry::semantic::tests::test_semantic_children ... ignored
test v1_1::geometry::semantic::tests::test_semantic_creation ... ignored
test v1_1::geometry::semantic::tests::test_semantic_display ... ignored
test v1_1::geometry::semantic::tests::test_semantic_equality ... ignored
test v1_1::geometry::semantic::tests::test_semantic_parent ... ignored
test v1_1::geometry::semantic::tests::test_semantic_type_extension ... ignored
test v1_1::metadata::test::display ... ignored
test v2_0::appearance::material::tests::test_material_equality ... ignored
test v2_0::appearance::texture::tests::test_texture_equality ... ignored
test v2_0::citymodel::tests::test_clear_cityobjects ... ignored
test v2_0::citymodel::tests::test_clear_geometries ... ignored
test v2_0::citymodel::tests::test_clear_template_vertices ... ignored
test v2_0::citymodel::tests::test_clear_vertices ... ignored
test v2_0::citymodel::tests::test_get_or_insert_material ... ignored
test v2_0::citymodel::tests::test_get_or_insert_semantic ... ignored
test v2_0::citymodel::tests::test_get_or_insert_texture ... ignored
test v2_0::cityobject::tests_cityobjects_container::test_attribute_filtering ... ignored
test v2_0::cityobject::tests_cityobjects_container::test_basic_operations ... ignored
test v2_0::cityobject::tests_cityobjects_container::test_bulk_operations ... ignored
test v2_0::cityobject::tests_cityobjects_container::test_filtering ... ignored
test v2_0::geometry::semantic::tests::test_semantic_attributes ... ignored
test v2_0::geometry::semantic::tests::test_semantic_children ... ignored
test v2_0::geometry::semantic::tests::test_semantic_creation ... ignored
test v2_0::geometry::semantic::tests::test_semantic_display ... ignored
test v2_0::geometry::semantic::tests::test_semantic_equality ... ignored
test v2_0::geometry::semantic::tests::test_semantic_parent ... ignored
test v2_0::geometry::semantic::tests::test_semantic_type_extension ... ignored
test v2_0::metadata::test::display ... ignored

test result: ok. 0 passed; 0 failed; 149 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running benches/backend_comparison.rs (target/release/deps/backend_comparison-4de9ff34a783c598)
Benchmarking backend_comparison/default/build_solids/100
Benchmarking backend_comparison/default/build_solids/100: Warming up for 3.0000 s
Benchmarking backend_comparison/default/build_solids/100: Collecting 100 samples in estimated 5.2816 s (86k iterations)
Benchmarking backend_comparison/default/build_solids/100: Analyzing
backend_comparison/default/build_solids/100
                        time:   [61.142 µs 61.548 µs 62.124 µs]
                        thrpt:  [1.6097 Melem/s 1.6248 Melem/s 1.6355 Melem/s]
                 change:
                        time:   [−1.1785% −0.4645% +0.2524%] (p = 0.24 > 0.05)
                        thrpt:  [−0.2518% +0.4667% +1.1925%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking backend_comparison/default/build_solids/1000
Benchmarking backend_comparison/default/build_solids/1000: Warming up for 3.0000 s
Benchmarking backend_comparison/default/build_solids/1000: Collecting 100 samples in estimated 6.4069 s (10k iterations)
Benchmarking backend_comparison/default/build_solids/1000: Analyzing
backend_comparison/default/build_solids/1000
                        time:   [636.72 µs 638.01 µs 639.39 µs]
                        thrpt:  [1.5640 Melem/s 1.5674 Melem/s 1.5705 Melem/s]
                 change:
                        time:   [−2.7889% −2.4232% −2.0271%] (p = 0.00 < 0.05)
                        thrpt:  [+2.0690% +2.4834% +2.8689%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking backend_comparison/default/build_solids/5000
Benchmarking backend_comparison/default/build_solids/5000: Warming up for 3.0000 s
Benchmarking backend_comparison/default/build_solids/5000: Collecting 100 samples in estimated 5.0853 s (1300 iterations)
Benchmarking backend_comparison/default/build_solids/5000: Analyzing
backend_comparison/default/build_solids/5000
                        time:   [3.3698 ms 3.4153 ms 3.4611 ms]
                        thrpt:  [1.4446 Melem/s 1.4640 Melem/s 1.4838 Melem/s]
                 change:
                        time:   [−11.321% −10.108% −8.8529%] (p = 0.00 < 0.05)
                        thrpt:  [+9.7127% +11.244% +12.766%]
                        Performance has improved.

     Running benches/builder.rs (target/release/deps/builder-ea562216731f4324)
Benchmarking builder/default/build_without_geometry
Benchmarking builder/default/build_without_geometry: Warming up for 3.0000 s

Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.2s, enable flat sampling, or reduce sample count to 50.
Benchmarking builder/default/build_without_geometry: Collecting 100 samples in estimated 7.2488 s (5050 iterations)
Benchmarking builder/default/build_without_geometry: Analyzing
builder/default/build_without_geometry
                        time:   [1.4340 ms 1.4369 ms 1.4402 ms]
                        thrpt:  [6.9433 Melem/s 6.9594 Melem/s 6.9737 Melem/s]
                 change:
                        time:   [−4.5397% −4.2966% −4.0647%] (p = 0.00 < 0.05)
                        thrpt:  [+4.2369% +4.4895% +4.7556%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking builder/default/build_with_geometry
Benchmarking builder/default/build_with_geometry: Warming up for 3.0000 s
Benchmarking builder/default/build_with_geometry: Collecting 100 samples in estimated 5.7333 s (200 iterations)
Benchmarking builder/default/build_with_geometry: Analyzing
builder/default/build_with_geometry
                        time:   [29.875 ms 30.283 ms 30.745 ms]
                        thrpt:  [325.25 Kelem/s 330.22 Kelem/s 334.73 Kelem/s]
                 change:
                        time:   [−1.2731% +0.8954% +3.0606%] (p = 0.44 > 0.05)
                        thrpt:  [−2.9697% −0.8874% +1.2895%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe

     Running benches/memory.rs (target/release/deps/memory-698931b2f061b799)
Benchmarking memory/default/u32/7000
Benchmarking memory/default/u32/7000: Warming up for 3.0000 s
Benchmarking memory/default/u32/7000: Collecting 100 samples in estimated 5.0866 s (700 iterations)
Benchmarking memory/default/u32/7000: Analyzing
memory/default/u32/7000 time:   [7.2077 ms 7.2452 ms 7.2878 ms]
                        change: [+1.7091% +2.3900% +3.0711%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 15 outliers among 100 measurements (15.00%)
  6 (6.00%) high mild
  9 (9.00%) high severe

     Running benches/processor.rs (target/release/deps/processor-bf3d58a809641e58)
Benchmarking default/compute_mean_coordinates_10k
Benchmarking default/compute_mean_coordinates_10k: Warming up for 3.0000 s
Benchmarking default/compute_mean_coordinates_10k: Collecting 100 samples in estimated 5.1439 s (15k iterations)
Benchmarking default/compute_mean_coordinates_10k: Analyzing
default/compute_mean_coordinates_10k
                        time:   [336.47 µs 338.35 µs 340.50 µs]
                        change: [+0.7397% +1.2786% +1.8842%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 16 outliers among 100 measurements (16.00%)
  4 (4.00%) high mild
  12 (12.00%) high severe


## Memory Profiling

   Compiling cityjson v0.3.0 (/home/balazs/Development/cityjson-rs)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.00s
     Running tests/v2_0/main.rs (target/debug/deps/v2_0-d4975ab243ce9342)

running 1 test
test producer_consumer_stream::test_producer_consumer_stream ... Consumer: Received global properties with 1 templates
Batch 0: 1000 buildings processed (total: 1000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 1: 1000 buildings processed (total: 2000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 2: 1000 buildings processed (total: 3000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 3: 1000 buildings processed (total: 4000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 4: 1000 buildings processed (total: 5000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 5: 1000 buildings processed (total: 6000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 6: 1000 buildings processed (total: 7000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 7: 1000 buildings processed (total: 8000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 8: 1000 buildings processed (total: 9000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 9: 1000 buildings processed (total: 10000), 13328 vertices, 2000 geometries, 9996 surfaces
Consumer: Received end-of-stream signal

========== Performance Summary ==========
Total buildings processed: 10000
Total batches: 11
Total geometries processed: 20000
Total surfaces processed: 99996
Peak vertices per batch: 13340
Peak CityObjects per batch: 1000
=========================================

Consumer finished in 2.44s
Producer finished in 2.45s

========== Overall Test Summary ==========
Total test duration: 2.47s
Throughput: 4050 buildings/sec
Average processing time per building: 0.247ms
==========================================

ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 2.50s

=== HEAP USAGE SUMMARY ===
Massif arguments:   --massif-out-file=./profiling/2025-11-18_58f79ce/massif.out --time-unit=B --detailed-freq=1 --max-snapshots=200 --threshold=0.1
    MB
   0 +----------------------------------------------------------------------->MB
Number of snapshots: 151
 Detailed snapshots: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 (peak), 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150]

=== TOP ALLOCATION SITES ===
00.00% (0B) (heap allocation functions) malloc/new/new[], --alloc-fns, etc.

--------------------------------------------------------------------------------
  n        time(B)         total(B)   useful-heap(B) extra-heap(B)    stacks(B)
--------------------------------------------------------------------------------
  1      5,006,904        1,309,176        1,153,792       155,384            0
88.13% (1,153,792B) (heap allocation functions) malloc/new/new[], --alloc-fns, etc.
->44.08% (577,024B) 0x41AFE39: alloc::raw_vec::finish_grow (in /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-d4975ab243ce9342)
| ->44.08% (577,024B) 0x41B0022: alloc::raw_vec::RawVecInner<A>::grow_amortized (in /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-d4975ab243ce9342)
|   ->17.52% (229,376B) 0x408EA62: grow_one<alloc::alloc::Global> (mod.rs:567)
|   | ->17.52% (229,376B) 0x408EA62: alloc::raw_vec::RawVec<T,A>::grow_one (mod.rs:340)
|   |   ->17.52% (229,376B) 0x40A947C: alloc::vec::Vec<T,A>::push_mut (mod.rs:2655)
|   |     ->17.52% (229,376B) 0x40A4F49: alloc::vec::Vec<T,A>::push (mod.rs:2572)
|   |       ->17.52% (229,376B) 0x4089BB6: <cityjson::resources::pool::DefaultResourcePool<T,RR> as cityjson::resources::pool::ResourcePool<T,RR>>::add (pool.rs:466)
|   |         ->17.52% (229,376B) 0x40783D0: cityjson::backend::default::citymodel::CityModelCore<C,VR,RR,SS,Semantic,Material,Texture,Geometry,Metadata,Transform,Extensions,CityObjects>::add_semantic (citymodel.rs:190)
|   |           ->17.52% (229,376B) 0x40E44A9: cityjson::v2_0::citymodel::CityModel<VR,RR,SS>::add_semantic (macros.rs:914)
|   |             ->17.52% (229,376B) 0x40E4239: <cityjson::v2_0::citymodel::CityModel<VR,RR,SS> as cityjson::backend::default::geometry::GeometryModelOps<VR,RR,cityjson::backend::default::coordinate::QuantizedCoordinate,cityjson::v2_0::geometry::semantic::Semantic<RR,SS>,cityjson::v2_0::appearance::material::Material<SS>,cityjson::v2_0::appearance::texture::Texture<SS>,cityjson::v2_0::geometry::Geometry<VR,RR,SS>,SS>>::add_semantic (citymodel.rs:93)
|   |               ->17.52% (229,376B) 0x40D6AB5: cityjson::backend::default::geometry::GeometryBuilder<VR,RR,C,Semantic,Material,Texture,Geometry,M,SS>::set_semantic_surface (geometry.rs:591)
|   |                 ->17.52% (229,376B) 0x40B7943: v2_0::producer_consumer_stream::build_geometry_from_wire (producer_consumer_stream.rs:1104)
|   |                   ->17.52% (229,376B) 0x40BA7C7: v2_0::producer_consumer_stream::consumer (producer_consumer_stream.rs:887)
|   |                     ->17.52% (229,376B) 0x40710B5: v2_0::producer_consumer_stream::test_producer_consumer_stream::{{closure}}::{{closure}} (producer_consumer_stream.rs:161)
|   |                       ->17.52% (229,376B) 0x408E0DA: std::sys::backtrace::__rust_begin_short_backtrace (backtrace.rs:158)
|   |                         ->17.52% (229,376B) 0x40F1DF9: std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}} (mod.rs:559)
|   |                           ->17.52% (229,376B) 0x40E32EB: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once (unwind_safe.rs:274)
|   |                             ->17.52% (229,376B) 0x40F9D77: std::panicking::catch_unwind::do_call (panicking.rs:590)
|   |                               ->17.52% (229,376B) 0x40F241A: __rust_try (in /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-d4975ab243ce9342)
|   |                                 ->17.52% (229,376B) 0x40F15BB: catch_unwind<core::result::Result<(), cityjson::error::Error>, core::panic::unwind_safe::AssertUnwindSafe<std::thread::{impl
|   |                                   ->17.52% (229,376B) 0x40F15BB: catch_unwind<core::panic::unwind_safe::AssertUnwindSafe<std::thread::{impl
|   |                                     ->17.52% (229,376B) 0x40F15BB: std::thread::Builder::spawn_unchecked_::{{closure}} (mod.rs:557)
|   |                                       ->17.52% (229,376B) 0x407C19D: core::ops::function::FnOnce::call_once{{vtable.shim}} (function.rs:250)
|   |                                         ->17.52% (229,376B) 0x417FBCE: call_once<(), dyn core::ops::function::FnOnce<(), Output=()>, alloc::alloc::Global> (boxed.rs:1985)
|   |                                           ->17.52% (229,376B) 0x417FBCE: std::sys::thread::unix::Thread::new::thread_start (unix.rs:126)
|   |                                             ->17.52% (229,376B) 0x4B0FAA3: start_thread (pthread_create.c:447)
|   |                                               ->17.52% (229,376B) 0x4B9CA63: clone (clone.S:100)
|   |                                                 
|   ->08.92% (116,736B) 0x408EF62: grow_one<alloc::alloc::Global> (mod.rs:567)
|   | ->08.92% (116,736B) 0x408EF62: alloc::raw_vec::RawVec<T,A>::grow_one (mod.rs:340)
|   |   ->08.92% (116,736B) 0x40A8D3C: alloc::vec::Vec<T,A>::push_mut (mod.rs:2655)
|   |     ->08.92% (116,736B) 0x40A4F89: alloc::vec::Vec<T,A>::push (mod.rs:2572)
|   |       ->08.92% (116,736B) 0x408B3EB: <cityjson::resources::pool::DefaultResourcePool<T,RR> as cityjson::resources::pool::ResourcePool<T,RR>>::add (pool.rs:466)
|   |         ->08.92% (116,736B) 0x4078350: cityjson::backend::default::citymodel::CityModelCore<C,VR,RR,SS,Semantic,Material,Texture,Geometry,Metadata,Transform,Extensions,CityObjects>::add_geometry (citymodel.rs:382)
|   |           ->08.92% (116,736B) 0x40E4489: cityjson::v2_0::citymodel::CityModel<VR,RR,SS>::add_geometry (macros.rs:1062)
|   |             ->08.92% (116,736B) 0x40E41F9: <cityjson::v2_0::citymodel::CityModel<VR,RR,SS> as cityjson::backend::default::geometry::GeometryModelOps<VR,RR,cityjson::backend::default::coordinate::QuantizedCoordinate,cityjson::v2_0::geometry::semantic::Semantic<RR,SS>,cityjson::v2_0::appearance::material::Material<SS>,cityjson::v2_0::appearance::texture::Texture<SS>,cityjson::v2_0::geometry::Geometry<VR,RR,SS>,SS>>::add_geometry (citymodel.rs:121)
|   |               ->08.92% (116,736B) 0x40DB9A4: cityjson::backend::default::geometry::GeometryBuilder<VR,RR,C,Semantic,Material,Texture,Geometry,M,SS>::build (geometry.rs:1061)
|   |                 ->08.92% (116,736B) 0x40B6F3F: v2_0::producer_consumer_stream::build_geometry_from_wire (producer_consumer_stream.rs:1118)
|   |                   ->08.92% (116,736B) 0x40BA7C7: v2_0::producer_consumer_stream::consumer (producer_consumer_stream.rs:887)
|   |                     ->08.92% (116,736B) 0x40710B5: v2_0::producer_consumer_stream::test_producer_consumer_stream::{{closure}}::{{closure}} (producer_consumer_stream.rs:161)
|   |                       ->08.92% (116,736B) 0x408E0DA: std::sys::backtrace::__rust_begin_short_backtrace (backtrace.rs:158)
|   |                         ->08.92% (116,736B) 0x40F1DF9: std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}} (mod.rs:559)
|   |                           ->08.92% (116,736B) 0x40E32EB: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once (unwind_safe.rs:274)

Full report saved to ./profiling/2025-11-18_58f79ce/massif-report.txt
Visualize with: massif-visualizer ./profiling/2025-11-18_58f79ce/massif.out
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/v2_0/main.rs (target/debug/deps/v2_0-d4975ab243ce9342)
==158558== Cachegrind, a high-precision tracing profiler
==158558== Copyright (C) 2002-2024, and GNU GPL'd, by Nicholas Nethercote et al.
==158558== Using Valgrind-3.26.0 and LibVEX; rerun with -h for copyright info
==158558== Command: /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-d4975ab243ce9342 test_producer_consumer_stream --test-threads=1 --nocapture
==158558== 
--158558-- warning: L3 cache found, using its data for the LL simulation.

running 1 test
test producer_consumer_stream::test_producer_consumer_stream ... Consumer: Received global properties with 1 templates
Batch 0: 1000 buildings processed (total: 1000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 1: 1000 buildings processed (total: 2000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 2: 1000 buildings processed (total: 3000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 3: 1000 buildings processed (total: 4000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 4: 1000 buildings processed (total: 5000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 5: 1000 buildings processed (total: 6000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 6: 1000 buildings processed (total: 7000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 7: 1000 buildings processed (total: 8000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 8: 1000 buildings processed (total: 9000), 13340 vertices, 2000 geometries, 10004 surfaces
Producer finished in 23.61s
Batch 9: 1000 buildings processed (total: 10000), 13328 vertices, 2000 geometries, 9996 surfaces
Consumer: Received end-of-stream signal

========== Performance Summary ==========
Total buildings processed: 10000
Total batches: 11
Total geometries processed: 20000
Total surfaces processed: 99996
Peak vertices per batch: 13340
Peak CityObjects per batch: 1000
=========================================

Consumer finished in 23.65s

========== Overall Test Summary ==========
Total test duration: 23.70s
Throughput: 422 buildings/sec
Average processing time per building: 2.370ms
==========================================

ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 23.74s

==158558== 
==158558== I refs:        5,646,680,815
==158558== I1  misses:      124,372,156
==158558== LLi misses:          712,548
==158558== I1  miss rate:          2.20%
==158558== LLi miss rate:          0.01%
==158558== 
==158558== D refs:        3,400,061,618  (1,510,600,673 rd   + 1,889,460,945 wr)
==158558== D1  misses:       16,107,834  (    8,499,176 rd   +     7,608,658 wr)
==158558== LLd misses:        2,978,246  (    1,563,427 rd   +     1,414,819 wr)
==158558== D1  miss rate:           0.5% (          0.6%     +           0.4%  )
==158558== LLd miss rate:           0.1% (          0.1%     +           0.1%  )
==158558== 
==158558== LL refs:         140,479,990  (  132,871,332 rd   +     7,608,658 wr)
==158558== LL misses:         3,690,794  (    2,275,975 rd   +     1,414,819 wr)
==158558== LL miss rate:            0.0% (          0.0%     +           0.1%  )
==158558== 
==158558== Branches:        442,959,683  (  402,060,214 cond +    40,899,469 ind)
==158558== Mispredicts:      17,830,542  (   16,040,993 cond +     1,789,549 ind)
==158558== Mispred rate:            4.0% (          4.0%     +           4.4%   )
================================
CACHE STATISTICS:
================================
--------------------------------------------------------------------------------
-- Metadata
--------------------------------------------------------------------------------
Invocation:       /snap/valgrind/181/usr/bin/cg_annotate ./profiling/2025-11-18_58f79ce/cachegrind.out
I1 cache:         32768 B, 64 B, 8-way associative
D1 cache:         49152 B, 64 B, 12-way associative
LL cache:         67108864 B, 64 B, direct-mapped
Command:          /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-d4975ab243ce9342 test_producer_consumer_stream --test-threads=1 --nocapture
Events recorded:  Ir I1mr ILmr Dr D1mr DLmr Dw D1mw DLmw Bc Bcm Bi Bim
Events shown:     Ir I1mr ILmr Dr D1mr DLmr Dw D1mw DLmw Bc Bcm Bi Bim
Event sort order: Ir I1mr ILmr Dr D1mr DLmr Dw D1mw DLmw Bc Bcm Bi Bim
Threshold:        0.1%
Annotation:       on

--------------------------------------------------------------------------------
-- Summary
--------------------------------------------------------------------------------
Ir____________________ I1mr________________ ILmr____________ Dr____________________ D1mr______________ DLmr______________ Dw____________________ D1mw______________ DLmw______________ Bc__________________ Bcm________________ Bi_________________ Bim_______________ 

5,646,680,815 (100.0%) 124,372,156 (100.0%) 712,548 (100.0%) 1,510,600,673 (100.0%) 8,499,176 (100.0%) 1,563,427 (100.0%) 1,889,460,945 (100.0%) 7,608,658 (100.0%) 1,414,819 (100.0%) 402,060,214 (100.0%) 16,040,993 (100.0%) 40,899,469 (100.0%) 1,789,549 (100.0%)  PROGRAM TOTALS

--------------------------------------------------------------------------------
-- File:function summary
--------------------------------------------------------------------------------
  Ir________________________ I1mr_____________________ ILmr__________________ Dr________________________ D1mr____________________ DLmr__________________ Dw________________________ D1mw____________________ DLmw__________________ Bc________________________ Bcm_____________________ Bi_______________________ Bim___________________  file:function

< 783,222,139 (13.9%, 13.9%)  8,331,346  (6.7%,  6.7%)     211  (0.0%,  0.0%) 196,913,060 (13.0%, 13.0%) 4,189,651 (49.3%, 49.3%) 771,566 (49.4%, 49.4%) 100,290,807  (5.3%,  5.3%)   735,144  (9.7%,  9.7%) 191,043 (13.5%, 13.5%) 134,345,100 (33.4%, 33.4%) 7,043,919 (43.9%, 43.9%)          0  (0.0%,  0.0%)       0  (0.0%,  0.0%)  ./malloc/./malloc/malloc.c:
  250,307,311  (4.4%)         4,106,565  (3.3%)             82  (0.0%)         47,045,759  (3.1%)          698,596  (8.2%)        132,618  (8.5%)         29,605,898  (1.6%)          673,615  (8.9%)        180,811 (12.8%)         44,033,604 (11.0%)        3,385,037 (21.1%)                 0                      0                   _int_malloc
  178,800,439  (3.2%)         1,187,866  (1.0%)             11  (0.0%)         46,737,730  (3.1%)          353,788  (4.2%)        144,775  (9.3%)         16,885,176  (0.9%)              405  (0.0%)              0                 31,053,325  (7.7%)          944,800  (5.9%)                 0                      0                   _int_free
  120,541,996  (2.1%)           694,508  (0.6%)              9  (0.0%)         34,192,987  (2.3%)          189,735  (2.2%)         21,485  (1.4%)         13,699,311  (0.7%)           11,883  (0.2%)              4  (0.0%)         25,134,806  (6.3%)          954,976  (6.0%)                 0                      0                   malloc
   69,057,036  (1.2%)           104,061  (0.1%)              7  (0.0%)         14,159,135  (0.9%)        1,405,410 (16.5%)        173,768 (11.1%)         12,136,747  (0.6%)            8,467  (0.1%)          1,506  (0.1%)          9,649,730  (2.4%)          797,259  (5.0%)                 0                      0                   malloc_consolidate
   66,107,449  (1.2%)            47,495  (0.0%)              5  (0.0%)         23,609,782  (1.6%)          510,718  (6.0%)        189,509 (12.1%)         14,165,877  (0.7%)              210  (0.0%)              0                  7,082,949  (1.8%)            4,661  (0.0%)                 0                      0                   free
   36,086,156  (0.6%)           419,953  (0.3%)              3  (0.0%)         15,401,676  (1.0%)          773,089  (9.1%)         54,849  (3.5%)          5,103,649  (0.3%)                0                      0                  7,093,757  (1.8%)          300,172  (1.9%)                 0                      0                   unlink_chunk.isra.0
   17,175,263  (0.3%)           375,526  (0.3%)             16  (0.0%)          3,618,583  (0.2%)           16,404  (0.2%)          1,758  (0.1%)          1,639,013  (0.1%)              183  (0.0%)              0                  3,539,710  (0.9%)          103,136  (0.6%)                 0                      0                   realloc
   15,756,731  (0.3%)           300,202  (0.2%)              7  (0.0%)          4,155,341  (0.3%)          160,014  (1.9%)         41,540  (2.7%)          2,840,113  (0.2%)           10,723  (0.1%)          3,948  (0.3%)          2,709,273  (0.7%)          205,342  (1.3%)                 0                      0                   _int_free_merge_chunk
   14,406,117  (0.3%)           425,592  (0.3%)             10  (0.0%)          2,792,318  (0.2%)           34,485  (0.4%)          5,167  (0.3%)          1,790,506  (0.1%)            7,123  (0.1%)          2,525  (0.2%)          1,846,335  (0.5%)          270,807  (1.7%)                 0                      0                   _int_realloc
    6,166,558  (0.1%)           161,184  (0.1%)              3  (0.0%)          2,556,800  (0.2%)              592  (0.0%)             14  (0.0%)          1,772,009  (0.1%)              167  (0.0%)              0                    535,461  (0.1%)           13,848  (0.1%)                 0                      0                   _int_free_maybe_consolidate

< 601,100,774 (10.6%, 24.5%)  6,538,929  (5.3%, 12.0%)   7,875  (1.1%,  1.1%) 227,633,020 (15.1%, 28.1%)         0  (0.0%, 49.3%)       0  (0.0%, 49.4%) 280,523,994 (14.8%, 20.2%)    40,808  (0.5%, 10.2%)       1  (0.0%, 13.5%)  10,945,274  (2.7%, 36.1%) 1,803,248 (11.2%, 55.2%)          0  (0.0%,  0.0%)       0  (0.0%,  0.0%)  /home/balazs/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/hash/sip.rs:
  180,299,196  (3.2%)         2,493,930  (2.0%)          2,470  (0.3%)         78,441,858  (5.2%)                0                      0                 99,515,790  (5.3%)                0                      0                          0                        0                         0                      0                   <core::hash::sip::Sip13Rounds as core::hash::sip::Sip>::d_rounds
  118,446,732  (2.1%)           570,827  (0.5%)            955  (0.1%)         50,449,534  (3.3%)                0                      0                 63,610,282  (3.4%)               83  (0.0%)              0                          0                        0                         0                      0                   <core::hash::sip::Sip13Rounds as core::hash::sip::Sip>::c_rounds
  111,205,880  (2.0%)         1,363,175  (1.1%)          1,102  (0.2%)         42,490,248  (2.8%)                0                      0                 40,226,704  (2.1%)            8,510  (0.1%)              0                  4,929,272  (1.2%)          502,373  (3.1%)                 0                      0                   <core::hash::sip::Hasher<S> as core::hash::Hasher>::write
   90,444,366  (1.6%)           801,085  (0.6%)          1,723  (0.2%)         25,416,990  (1.7%)                0                      0                 29,939,980  (1.6%)           16,674  (0.2%)              0                  6,016,002  (1.5%)        1,300,875  (8.1%)                 0                      0                   core::hash::sip::u8to64_le
   39,806,316  (0.7%)           246,718  (0.2%)            342  (0.0%)         17,561,610  (1.2%)                0                      0                 14,049,288  (0.7%)                0                      0                          0                        0                         0                      0                   <core::hash::sip::Hasher<S> as core::hash::Hasher>::finish
   32,781,672  (0.6%)           733,705  (0.6%)            745  (0.1%)          5,853,870  (0.4%)                0                      0                 18,732,384  (1.0%)           15,541  (0.2%)              1  (0.0%)                  0                        0                         0                      0                   <std::hash::random::RandomState as core::hash::BuildHasher>::build_hasher
   18,732,384  (0.3%)           270,034  (0.2%)            382  (0.1%)          5,853,870  (0.4%)                0                      0                  8,195,418  (0.4%)                0                      0                          0                        0                         0                      0                   core::hash::sip::Hasher<S>::reset
    7,042,680  (0.1%)            59,455  (0.0%)            156  (0.0%)          1,565,040  (0.1%)                0                      0                  3,912,600  (0.2%)                0                      0                          0                        0                         0                      0                   <core::hash::sip::Hasher<S> as core::hash::Hasher>::write_str

< 517,229,241  (9.2%, 33.7%)  2,421,850  (1.9%, 13.9%)   7,157  (1.0%,  2.1%)  42,839,781  (2.8%, 30.9%)         0  (0.0%, 49.3%)       0  (0.0%, 49.4%) 117,365,044  (6.2%, 26.4%)    36,014  (0.5%, 10.7%)       5  (0.0%, 13.5%)   4,709,765  (1.2%, 37.3%)    82,072  (0.5%, 55.7%)          0  (0.0%,  0.0%)       0  (0.0%,  0.0%)  /home/balazs/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:
  147,708,040  (2.6%)           497,053  (0.4%)            690  (0.1%)                  0                        0                      0                  7,385,402  (0.4%)           17,617  (0.2%)              1  (0.0%)                  0                        0                         0                      0                   core::ptr::copy_nonoverlapping::precondition_check
=== KEY METRICS ===
Full report saved to ./profiling/2025-11-18_58f79ce/cachegrind-report.txt
Analyze with: kcachegrind ./profiling/2025-11-18_58f79ce/cachegrind.out
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/v2_0/main.rs (target/debug/deps/v2_0-d4975ab243ce9342)

running 1 test
test producer_consumer_stream::test_producer_consumer_stream ... Consumer: Received global properties with 1 templates
Batch 0: 1000 buildings processed (total: 1000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 1: 1000 buildings processed (total: 2000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 2: 1000 buildings processed (total: 3000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 3: 1000 buildings processed (total: 4000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 4: 1000 buildings processed (total: 5000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 5: 1000 buildings processed (total: 6000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 6: 1000 buildings processed (total: 7000), 13328 vertices, 2000 geometries, 9996 surfaces
Batch 7: 1000 buildings processed (total: 8000), 13332 vertices, 2000 geometries, 10000 surfaces
Batch 8: 1000 buildings processed (total: 9000), 13340 vertices, 2000 geometries, 10004 surfaces
Batch 9: 1000 buildings processed (total: 10000), 13328 vertices, 2000 geometries, 9996 surfaces
Consumer: Received end-of-stream signal

========== Performance Summary ==========
Total buildings processed: 10000
Total batches: 11
Total geometries processed: 20000
Total surfaces processed: 99996
Peak vertices per batch: 13340
Peak CityObjects per batch: 1000
=========================================

Consumer finished in 40.91s
Producer finished in 40.81s

========== Overall Test Summary ==========
Total test duration: 40.94s
Throughput: 244 buildings/sec
Average processing time per building: 4.094ms
==========================================

ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 40.99s

================================
LEAK SUMMARY:
================================
==159030== LEAK SUMMARY:
==159030==    definitely lost: 0 bytes in 0 blocks
==159030==    indirectly lost: 0 bytes in 0 blocks
==159030==      possibly lost: 48 bytes in 1 blocks
==159030==    still reachable: 456 bytes in 1 blocks
==159030==         suppressed: 0 bytes in 0 blocks
==159030== 
==159030== ERROR SUMMARY: 1 errors from 1 contexts (suppressed: 0 from 0)
================================
HEAP SUMMARY:
================================
==159030== HEAP SUMMARY:
==159030==     in use at exit: 504 bytes in 2 blocks
==159030==   total heap usage: 2,570,765 allocs, 2,570,763 frees, 298,688,095 bytes allocated
==159030== 
==159030== Searching for pointers to 2 not-freed blocks
==159030== Checked 116,088 bytes
==159030== 
==159030== 48 bytes in 1 blocks are possibly lost in loss record 1 of 2
==159030==    at 0x4A2280F: malloc (vg_replace_malloc.c:447)
==159030==    by 0x41882F3: alloc (alloc.rs:94)
==159030==    by 0x41882F3: alloc_impl (alloc.rs:189)
Full report saved to ./profiling/2025-11-18_58f79ce/memcheck.log
No definite memory leaks detected
