use std::env;
use std::hint::black_box;
use std::ptr;
use std::time::Instant;

use cityjson_lib::ops;
use cityjson_lib::CityModel;
use cityjson_lib_ffi_core::exports::{
    cj_bytes_free, cj_bytes_list_free, cj_geometry_types_free, cj_model_append_model,
    cj_model_create, cj_model_free, cj_model_parse_document_bytes, cj_model_serialize_document,
    cj_model_set_transform, cj_model_clear_cityobject_geometry,
};
use cityjson_lib_ffi_core::{
    cj_bytes_list_t, cj_bytes_t, cj_geometry_type_t, cj_geometry_types_t, cj_model_t,
    cj_model_type_t, cj_status_t, cj_string_view_t, cj_transform_t,
};

const SMALL_FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/data/v2_0/minimal.city.json"
));
const LARGE_FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/data/v2_0/cityjson_fake_complete.city.json"
));
const APPEND_SOURCE_FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/data/v2_0/minimal.city.json"
));

#[derive(Clone, Copy)]
struct Fixture {
    name: &'static str,
    bytes: &'static [u8],
}

#[derive(Clone, Copy)]
struct Mode {
    label: &'static str,
    iterations: usize,
    repeats: usize,
    append_repeats: usize,
}

#[derive(Clone)]
struct Timing {
    elapsed_ns: u128,
    elapsed_per_iteration_ns: f64,
}

fn main() {
    let quick = env::args().any(|arg| arg == "--quick");
    let mode = if quick {
        Mode {
            label: "quick",
            iterations: 250,
            repeats: 7,
            append_repeats: 11,
        }
    } else {
        Mode {
            label: "full",
            iterations: 1_000,
            repeats: 9,
            append_repeats: 15,
        }
    };

    let fixtures = [
        Fixture {
            name: "small",
            bytes: SMALL_FIXTURE,
        },
        Fixture {
            name: "medium",
            bytes: LARGE_FIXTURE,
        },
    ];

    let results = fixtures
        .iter()
        .flat_map(|fixture| benchmark_fixture(*fixture, mode))
        .chain(benchmark_append(mode))
        .collect::<Vec<_>>();

    let output = serde_json::json!({
        "language": "rust",
        "mode": mode.label,
        "results": results,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn benchmark_fixture(fixture: Fixture, mode: Mode) -> Vec<serde_json::Value> {
    let direct_parse = measure_iterations(mode.iterations, mode.repeats, || {
        black_box(cityjson_lib::json::from_slice(black_box(fixture.bytes)).unwrap())
    });
    let abi_parse = measure_iterations(mode.iterations, mode.repeats, || {
        let handle = parse_document_abi(black_box(fixture.bytes));
        assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
        1usize
    });

    let direct_model = cityjson_lib::json::from_slice(fixture.bytes).unwrap();
    let abi_model = parse_document_abi(fixture.bytes);

    let direct_serialize = measure_iterations(mode.iterations, mode.repeats, || {
        black_box(
            cityjson_lib::json::to_vec_raw(
                black_box(&direct_model),
                &cityjson_lib::json::JsonWriteOptions::default(),
            )
            .unwrap(),
        )
    });
    let abi_serialize = measure_iterations(mode.iterations, mode.repeats, || {
        let mut payload = cj_bytes_t::null();
        assert_eq!(
            cj_model_serialize_document(black_box(abi_model), &mut payload),
            cj_status_t::CJ_STATUS_SUCCESS
        );
        black_box(take_bytes(payload))
    });

    let direct_cityobject_ids = measure_iterations(mode.iterations, mode.repeats, || {
        black_box(cityobject_ids_direct(black_box(&direct_model)))
    });
    let abi_cityobject_ids = measure_iterations(mode.iterations, mode.repeats, || {
        black_box(cityobject_ids_abi(black_box(abi_model)))
    });

    let direct_geometry_types = measure_iterations(mode.iterations, mode.repeats, || {
        black_box(geometry_types_direct(black_box(&direct_model)))
    });
    let abi_geometry_types = measure_iterations(mode.iterations, mode.repeats, || {
        black_box(geometry_types_abi(black_box(abi_model)))
    });

    assert_eq!(cj_model_free(abi_model), cj_status_t::CJ_STATUS_SUCCESS);
    black_box(direct_model);

    vec![
        result_json(
            fixture.name,
            mode,
            "parse",
            "direct",
            mode.iterations,
            mode.repeats,
            &direct_parse,
        ),
        result_json(
            fixture.name,
            mode,
            "parse",
            "abi",
            mode.iterations,
            mode.repeats,
            &abi_parse,
        ),
        result_json(
            fixture.name,
            mode,
            "serialize",
            "direct",
            mode.iterations,
            mode.repeats,
            &direct_serialize,
        ),
        result_json(
            fixture.name,
            mode,
            "serialize",
            "abi",
            mode.iterations,
            mode.repeats,
            &abi_serialize,
        ),
        result_json(
            fixture.name,
            mode,
            "cityobject_ids",
            "direct",
            mode.iterations,
            mode.repeats,
            &direct_cityobject_ids,
        ),
        result_json(
            fixture.name,
            mode,
            "cityobject_ids",
            "abi",
            mode.iterations,
            mode.repeats,
            &abi_cityobject_ids,
        ),
        result_json(
            fixture.name,
            mode,
            "geometry_types",
            "direct",
            mode.iterations,
            mode.repeats,
            &direct_geometry_types,
        ),
        result_json(
            fixture.name,
            mode,
            "geometry_types",
            "abi",
            mode.iterations,
            mode.repeats,
            &abi_geometry_types,
        ),
    ]
}

fn benchmark_append(mode: Mode) -> Vec<serde_json::Value> {
    let source = Fixture {
        name: "empty<-small",
        bytes: APPEND_SOURCE_FIXTURE,
    };

    let direct_append = measure_append_direct(mode.append_repeats, || {
        let mut target = cityjson_lib::CityModel::new(cityjson_lib::cityjson::CityModelType::CityJSON);
        apply_unit_transform(&mut target);
        let mut source_model = cityjson_lib::json::from_slice(source.bytes).unwrap();
        apply_unit_transform(&mut source_model);
        clear_cityobject_geometries_direct(&mut source_model);
        (target, source_model)
    });
    let abi_append = measure_append_abi(mode.append_repeats, || {
        let mut target = ptr::null_mut();
        assert_eq!(
            cj_model_create(cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON, &mut target),
            cj_status_t::CJ_STATUS_SUCCESS
        );
        assert_eq!(
            cj_model_set_transform(target, unit_transform()),
            cj_status_t::CJ_STATUS_SUCCESS
        );

        let source_handle = parse_document_abi(source.bytes);
        assert_eq!(
            cj_model_set_transform(source_handle, unit_transform()),
            cj_status_t::CJ_STATUS_SUCCESS
        );
        clear_cityobject_geometries_abi(source_handle);
        (target, source_handle)
    });

    vec![
        result_json(
            source.name,
            mode,
            "append",
            "direct",
            1,
            mode.append_repeats,
            &direct_append,
        ),
        result_json(
            source.name,
            mode,
            "append",
            "abi",
            1,
            mode.append_repeats,
            &abi_append,
        ),
    ]
}

fn measure_iterations<T, F>(iterations: usize, repeats: usize, mut action: F) -> Timing
where
    F: FnMut() -> T,
{
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let started = Instant::now();
        for _ in 0..iterations {
            black_box(action());
        }
        samples.push(started.elapsed().as_nanos());
    }

    summarize_samples(samples, iterations)
}

fn measure_append_direct<F>(repeats: usize, mut setup: F) -> Timing
where
    F: FnMut() -> (CityModel, CityModel),
{
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (mut target, source) = setup();
        let started = Instant::now();
        ops::append(&mut target, &source).unwrap();
        samples.push(started.elapsed().as_nanos());
        black_box(target);
        black_box(source);
    }

    summarize_samples(samples, 1)
}

fn measure_append_abi<F>(repeats: usize, mut setup: F) -> Timing
where
    F: FnMut() -> (*mut cj_model_t, *mut cj_model_t),
{
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (target, source) = setup();
        let started = Instant::now();
        assert_eq!(cj_model_append_model(target, source), cj_status_t::CJ_STATUS_SUCCESS);
        samples.push(started.elapsed().as_nanos());
        assert_eq!(cj_model_free(target), cj_status_t::CJ_STATUS_SUCCESS);
        assert_eq!(cj_model_free(source), cj_status_t::CJ_STATUS_SUCCESS);
    }

    summarize_samples(samples, 1)
}

fn summarize_samples(samples: Vec<u128>, iterations: usize) -> Timing {
    let elapsed_ns = median(samples);
    Timing {
        elapsed_ns,
        elapsed_per_iteration_ns: elapsed_ns as f64 / iterations as f64,
    }
}

fn median(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    if samples.is_empty() {
        return 0;
    }

    if samples.len() % 2 == 1 {
        return samples[samples.len() / 2];
    }

    let upper = samples[samples.len() / 2];
    let lower = samples[(samples.len() / 2) - 1];
    (upper + lower) / 2
}

fn result_json(
    fixture: &'static str,
    mode: Mode,
    operation: &'static str,
    layer: &'static str,
    iterations: usize,
    repeats: usize,
    timing: &Timing,
) -> serde_json::Value {
    serde_json::json!({
        "fixture": fixture,
        "mode": mode.label,
        "operation": operation,
        "layer": layer,
        "iterations": iterations,
        "repeats": repeats,
        "elapsed_ns": timing.elapsed_ns,
        "elapsed_per_iteration_ns": timing.elapsed_per_iteration_ns,
    })
}

fn parse_document_abi(bytes: &[u8]) -> *mut cj_model_t {
    let mut handle = ptr::null_mut();
    assert_eq!(
        cj_model_parse_document_bytes(bytes.as_ptr(), bytes.len(), &mut handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    handle
}

fn cityobject_ids_direct(model: &CityModel) -> Vec<String> {
    model
        .cityobjects()
        .iter()
        .map(|(_, cityobject)| cityobject.id().to_owned())
        .collect()
}

fn cityobject_ids_abi(model: *const cj_model_t) -> Vec<String> {
    let mut payload = cj_bytes_list_t::null();
    assert_eq!(
        cityjson_lib_ffi_core::exports::cj_model_copy_cityobject_ids(model, &mut payload),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    take_bytes_list(payload)
}

fn geometry_types_direct(model: &CityModel) -> Vec<cj_geometry_type_t> {
    model
        .iter_geometries()
        .map(|(_, geometry)| (*geometry.type_geometry()).into())
        .collect()
}

fn geometry_types_abi(model: *const cj_model_t) -> Vec<cj_geometry_type_t> {
    let mut payload = cj_geometry_types_t::null();
    assert_eq!(
        cityjson_lib_ffi_core::exports::cj_model_copy_geometry_types(model, &mut payload),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    take_geometry_types(payload)
}

fn take_bytes(bytes: cj_bytes_t) -> usize {
    let len = bytes.len;
    if len > 0 {
        assert!(!bytes.data.is_null());
        // SAFETY: the ABI allocated `len` readable bytes.
        unsafe {
            let _ = std::slice::from_raw_parts(bytes.data.cast_const(), len);
        }
    }
    assert_eq!(cj_bytes_free(bytes), cj_status_t::CJ_STATUS_SUCCESS);
    len
}

fn take_bytes_list(bytes: cj_bytes_list_t) -> Vec<String> {
    if bytes.len == 0 {
        assert_eq!(cj_bytes_list_free(bytes), cj_status_t::CJ_STATUS_SUCCESS);
        return Vec::new();
    }

    assert!(!bytes.data.is_null());
    let mut values = Vec::with_capacity(bytes.len);
    for index in 0..bytes.len {
        let item = unsafe { *bytes.data.add(index) };
        if item.len == 0 {
            values.push(String::new());
            continue;
        }

        // SAFETY: the ABI allocated `item.len` readable bytes.
        let bytes = unsafe { std::slice::from_raw_parts(item.data.cast_const(), item.len) };
        values.push(std::str::from_utf8(bytes).unwrap().to_owned());
    }
    assert_eq!(cj_bytes_list_free(bytes), cj_status_t::CJ_STATUS_SUCCESS);
    values
}

fn take_geometry_types(types: cj_geometry_types_t) -> Vec<cj_geometry_type_t> {
    if types.len == 0 {
        assert_eq!(cj_geometry_types_free(types), cj_status_t::CJ_STATUS_SUCCESS);
        return Vec::new();
    }

    assert!(!types.data.is_null());
    // SAFETY: the ABI allocated `types.len` readable geometry types.
    let values = unsafe { std::slice::from_raw_parts(types.data.cast_const(), types.len) }
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(cj_geometry_types_free(types), cj_status_t::CJ_STATUS_SUCCESS);
    values
}

fn clear_cityobject_geometries_direct(model: &mut cityjson_lib::CityModel) {
    for (_, cityobject) in model.cityobjects_mut().iter_mut() {
        cityobject.clear_geometry();
    }
}

fn clear_cityobject_geometries_abi(model: *mut cj_model_t) {
    let ids = cityobject_ids_abi(model as *const cj_model_t);
    for id in ids {
        let view = cj_string_view_t {
            data: id.as_bytes().as_ptr(),
            len: id.len(),
        };
        assert_eq!(
            cj_model_clear_cityobject_geometry(model, view),
            cj_status_t::CJ_STATUS_SUCCESS
        );
    }
}

fn unit_transform() -> cj_transform_t {
    cj_transform_t {
        scale_x: 1.0,
        scale_y: 1.0,
        scale_z: 1.0,
        translate_x: 0.0,
        translate_y: 0.0,
        translate_z: 0.0,
    }
}

fn apply_unit_transform(model: &mut cityjson_lib::CityModel) {
    let transform = model.transform_mut();
    transform.set_scale([1.0, 1.0, 1.0]);
    transform.set_translate([0.0, 0.0, 0.0]);
}
