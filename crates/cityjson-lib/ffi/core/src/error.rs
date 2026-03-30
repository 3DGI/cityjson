use std::cell::RefCell;
use std::ffi::c_char;
use std::panic::{AssertUnwindSafe, UnwindSafe, catch_unwind};
use std::ptr;

use crate::abi::{cj_error_kind_t, cj_status_t};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiError {
    pub status: cj_status_t,
    pub kind: cj_error_kind_t,
    pub message: String,
}

impl AbiError {
    pub fn new(status: cj_status_t, kind: cj_error_kind_t, message: impl Into<String>) -> Self {
        Self {
            status,
            kind,
            message: message.into(),
        }
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(
            cj_status_t::InvalidArgument,
            cj_error_kind_t::InvalidArgument,
            message,
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(cj_status_t::Internal, cj_error_kind_t::Internal, message)
    }
}

impl From<&cjlib::Error> for AbiError {
    fn from(error: &cjlib::Error) -> Self {
        match error {
            cjlib::Error::Io(inner) => {
                Self::new(cj_status_t::Io, cj_error_kind_t::Io, inner.to_string())
            }
            cjlib::Error::Json(inner) => Self::new(
                cj_status_t::Syntax,
                cj_error_kind_t::Syntax,
                inner.to_string(),
            ),
            cjlib::Error::CityJSON(inner) => Self::new(
                cj_status_t::Model,
                cj_error_kind_t::Model,
                inner.to_string(),
            ),
            cjlib::Error::MissingVersion => Self::new(
                cj_status_t::Version,
                cj_error_kind_t::Version,
                error.to_string(),
            ),
            cjlib::Error::ExpectedCityJSON(_) | cjlib::Error::ExpectedCityJSONFeature(_) => {
                Self::new(
                    cj_status_t::Shape,
                    cj_error_kind_t::Shape,
                    error.to_string(),
                )
            }
            cjlib::Error::UnsupportedType(_) => Self::new(
                cj_status_t::Unsupported,
                cj_error_kind_t::Unsupported,
                error.to_string(),
            ),
            cjlib::Error::UnsupportedVersion { .. } => Self::new(
                cj_status_t::Version,
                cj_error_kind_t::Version,
                error.to_string(),
            ),
            cjlib::Error::Streaming(_) => Self::new(
                cj_status_t::Shape,
                cj_error_kind_t::Shape,
                error.to_string(),
            ),
            cjlib::Error::Import(_) => Self::new(
                cj_status_t::Model,
                cj_error_kind_t::Model,
                error.to_string(),
            ),
            cjlib::Error::UnsupportedFeature(_) => Self::new(
                cj_status_t::Unsupported,
                cj_error_kind_t::Unsupported,
                error.to_string(),
            ),
        }
    }
}

impl From<cjlib::Error> for AbiError {
    fn from(error: cjlib::Error) -> Self {
        Self::from(&error)
    }
}

impl From<cjlib::ErrorKind> for cj_error_kind_t {
    fn from(value: cjlib::ErrorKind) -> Self {
        match value {
            cjlib::ErrorKind::Io => Self::Io,
            cjlib::ErrorKind::Syntax => Self::Syntax,
            cjlib::ErrorKind::Version => Self::Version,
            cjlib::ErrorKind::Shape => Self::Shape,
            cjlib::ErrorKind::Unsupported => Self::Unsupported,
            cjlib::ErrorKind::Model => Self::Model,
        }
    }
}

impl From<cjlib::ErrorKind> for cj_status_t {
    fn from(value: cjlib::ErrorKind) -> Self {
        match value {
            cjlib::ErrorKind::Io => Self::Io,
            cjlib::ErrorKind::Syntax => Self::Syntax,
            cjlib::ErrorKind::Version => Self::Version,
            cjlib::ErrorKind::Shape => Self::Shape,
            cjlib::ErrorKind::Unsupported => Self::Unsupported,
            cjlib::ErrorKind::Model => Self::Model,
        }
    }
}

#[derive(Debug, Clone)]
struct LastError {
    status: cj_status_t,
    kind: cj_error_kind_t,
    message: String,
}

impl LastError {
    fn empty() -> Self {
        Self {
            status: cj_status_t::Success,
            kind: cj_error_kind_t::None,
            message: String::new(),
        }
    }
}

thread_local! {
    static LAST_ERROR: RefCell<LastError> = RefCell::new(LastError::empty());
}

pub fn clear_last_error() {
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = LastError::empty();
    });
}

pub fn set_last_error(error: AbiError) {
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = LastError {
            status: error.status,
            kind: error.kind,
            message: error.message,
        };
    });
}

pub fn set_last_error_from_cjlib_error(error: cjlib::Error) -> cj_status_t {
    let abi_error = AbiError::from(error);
    let status = abi_error.status;
    set_last_error(abi_error);
    status
}

pub fn last_error_kind() -> cj_error_kind_t {
    LAST_ERROR.with(|cell| cell.borrow().kind)
}

pub fn last_error_status() -> cj_status_t {
    LAST_ERROR.with(|cell| cell.borrow().status)
}

pub fn last_error_message_len() -> usize {
    LAST_ERROR.with(|cell| cell.borrow().message.len())
}

pub unsafe fn copy_last_error_message(
    buffer: *mut c_char,
    capacity: usize,
    out_len: *mut usize,
) -> cj_status_t {
    if out_len.is_null() {
        return cj_status_t::InvalidArgument;
    }

    let (message_len, message) = LAST_ERROR.with(|cell| {
        let borrowed = cell.borrow();
        (borrowed.message.len(), borrowed.message.clone())
    });

    unsafe {
        ptr::write(out_len, message_len);
    }

    if capacity == 0 {
        if buffer.is_null() {
            return cj_status_t::Success;
        }

        return cj_status_t::InvalidArgument;
    }

    if buffer.is_null() {
        return cj_status_t::InvalidArgument;
    }

    let available = capacity.saturating_sub(1);
    let copy_len = message_len.min(available);
    if copy_len > 0 {
        unsafe {
            ptr::copy_nonoverlapping(message.as_ptr().cast::<c_char>(), buffer, copy_len);
        }
    }
    unsafe {
        *buffer.add(copy_len) = 0;
    }

    if message_len >= capacity {
        return cj_status_t::InvalidArgument;
    }

    cj_status_t::Success
}

pub fn run_ffi<T, F>(f: F) -> Result<T, cj_status_t>
where
    F: FnOnce() -> cjlib::Result<T> + UnwindSafe,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(value)) => {
            clear_last_error();
            Ok(value)
        }
        Ok(Err(error)) => {
            let abi_error = AbiError::from(error);
            let status = abi_error.status;
            set_last_error(abi_error);
            Err(status)
        }
        Err(_) => {
            let abi_error = AbiError::internal("panic across the C ABI boundary");
            let status = abi_error.status;
            set_last_error(abi_error);
            Err(status)
        }
    }
}
