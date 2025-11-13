# Benchmark Results: 2025-11-13 - flat_attributes

- **Date**: 2025-11-13 15-32-06
- **Commit**: fd902ce
- **Rust Version**: 1.91.0
- **System**: Linux 6.14.0-35-generic x86_64

## Criterion Benchmarks

   Compiling cityjson v0.2.0 (/home/balazs/Development/cityjson-rs)
    Finished `bench` profile [optimized + debuginfo] target(s) in 2.29s
     Running unittests src/lib.rs (target/release/deps/cityjson-d0fb167bc73cd152)

running 149 tests
test cityjson::core::attributes::tests::test_attribute_container ... ignored
test cityjson::core::attributes::tests::test_attribute_pool_basic ... ignored
test cityjson::core::attributes::tests::test_attribute_pool_maps ... ignored
test cityjson::core::attributes::tests::test_attribute_pool_vectors ... ignored
test cityjson::core::attributes::tests::test_clear ... ignored
test cityjson::core::attributes::tests::test_display_implementations ... ignored
test cityjson::core::attributes::tests::test_get_by_key ... ignored
test cityjson::core::attributes::tests::test_nested_maps_and_vectors ... ignored
test cityjson::core::attributes::tests::test_remove_and_reuse ... ignored
test cityjson::core::boundary::nested_tests::test_empty_nested_conversions ... ignored
test cityjson::core::boundary::nested_tests::test_multisolid_with_complex_structure ... ignored
test cityjson::core::boundary::nested_tests::test_nested_multilinestring_with_empty_linestrings ... ignored
test cityjson::core::boundary::nested_tests::test_nested_multisurface_with_empty_components ... ignored
test cityjson::core::boundary::nested_tests::test_type_alias_consistency ... ignored
test cityjson::core::boundary::tests::test_boundary_consistency ... ignored
test cityjson::core::boundary::tests::test_boundary_counter ... ignored
test cityjson::core::boundary::tests::test_boundary_type_detection ... ignored
test cityjson::core::boundary::tests::test_boundary_with_capacity ... ignored
test cityjson::core::boundary::tests::test_display_boundary_type ... ignored
test cityjson::core::boundary::tests::test_empty_boundary ... ignored
test cityjson::core::boundary::tests::test_multi_linestring_conversion ... ignored
test cityjson::core::boundary::tests::test_multi_point_conversion ... ignored
test cityjson::core::boundary::tests::test_multi_solid_conversion ... ignored
test cityjson::core::boundary::tests::test_multi_surface_conversion ... ignored
test cityjson::core::boundary::tests::test_solid_conversion ... ignored
test cityjson::core::coordinate::tests::test_flexible_coordinate ... ignored
test cityjson::core::coordinate::tests::test_quantized_coordinate ... ignored
test cityjson::core::coordinate::tests::test_real_world_coordinate ... ignored
test cityjson::core::coordinate::tests::test_uv_coordinate ... ignored
test cityjson::core::coordinate::tests::test_vertices16_limit ... ignored
test cityjson::core::coordinate::tests::test_vertices32_capacity ... ignored
test cityjson::core::coordinate::tests::test_vertices_default ... ignored
test cityjson::core::coordinate::tests::test_vertices_indexing ... ignored
test cityjson::core::coordinate::tests::test_vertices_methods ... ignored
test cityjson::core::extension::tests::test_extension ... ignored
test cityjson::core::extension::tests::test_extensions_add_get ... ignored
test cityjson::core::extension::tests::test_extensions_iteration ... ignored
test cityjson::core::extension::tests::test_extensions_remove_empty ... ignored
test cityjson::core::geometry::tests::test_geometry_template_and_instance ... ignored
test cityjson::core::geometry::tests::test_multilinestring ... ignored
test cityjson::core::geometry::tests::test_multipoint_with_add_point ... ignored
test cityjson::core::geometry::tests::test_multipoint_with_add_vertex ... ignored
test cityjson::core::geometry::tests::test_multipoint_with_mixed_adds ... ignored
test cityjson::core::geometry::tests::test_multipoint_with_semantics ... ignored
test cityjson::core::geometry::tests::test_multisolid ... ignored
test cityjson::core::geometry::tests::test_multisurface ... ignored
test cityjson::core::geometry::tests::test_solid ... ignored
test cityjson::core::transform::test::display ... ignored
test cityjson::core::vertex::tests::test_hash_and_equality ... ignored
test cityjson::core::vertex::tests::test_integer_to_vertex_index_conversion ... ignored
test cityjson::core::vertex::tests::test_to_usize_conversion ... ignored
test cityjson::core::vertex::tests::test_vertex_coordinate ... ignored
test cityjson::core::vertex::tests::test_vertex_index_conversion ... ignored
test cityjson::core::vertex::tests::test_vertex_index_creation ... ignored
test cityjson::core::vertex::tests::test_vertex_index_from_u32 ... ignored
test cityjson::core::vertex::tests::test_vertex_index_helpers ... ignored
test cityjson::core::vertex::tests::test_vertex_index_overflow - should panic ... ignored
test cityjson::core::vertex::tests::test_vertex_index_vec_trait ... ignored
test cityjson::core::vertex::tests::test_vertex_indices_sequence ... ignored
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

     Running benches/builder.rs (target/release/deps/builder-b9f745c4570301b4)
Benchmarking builder/build_cityobjects_without_geometry
Benchmarking builder/build_cityobjects_without_geometry: Warming up for 3.0000 s
Benchmarking builder/build_cityobjects_without_geometry: Collecting 100 samples in estimated 5.9316 s (500 iterations)
Benchmarking builder/build_cityobjects_without_geometry: Analyzing
builder/build_cityobjects_without_geometry
                        time:   [12.845 ms 13.415 ms 14.024 ms]
                        thrpt:  [713.05 Kelem/s 745.45 Kelem/s 778.53 Kelem/s]
                 change:
                        time:   [+8.1071% +13.017% +17.994%] (p = 0.00 < 0.05)
                        thrpt:  [−15.250% −11.518% −7.4991%]
                        Performance has regressed.
Found 16 outliers among 100 measurements (16.00%)
  8 (8.00%) high mild
  8 (8.00%) high severe

Benchmarking builder/build_cityobjects_with_geometry
Benchmarking builder/build_cityobjects_with_geometry: Warming up for 3.0000 s

Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.7s, or reduce sample count to 70.
Benchmarking builder/build_cityobjects_with_geometry: Collecting 100 samples in estimated 6.6512 s (100 iterations)
Benchmarking builder/build_cityobjects_with_geometry: Analyzing
builder/build_cityobjects_with_geometry
                        time:   [65.940 ms 67.312 ms 68.779 ms]
                        thrpt:  [145.39 Kelem/s 148.56 Kelem/s 151.65 Kelem/s]
                 change:
                        time:   [−10.158% −7.8473% −5.3029%] (p = 0.00 < 0.05)
                        thrpt:  [+5.5998% +8.5156% +11.306%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

     Running benches/memory.rs (target/release/deps/memory-f1de40a9872e87da)
Benchmarking memory/u16/7000
Benchmarking memory/u16/7000: Warming up for 3.0000 s
Benchmarking memory/u16/7000: Collecting 100 samples in estimated 5.0923 s (800 iterations)
Benchmarking memory/u16/7000: Analyzing
memory/u16/7000         time:   [6.3473 ms 6.3873 ms 6.4313 ms]
                        change: [+0.2776% +1.1293% +2.0842%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking memory/u32/7000
Benchmarking memory/u32/7000: Warming up for 3.0000 s
Benchmarking memory/u32/7000: Collecting 100 samples in estimated 5.5441 s (700 iterations)
Benchmarking memory/u32/7000: Analyzing
memory/u32/7000         time:   [7.8326 ms 7.9140 ms 8.0102 ms]
                        change: [−2.3449% −0.9547% +0.6405%] (p = 0.22 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

Benchmarking memory/u64/7000
Benchmarking memory/u64/7000: Warming up for 3.0000 s
Benchmarking memory/u64/7000: Collecting 100 samples in estimated 5.7890 s (700 iterations)
Benchmarking memory/u64/7000: Analyzing
memory/u64/7000         time:   [8.4599 ms 8.5575 ms 8.6711 ms]
                        change: [+6.3611% +7.9165% +9.7666%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

     Running benches/processor.rs (target/release/deps/processor-602cb5541bb1002a)
Benchmarking compute_mean_coordinates_10k
Benchmarking compute_mean_coordinates_10k: Warming up for 3.0000 s
Benchmarking compute_mean_coordinates_10k: Collecting 100 samples in estimated 5.3146 s (15k iterations)
Benchmarking compute_mean_coordinates_10k: Analyzing
compute_mean_coordinates_10k
                        time:   [345.43 µs 347.88 µs 350.52 µs]
                        change: [+4.2265% +4.7002% +5.1273%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 18 outliers among 100 measurements (18.00%)
  14 (14.00%) low severe
  4 (4.00%) high mild


## Memory Profiling

   Compiling cityjson v0.2.0 (/home/balazs/Development/cityjson-rs)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running tests/v2_0/main.rs (target/debug/deps/v2_0-b2a93070bb1e93b3)

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
Producer finished in 2.62s
Consumer: Received end-of-stream signal

========== Performance Summary ==========
Total buildings processed: 10000
Total batches: 11
Total geometries processed: 20000
Total surfaces processed: 99996
Peak vertices per batch: 13340
Peak CityObjects per batch: 1000
=========================================

Consumer finished in 2.63s

========== Overall Test Summary ==========
Total test duration: 2.63s
Throughput: 3799 buildings/sec
Average processing time per building: 0.263ms
==========================================

ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 2.67s

=== HEAP USAGE SUMMARY ===
Massif arguments:   --massif-out-file=./profiling/2025-11-13_fd902ce/massif.out --time-unit=B --detailed-freq=1 --max-snapshots=200 --threshold=0.1
    MB
   0 +----------------------------------------------------------------------->MB
Number of snapshots: 175
 Detailed snapshots: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 (peak), 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174]

=== TOP ALLOCATION SITES ===
00.00% (0B) (heap allocation functions) malloc/new/new[], --alloc-fns, etc.

--------------------------------------------------------------------------------
  n        time(B)         total(B)   useful-heap(B) extra-heap(B)    stacks(B)
--------------------------------------------------------------------------------
  1      2,421,856          748,560          667,717        80,843            0
89.20% (667,717B) (heap allocation functions) malloc/new/new[], --alloc-fns, etc.
->48.74% (364,832B) 0x41AFD09: alloc::raw_vec::finish_grow (in /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-b2a93070bb1e93b3)
| ->48.74% (364,832B) 0x41AFEF2: alloc::raw_vec::RawVecInner<A>::grow_amortized (in /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-b2a93070bb1e93b3)
|   ->07.80% (58,368B) 0x40FD502: grow_one<alloc::alloc::Global> (mod.rs:567)
|   | ->07.80% (58,368B) 0x40FD502: alloc::raw_vec::RawVec<T,A>::grow_one (mod.rs:340)
|   |   ->07.80% (58,368B) 0x40C827C: alloc::vec::Vec<T,A>::push_mut (mod.rs:2655)
|   |     ->07.80% (58,368B) 0x40C5B19: alloc::vec::Vec<T,A>::push (mod.rs:2572)
|   |       ->07.80% (58,368B) 0x4071C6B: <cityjson::resources::pool::DefaultResourcePool<T,RR> as cityjson::resources::pool::ResourcePool<T,RR>>::add (pool.rs:466)
|   |         ->07.80% (58,368B) 0x407B9E0: cityjson::cityjson::core::citymodel::CityModelCore<C,VR,RR,SS,Semantic,Material,Texture,Geometry,Metadata,Transform,Extensions,CityObjects>::add_geometry (citymodel.rs:378)
|   |           ->07.80% (58,368B) 0x4091A19: cityjson::v2_0::citymodel::CityModel<VR,RR,SS>::add_geometry (macros.rs:1062)
|   |             ->07.80% (58,368B) 0x40917A9: <cityjson::v2_0::citymodel::CityModel<VR,RR,SS> as cityjson::cityjson::core::geometry::GeometryModelOps<VR,RR,cityjson::cityjson::core::coordinate::QuantizedCoordinate,cityjson::v2_0::geometry::semantic::Semantic<RR,SS>,cityjson::v2_0::appearance::material::Material<SS>,cityjson::v2_0::appearance::texture::Texture<SS>,cityjson::v2_0::geometry::Geometry<VR,RR,SS>,SS>>::add_geometry (citymodel.rs:121)
|   |               ->07.80% (58,368B) 0x408E394: cityjson::cityjson::core::geometry::GeometryBuilder<VR,RR,C,Semantic,Material,Texture,Geometry,M,SS>::build (geometry.rs:1051)
|   |                 ->07.80% (58,368B) 0x40EA9AF: v2_0::producer_consumer_stream::build_geometry_from_wire (producer_consumer_stream.rs:1118)
|   |                   ->07.80% (58,368B) 0x40EE228: v2_0::producer_consumer_stream::consumer (producer_consumer_stream.rs:887)
|   |                     ->07.80% (58,368B) 0x40E9105: v2_0::producer_consumer_stream::test_producer_consumer_stream::{{closure}}::{{closure}} (producer_consumer_stream.rs:161)
|   |                       ->07.80% (58,368B) 0x40A4FCA: std::sys::backtrace::__rust_begin_short_backtrace (backtrace.rs:158)
|   |                         ->07.80% (58,368B) 0x40D1929: std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}} (mod.rs:559)
|   |                           ->07.80% (58,368B) 0x407490B: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once (unwind_safe.rs:274)
|   |                             ->07.80% (58,368B) 0x40D4367: std::panicking::catch_unwind::do_call (panicking.rs:590)
|   |                               ->07.80% (58,368B) 0x40D1F4A: __rust_try (in /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-b2a93070bb1e93b3)
|   |                                 ->07.80% (58,368B) 0x40D0DAB: catch_unwind<core::result::Result<(), cityjson::error::Error>, core::panic::unwind_safe::AssertUnwindSafe<std::thread::{impl
|   |                                   ->07.80% (58,368B) 0x40D0DAB: catch_unwind<core::panic::unwind_safe::AssertUnwindSafe<std::thread::{impl
|   |                                     ->07.80% (58,368B) 0x40D0DAB: std::thread::Builder::spawn_unchecked_::{{closure}} (mod.rs:557)
|   |                                       ->07.80% (58,368B) 0x40DB30D: core::ops::function::FnOnce::call_once{{vtable.shim}} (function.rs:250)
|   |                                         ->07.80% (58,368B) 0x417FA9E: call_once<(), dyn core::ops::function::FnOnce<(), Output=()>, alloc::alloc::Global> (boxed.rs:1985)
|   |                                           ->07.80% (58,368B) 0x417FA9E: std::sys::thread::unix::Thread::new::thread_start (unix.rs:126)
|   |                                             ->07.80% (58,368B) 0x4B0FAA3: start_thread (pthread_create.c:447)
|   |                                               ->07.80% (58,368B) 0x4B9CA63: clone (clone.S:100)
|   |                                                 
|   ->07.66% (57,344B) 0x40FCBB2: grow_one<alloc::alloc::Global> (mod.rs:567)
|   | ->07.66% (57,344B) 0x40FCBB2: alloc::raw_vec::RawVec<T,A>::grow_one (mod.rs:340)
|   |   ->07.66% (57,344B) 0x40C94EC: alloc::vec::Vec<T,A>::push_mut (mod.rs:2655)
|   |     ->07.66% (57,344B) 0x40C5C39: alloc::vec::Vec<T,A>::push (mod.rs:2572)
|   |       ->07.66% (57,344B) 0x40710A6: <cityjson::resources::pool::DefaultResourcePool<T,RR> as cityjson::resources::pool::ResourcePool<T,RR>>::add (pool.rs:466)
|   |         ->07.66% (57,344B) 0x407BA60: cityjson::cityjson::core::citymodel::CityModelCore<C,VR,RR,SS,Semantic,Material,Texture,Geometry,Metadata,Transform,Extensions,CityObjects>::add_semantic (citymodel.rs:186)
|   |           ->07.66% (57,344B) 0x4091A39: cityjson::v2_0::citymodel::CityModel<VR,RR,SS>::add_semantic (macros.rs:914)
|   |             ->07.66% (57,344B) 0x40917C9: <cityjson::v2_0::citymodel::CityModel<VR,RR,SS> as cityjson::cityjson::core::geometry::GeometryModelOps<VR,RR,cityjson::cityjson::core::coordinate::QuantizedCoordinate,cityjson::v2_0::geometry::semantic::Semantic<RR,SS>,cityjson::v2_0::appearance::material::Material<SS>,cityjson::v2_0::appearance::texture::Texture<SS>,cityjson::v2_0::geometry::Geometry<VR,RR,SS>,SS>>::add_semantic (citymodel.rs:93)
|   |               ->07.66% (57,344B) 0x4085155: cityjson::cityjson::core::geometry::GeometryBuilder<VR,RR,C,Semantic,Material,Texture,Geometry,M,SS>::set_semantic_surface (geometry.rs:581)
|   |                 ->07.66% (57,344B) 0x40EB3B3: v2_0::producer_consumer_stream::build_geometry_from_wire (producer_consumer_stream.rs:1104)
|   |                   ->07.66% (57,344B) 0x40EE228: v2_0::producer_consumer_stream::consumer (producer_consumer_stream.rs:887)
|   |                     ->07.66% (57,344B) 0x40E9105: v2_0::producer_consumer_stream::test_producer_consumer_stream::{{closure}}::{{closure}} (producer_consumer_stream.rs:161)
|   |                       ->07.66% (57,344B) 0x40A4FCA: std::sys::backtrace::__rust_begin_short_backtrace (backtrace.rs:158)
|   |                         ->07.66% (57,344B) 0x40D1929: std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}} (mod.rs:559)
|   |                           ->07.66% (57,344B) 0x407490B: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once (unwind_safe.rs:274)

Full report saved to ./profiling/2025-11-13_fd902ce/massif-report.txt
Visualize with: massif-visualizer ./profiling/2025-11-13_fd902ce/massif.out
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/v2_0/main.rs (target/debug/deps/v2_0-b2a93070bb1e93b3)
==1620445== Cachegrind, a high-precision tracing profiler
==1620445== Copyright (C) 2002-2024, and GNU GPL'd, by Nicholas Nethercote et al.
==1620445== Using Valgrind-3.26.0 and LibVEX; rerun with -h for copyright info
==1620445== Command: /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-b2a93070bb1e93b3 test_producer_consumer_stream --test-threads=1 --nocapture
==1620445== 
--1620445-- warning: L3 cache found, using its data for the LL simulation.

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
Producer finished in 24.36s
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

Consumer finished in 24.45s

========== Overall Test Summary ==========
Total test duration: 24.46s
Throughput: 409 buildings/sec
Average processing time per building: 2.446ms
==========================================

ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 24.49s

==1620445== 
==1620445== I refs:        5,685,105,865
==1620445== I1  misses:      124,533,479
==1620445== LLi misses:          676,057
==1620445== I1  miss rate:          2.19%
==1620445== LLi miss rate:          0.01%
==1620445== 
==1620445== D refs:        3,460,172,704  (1,538,240,904 rd   + 1,921,931,800 wr)
==1620445== D1  misses:       17,215,775  (    8,967,205 rd   +     8,248,570 wr)
==1620445== LLd misses:        3,267,017  (    1,641,901 rd   +     1,625,116 wr)
==1620445== D1  miss rate:           0.5% (          0.6%     +           0.4%  )
==1620445== LLd miss rate:           0.1% (          0.1%     +           0.1%  )
==1620445== 
==1620445== LL refs:         141,749,254  (  133,500,684 rd   +     8,248,570 wr)
==1620445== LL misses:         3,943,074  (    2,317,958 rd   +     1,625,116 wr)
==1620445== LL miss rate:            0.0% (          0.0%     +           0.1%  )
==1620445== 
==1620445== Branches:        461,217,616  (  420,042,839 cond +    41,174,777 ind)
==1620445== Mispredicts:      16,965,406  (   15,506,523 cond +     1,458,883 ind)
==1620445== Mispred rate:            3.7% (          3.7%     +           3.5%   )
================================
CACHE STATISTICS:
================================
--------------------------------------------------------------------------------
-- Metadata
--------------------------------------------------------------------------------
Invocation:       /snap/valgrind/181/usr/bin/cg_annotate ./profiling/2025-11-13_fd902ce/cachegrind.out
I1 cache:         32768 B, 64 B, 8-way associative
D1 cache:         49152 B, 64 B, 12-way associative
LL cache:         67108864 B, 64 B, direct-mapped
Command:          /home/balazs/Development/cityjson-rs/target/debug/deps/v2_0-b2a93070bb1e93b3 test_producer_consumer_stream --test-threads=1 --nocapture
Events recorded:  Ir I1mr ILmr Dr D1mr DLmr Dw D1mw DLmw Bc Bcm Bi Bim
Events shown:     Ir I1mr ILmr Dr D1mr DLmr Dw D1mw DLmw Bc Bcm Bi Bim
Event sort order: Ir I1mr ILmr Dr D1mr DLmr Dw D1mw DLmw Bc Bcm Bi Bim
Threshold:        0.1%
Annotation:       on

--------------------------------------------------------------------------------
-- Summary
--------------------------------------------------------------------------------
Ir____________________ I1mr________________ ILmr____________ Dr____________________ D1mr______________ DLmr______________ Dw____________________ D1mw______________ DLmw______________ Bc__________________ Bcm________________ Bi_________________ Bim_______________ 

5,685,105,865 (100.0%) 124,533,479 (100.0%) 676,057 (100.0%) 1,538,240,904 (100.0%) 8,967,205 (100.0%) 1,641,901 (100.0%) 1,921,931,800 (100.0%) 8,248,570 (100.0%) 1,625,116 (100.0%) 420,042,839 (100.0%) 15,506,523 (100.0%) 41,174,777 (100.0%) 1,458,883 (100.0%)  PROGRAM TOTALS

--------------------------------------------------------------------------------
-- File:function summary
--------------------------------------------------------------------------------
  Ir________________________ I1mr_____________________ ILmr__________________ Dr________________________ D1mr____________________ DLmr__________________ Dw________________________ D1mw____________________ DLmw__________________ Bc________________________ Bcm_____________________ Bi_______________________ Bim___________________  file:function

< 782,578,419 (13.8%, 13.8%)  8,416,754  (6.8%,  6.8%)     212  (0.0%,  0.0%) 196,807,710 (12.8%, 12.8%) 4,199,040 (46.8%, 46.8%) 769,762 (46.9%, 46.9%) 100,235,109  (5.2%,  5.2%)   746,367  (9.0%,  9.0%) 200,960 (12.4%, 12.4%) 134,178,904 (31.9%, 31.9%) 6,882,431 (44.4%, 44.4%)          0  (0.0%,  0.0%)       0  (0.0%,  0.0%)  ./malloc/./malloc/malloc.c:
  249,793,877  (4.4%)         3,538,138  (2.8%)             84  (0.0%)         46,979,863  (3.1%)          704,541  (7.9%)        156,603  (9.5%)         29,610,255  (1.5%)          673,535  (8.2%)        190,338 (11.7%)         43,857,924 (10.4%)        3,375,286 (21.8%)                 0                      0                   _int_malloc
  178,849,245  (3.1%)         1,600,811  (1.3%)             11  (0.0%)         46,743,269  (3.0%)          355,974  (4.0%)        135,638  (8.3%)         16,869,707  (0.9%)              488  (0.0%)              0                 31,079,653  (7.4%)          854,537  (5.5%)                 0                      0                   _int_free
  120,562,962  (2.1%)           271,273  (0.2%)              9  (0.0%)         34,200,503  (2.2%)          196,349  (2.2%)         19,287  (1.2%)         13,696,190  (0.7%)           20,090  (0.2%)              5  (0.0%)         25,145,425  (6.0%)          926,109  (6.0%)                 0                      0                   malloc
   68,614,677  (1.2%)           103,028  (0.1%)              7  (0.0%)         14,072,533  (0.9%)        1,397,043 (15.6%)        181,538 (11.1%)         12,061,805  (0.6%)            6,737  (0.1%)          1,321  (0.1%)          9,586,274  (2.3%)          814,363  (5.3%)                 0                      0                   malloc_consolidate
   66,107,173  (1.2%)           553,284  (0.4%)              5  (0.0%)         23,609,682  (1.5%)          520,643  (5.8%)        180,690 (11.0%)         14,165,815  (0.7%)              205  (0.0%)              0                  7,082,922  (1.7%)            4,576  (0.0%)                 0                      0                   free
   35,990,745  (0.6%)           437,466  (0.4%)              3  (0.0%)         15,359,582  (1.0%)          760,590  (8.5%)         49,246  (3.0%)          5,086,671  (0.3%)                0                      0                  7,073,949  (1.7%)          315,903  (2.0%)                 0                      0                   unlink_chunk.isra.0
   17,176,922  (0.3%)           471,967  (0.4%)             15  (0.0%)          3,618,872  (0.2%)           17,703  (0.2%)          1,378  (0.1%)          1,639,116  (0.1%)              196  (0.0%)              0                  3,540,102  (0.8%)          153,250  (1.0%)                 0                      0                   realloc
   15,832,485  (0.3%)           283,855  (0.2%)              7  (0.0%)          4,173,471  (0.3%)          160,398  (1.8%)         38,716  (2.4%)          2,855,423  (0.1%)           11,159  (0.1%)          4,136  (0.3%)          2,722,289  (0.6%)          171,960  (1.1%)                 0                      0                   _int_free_merge_chunk
   14,406,197  (0.3%)           493,942  (0.4%)             10  (0.0%)          2,792,169  (0.2%)           34,846  (0.4%)          3,972  (0.2%)          1,790,598  (0.1%)            7,752  (0.1%)          2,823  (0.2%)          1,846,238  (0.4%)          210,327  (1.4%)                 0                      0                   _int_realloc
    6,244,521  (0.1%)           158,419  (0.1%)              3  (0.0%)          2,586,720  (0.2%)              700  (0.0%)             13  (0.0%)          1,790,931  (0.1%)              303  (0.0%)              0                    545,185  (0.1%)           14,493  (0.1%)                 0                      0                   _int_free_maybe_consolidate

< 601,100,774 (10.6%, 24.3%)  5,720,882  (4.6%, 11.4%)   7,685  (1.1%,  1.2%) 227,633,020 (14.8%, 27.6%)         1  (0.0%, 46.8%)       0  (0.0%, 46.9%) 280,523,994 (14.6%, 19.8%)    23,538  (0.3%,  9.3%)       1  (0.0%, 12.4%)  10,945,274  (2.6%, 34.5%) 1,457,954  (9.4%, 53.8%)          0  (0.0%,  0.0%)       0  (0.0%,  0.0%)  /home/balazs/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/hash/sip.rs:
  180,299,196  (3.2%)         2,442,061  (2.0%)          2,867  (0.4%)         78,441,858  (5.1%)                0                      0                 99,515,790  (5.2%)                0                      0                          0                        0                         0                      0                   <core::hash::sip::Sip13Rounds as core::hash::sip::Sip>::d_rounds
  118,446,732  (2.1%)           676,910  (0.5%)            689  (0.1%)         50,449,534  (3.3%)                0                      0                 63,610,282  (3.3%)              110  (0.0%)              0                          0                        0                         0                      0                   <core::hash::sip::Sip13Rounds as core::hash::sip::Sip>::c_rounds
  111,205,880  (2.0%)           574,240  (0.5%)          1,241  (0.2%)         42,490,248  (2.8%)                1  (0.0%)              0                 40,226,704  (2.1%)            2,537  (0.0%)              0                  4,929,272  (1.2%)          460,429  (3.0%)                 0                      0                   <core::hash::sip::Hasher<S> as core::hash::Hasher>::write
   90,444,366  (1.6%)           855,625  (0.7%)          1,526  (0.2%)         25,416,990  (1.7%)                0                      0                 29,939,980  (1.6%)            7,961  (0.1%)              0                  6,016,002  (1.4%)          997,525  (6.4%)                 0                      0                   core::hash::sip::u8to64_le
   39,806,316  (0.7%)           395,629  (0.3%)            542  (0.1%)         17,561,610  (1.1%)                0                      0                 14,049,288  (0.7%)                0                      0                          0                        0                         0                      0                   <core::hash::sip::Hasher<S> as core::hash::Hasher>::finish
   32,781,672  (0.6%)           432,390  (0.3%)            250  (0.0%)          5,853,870  (0.4%)                0                      0                 18,732,384  (1.0%)           12,930  (0.2%)              1  (0.0%)                  0                        0                         0                      0                   <std::hash::random::RandomState as core::hash::BuildHasher>::build_hasher
   18,732,384  (0.3%)           299,880  (0.2%)            294  (0.0%)          5,853,870  (0.4%)                0                      0                  8,195,418  (0.4%)                0                      0                          0                        0                         0                      0                   core::hash::sip::Hasher<S>::reset
    7,042,680  (0.1%)            44,147  (0.0%)            276  (0.0%)          1,565,040  (0.1%)                0                      0                  3,912,600  (0.2%)                0                      0                          0                        0                         0                      0                   <core::hash::sip::Hasher<S> as core::hash::Hasher>::write_str

< 511,427,325  (9.0%, 33.3%)  2,010,373  (1.6%, 13.0%)   5,792  (0.9%,  2.0%)  42,762,840  (2.8%, 30.4%)         0  (0.0%, 46.8%)       0  (0.0%, 46.9%) 117,074,364  (6.1%, 25.9%)    32,304  (0.4%,  9.7%)       5  (0.0%, 12.4%)   4,709,652  (1.1%, 35.7%)    50,670  (0.3%, 54.1%)          0  (0.0%,  0.0%)       0  (0.0%,  0.0%)  /home/balazs/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:
  147,703,520  (2.6%)            82,502  (0.1%)            963  (0.1%)                  0                        0                      0                  7,385,176  (0.4%)           12,574  (0.2%)              1  (0.0%)                  0                        0                         0                      0                   core::ptr::copy_nonoverlapping::precondition_check
=== KEY METRICS ===
Full report saved to ./profiling/2025-11-13_fd902ce/cachegrind-report.txt
Analyze with: kcachegrind ./profiling/2025-11-13_fd902ce/cachegrind.out
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/v2_0/main.rs (target/debug/deps/v2_0-b2a93070bb1e93b3)

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
Producer finished in 42.08s
Consumer: Received end-of-stream signal

========== Performance Summary ==========
Total buildings processed: 10000
Total batches: 11
Total geometries processed: 20000
Total surfaces processed: 99996
Peak vertices per batch: 13340
Peak CityObjects per batch: 1000
=========================================

Consumer finished in 42.10s

========== Overall Test Summary ==========
Total test duration: 42.12s
Throughput: 237 buildings/sec
Average processing time per building: 4.212ms
==========================================

ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 42.17s

================================
LEAK SUMMARY:
================================
==1623443== LEAK SUMMARY:
==1623443==    definitely lost: 0 bytes in 0 blocks
==1623443==    indirectly lost: 0 bytes in 0 blocks
==1623443==      possibly lost: 48 bytes in 1 blocks
==1623443==    still reachable: 456 bytes in 1 blocks
==1623443==         suppressed: 0 bytes in 0 blocks
==1623443== 
==1623443== ERROR SUMMARY: 1 errors from 1 contexts (suppressed: 0 from 0)
================================
HEAP SUMMARY:
================================
==1623443== HEAP SUMMARY:
==1623443==     in use at exit: 504 bytes in 2 blocks
==1623443==   total heap usage: 2,570,755 allocs, 2,570,753 frees, 335,960,975 bytes allocated
==1623443== 
==1623443== Searching for pointers to 2 not-freed blocks
==1623443== Checked 116,024 bytes
==1623443== 
==1623443== 48 bytes in 1 blocks are possibly lost in loss record 1 of 2
==1623443==    at 0x4A2280F: malloc (vg_replace_malloc.c:447)
==1623443==    by 0x41881C3: alloc (alloc.rs:94)
==1623443==    by 0x41881C3: alloc_impl (alloc.rs:189)
Full report saved to ./profiling/2025-11-13_fd902ce/memcheck.log
No definite memory leaks detected
