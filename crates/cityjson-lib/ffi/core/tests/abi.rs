use std::ptr;

use cjlib_ffi_core::exports::*;
use cjlib_ffi_core::{
    cj_bytes_t, cj_error_kind_t, cj_probe_t, cj_root_kind_t, cj_status_t, cj_version_t, run_ffi,
};

fn v2_document() -> &'static [u8] {
    include_bytes!("../../../tests/data/v2_0/minimal.city.json")
}

fn v1_document() -> &'static [u8] {
    include_bytes!("../../../tests/data/v1_1/cityjson_minimal_complete.city.json")
}

fn feature_payload() -> &'static [u8] {
    br#"{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#
}

#[test]
fn free_functions_accept_null_handles_and_buffers() {
    assert_eq!(
        cj_model_free(ptr::null_mut()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_bytes_free(cj_bytes_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
}

#[test]
fn probe_reports_root_kind_and_version() {
    let mut probe = cj_probe_t::default();

    let status = cj_probe_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(probe.root_kind, cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON);
    assert_eq!(probe.version, cj_version_t::CJ_VERSION_V2_0);
    assert!(probe.has_version);
}

#[test]
fn parse_and_serialize_document_round_trip() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!handle.is_null());

    let mut serialized = cj_bytes_t::default();
    let status = cj_model_serialize_document(handle, &raw mut serialized);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!serialized.data.is_null());
    assert!(serialized.len > 0);

    let mut probe = cj_probe_t::default();
    let status = cj_probe_bytes(serialized.data, serialized.len, &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(probe.root_kind, cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON);
    assert_eq!(probe.version, cj_version_t::CJ_VERSION_V2_0);

    let mut round_trip = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(serialized.data, serialized.len, &raw mut round_trip);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!round_trip.is_null());

    assert_eq!(cj_model_free(round_trip), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_bytes_free(serialized), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn parse_feature_with_base_and_serialize_feature_round_trip() {
    let mut handle = ptr::null_mut();
    let status = cj_model_parse_feature_with_base_bytes(
        feature_payload().as_ptr(),
        feature_payload().len(),
        v2_document().as_ptr(),
        v2_document().len(),
        &raw mut handle,
    );
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!handle.is_null());

    let mut serialized = cj_bytes_t::default();
    let status = cj_model_serialize_feature(handle, &raw mut serialized);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!serialized.data.is_null());
    assert!(serialized.len > 0);

    let mut probe = cj_probe_t::default();
    let status = cj_probe_bytes(serialized.data, serialized.len, &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(
        probe.root_kind,
        cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON_FEATURE
    );

    let mut round_trip = ptr::null_mut();
    let status = cj_model_parse_feature_with_base_bytes(
        serialized.data,
        serialized.len,
        v2_document().as_ptr(),
        v2_document().len(),
        &raw mut round_trip,
    );
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!round_trip.is_null());

    assert_eq!(cj_model_free(round_trip), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_bytes_free(serialized), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn unsupported_version_is_reported_without_panicking() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v1_document().as_ptr(), v1_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_VERSION);
    assert!(handle.is_null());
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_VERSION);

    let mut len = 0usize;
    let status = cj_last_error_message_copy(ptr::null_mut(), 0, &raw mut len);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(len > 0);

    let mut buffer = vec![0u8; len + 1];
    let mut copied = 0usize;
    let status = cj_last_error_message_copy(buffer.as_mut_ptr(), buffer.len(), &raw mut copied);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(copied, len);

    let message = std::str::from_utf8(&buffer[..copied]).expect("last error should be utf-8");
    assert!(message.contains("unsupported"));
}

#[test]
fn last_error_copy_failures_do_not_clobber_the_stored_error() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v1_document().as_ptr(), v1_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_VERSION);
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_VERSION);

    let mut copied = 0usize;
    let mut buffer = [0u8; 4];
    let status = cj_last_error_message_copy(buffer.as_mut_ptr(), buffer.len(), &raw mut copied);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_VERSION);

    assert_eq!(cj_clear_error(), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_NONE);
    assert_eq!(cj_last_error_message_len(), 0);
}

#[test]
fn null_arguments_are_rejected_and_reported() {
    let mut probe = cj_probe_t::default();
    let status = cj_probe_bytes(ptr::null(), 1, &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);
    assert_eq!(
        cj_last_error_kind(),
        cj_error_kind_t::CJ_ERROR_KIND_INVALID_ARGUMENT
    );

    let mut document_handle = ptr::null_mut();
    let status = cj_model_parse_document_bytes(ptr::null(), 1, &raw mut document_handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut feature_handle = ptr::null_mut();
    let status = cj_model_parse_feature_bytes(ptr::null(), 1, &raw mut feature_handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut feature_with_base_handle = ptr::null_mut();
    let status = cj_model_parse_feature_with_base_bytes(
        ptr::null(),
        1,
        ptr::null(),
        1,
        &raw mut feature_with_base_handle,
    );
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut bytes = cj_bytes_t::default();
    let status = cj_model_serialize_document(ptr::null(), &raw mut bytes);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);
}

#[test]
fn panic_helper_translates_panics_to_internal_status() {
    let status = run_ffi::<(), _, _>(|| -> Result<(), cjlib_ffi_core::AbiError> {
        panic!("ffi panic test");
    })
    .unwrap_err();

    assert_eq!(status, cj_status_t::CJ_STATUS_INTERNAL);
    assert_eq!(
        cj_last_error_kind(),
        cj_error_kind_t::CJ_ERROR_KIND_INTERNAL
    );
}
