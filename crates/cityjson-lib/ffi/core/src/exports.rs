#![allow(non_camel_case_types)]

use std::ffi::c_char;
use std::ptr::{self, NonNull};
use std::slice;

use crate::abi::{cj_bytes_t, cj_error_kind_t, cj_model_t, cj_probe_t, cj_status_t};
use crate::error::{
    AbiError, clear_last_error, copy_last_error_message, last_error_kind, last_error_message_len,
    run_ffi,
};
use crate::handle::{
    bytes_free as free_bytes, bytes_from_vec, model_as_ref, model_free, model_into_handle,
};
use cjlib::{CityJSONVersion, CityModel, Error, json::RootKind};

fn invalid_argument(message: impl Into<String>) -> AbiError {
    AbiError::invalid_argument(message)
}

fn ffi_status(result: Result<(), cj_status_t>) -> cj_status_t {
    match result {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

fn required_bytes<'a>(
    data: *const u8,
    len: usize,
    name: &'static str,
) -> Result<&'a [u8], AbiError> {
    if len == 0 {
        return Ok(&[]);
    }

    let ptr = NonNull::new(data.cast_mut())
        .ok_or_else(|| invalid_argument(format!("{name} must not be null when len is non-zero")))?;

    // SAFETY: the caller promises `len` readable bytes when the pointer is non-null.
    Ok(unsafe { slice::from_raw_parts(ptr.as_ptr().cast_const(), len) })
}

fn write_model_handle(out_model: *mut *mut cj_model_t, model: CityModel) -> Result<(), AbiError> {
    let out_model =
        NonNull::new(out_model).ok_or_else(|| invalid_argument("out_model must not be null"))?;

    // SAFETY: `out_model` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out_model.as_ptr(), model_into_handle(model));
    }

    Ok(())
}

fn write_bytes(out_bytes: *mut cj_bytes_t, bytes: Vec<u8>) -> Result<(), AbiError> {
    let out_bytes =
        NonNull::new(out_bytes).ok_or_else(|| invalid_argument("out_bytes must not be null"))?;

    // SAFETY: `out_bytes` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out_bytes.as_ptr(), bytes_from_vec(bytes));
    }

    Ok(())
}

fn reject_unsupported_document_version(version: Option<CityJSONVersion>) -> Result<(), AbiError> {
    match version {
        Some(CityJSONVersion::V2_0) => Ok(()),
        Some(found) => Err(AbiError::from(Error::UnsupportedVersion {
            found: found.to_string(),
            supported: CityJSONVersion::V2_0.to_string(),
        })),
        None => Err(AbiError::from(Error::MissingVersion)),
    }
}

fn reject_unsupported_feature_version(version: Option<CityJSONVersion>) -> Result<(), AbiError> {
    match version {
        Some(found) => Err(AbiError::from(Error::UnsupportedVersion {
            found: found.to_string(),
            supported: CityJSONVersion::V2_0.to_string(),
        })),
        None => Ok(()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_free(handle: *mut cj_model_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if handle.is_null() {
            return Ok(());
        }

        // SAFETY: the ABI only frees handles that it allocated.
        unsafe {
            model_free(handle);
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_bytes_free(bytes: cj_bytes_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if bytes.data.is_null() {
            if bytes.len == 0 {
                return Ok(());
            }

            return Err(invalid_argument(
                "bytes data must not be null when len is non-zero",
            ));
        }

        // SAFETY: the ABI only frees buffers allocated by `bytes_from_vec`.
        unsafe {
            free_bytes(bytes);
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_last_error_kind() -> cj_error_kind_t {
    last_error_kind()
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_last_error_message_len() -> usize {
    last_error_message_len()
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_last_error_message_copy(
    buffer: *mut u8,
    capacity: usize,
    out_len: *mut usize,
) -> cj_status_t {
    // SAFETY: this helper validates the out-pointer and buffer/capacity pairing.
    unsafe { copy_last_error_message(buffer.cast::<c_char>(), capacity, out_len) }
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_clear_error() -> cj_status_t {
    clear_last_error();
    cj_status_t::CJ_STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_probe_bytes(
    data: *const u8,
    len: usize,
    out_probe: *mut cj_probe_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let probe = cjlib::json::probe(input)?;
        let out_probe = NonNull::new(out_probe)
            .ok_or_else(|| invalid_argument("out_probe must not be null"))?;

        // SAFETY: `out_probe` is validated to be non-null and points to writable storage.
        unsafe {
            ptr::write(out_probe.as_ptr(), cj_probe_t::from_probe(&probe));
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_document_bytes(
    data: *const u8,
    len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let probe = cjlib::json::probe(input)?;
        if probe.kind() != RootKind::CityJSON {
            return Err(AbiError::from(Error::ExpectedCityJSON(
                probe.kind().to_string(),
            )));
        }

        reject_unsupported_document_version(probe.version())?;
        let model = cjlib::json::from_slice(input)?;
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_feature_bytes(
    data: *const u8,
    len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let probe = cjlib::json::probe(input)?;
        if probe.kind() != RootKind::CityJSONFeature {
            return Err(AbiError::from(Error::ExpectedCityJSONFeature(
                probe.kind().to_string(),
            )));
        }

        reject_unsupported_feature_version(probe.version())?;
        let model = cjlib::json::from_feature_slice(input)?;
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_feature_with_base_bytes(
    feature_data: *const u8,
    feature_len: usize,
    base_data: *const u8,
    base_len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let feature = required_bytes(feature_data, feature_len, "feature_data")?;
        let base = required_bytes(base_data, base_len, "base_data")?;

        let feature_probe = cjlib::json::probe(feature)?;
        if feature_probe.kind() != RootKind::CityJSONFeature {
            return Err(AbiError::from(Error::ExpectedCityJSONFeature(
                feature_probe.kind().to_string(),
            )));
        }

        reject_unsupported_feature_version(feature_probe.version())?;

        let base_probe = cjlib::json::probe(base)?;
        if base_probe.kind() != RootKind::CityJSON {
            return Err(AbiError::from(Error::ExpectedCityJSON(
                base_probe.kind().to_string(),
            )));
        }

        reject_unsupported_document_version(base_probe.version())?;
        let model = cjlib::json::from_feature_slice_with_base(feature, base)?;
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_document(
    model: *const cj_model_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        // SAFETY: null is rejected by `model_as_ref`; valid handles point to Rust-owned models.
        let model = unsafe { model_as_ref(model) }
            .ok_or_else(|| invalid_argument("model must not be null"))?;
        let bytes = cjlib::json::to_vec(model)?;
        write_bytes(out_bytes, bytes)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_feature(
    model: *const cj_model_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        // SAFETY: null is rejected by `model_as_ref`; valid handles point to Rust-owned models.
        let model = unsafe { model_as_ref(model) }
            .ok_or_else(|| invalid_argument("model must not be null"))?;
        let bytes = cjlib::json::to_feature_string(model)?.into_bytes();
        write_bytes(out_bytes, bytes)
    }))
}
