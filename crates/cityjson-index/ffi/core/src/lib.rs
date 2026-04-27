use std::ffi::c_char;
use std::fs;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::ptr::{self, NonNull};
use std::slice;

use cityjson_lib::json;
use cityjson_lib_ffi_core::{
    AbiError, bytes_free, bytes_from_string, bytes_from_vec, cj_bytes_t, cj_error_kind_t,
    cj_status_t, clear_last_error, copy_last_error_message, last_error_kind,
    last_error_message_len, run_ffi,
};

use cityjson_index::{
    CityIndex, FeatureBounds, IndexedFeatureRef, ResolvedDataset, resolve_dataset,
};

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct cjx_index_t {
    _private: [u8; 0],
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cjx_index_status_t {
    pub exists: bool,
    pub needs_reindex: bool,
    pub indexed_feature_count: usize,
    pub indexed_source_count: usize,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cjx_feature_ref_t {
    pub row_id: i64,
    pub feature_id: cj_bytes_t,
    pub source_path: cj_bytes_t,
    pub offset: u64,
    pub length: u64,
    pub vertices_offset: u64,
    pub vertices_length: u64,
    pub member_ranges_json: cj_bytes_t,
    pub source_id: i64,
}

impl From<IndexedFeatureRef> for cjx_feature_ref_t {
    fn from(feature: IndexedFeatureRef) -> Self {
        Self {
            row_id: feature.row_id,
            feature_id: bytes_from_string(feature.feature_id),
            source_path: bytes_from_string(feature.source_path.to_string_lossy().into_owned()),
            offset: feature.offset,
            length: feature.length,
            vertices_offset: feature.vertices_offset.unwrap_or_default(),
            vertices_length: feature.vertices_length.unwrap_or_default(),
            member_ranges_json: bytes_from_string(feature.member_ranges_json.unwrap_or_default()),
            source_id: feature.source_id,
        }
    }
}

impl TryFrom<&cjx_feature_ref_t> for IndexedFeatureRef {
    type Error = AbiError;

    fn try_from(feature: &cjx_feature_ref_t) -> Result<Self, Self::Error> {
        let source_path = PathBuf::from(bytes_to_string(feature.source_path, "source_path")?);
        let feature_id = bytes_to_string(feature.feature_id, "feature_id")?;
        let member_ranges_json = (!feature.member_ranges_json.data.is_null()
            && feature.member_ranges_json.len > 0)
            .then(|| bytes_to_string(feature.member_ranges_json, "member_ranges_json"))
            .transpose()?;
        let has_vertices_range = feature.vertices_offset != 0 || feature.vertices_length != 0;

        Ok(Self {
            row_id: feature.row_id,
            feature_id,
            source_id: feature.source_id,
            source_path,
            offset: feature.offset,
            length: feature.length,
            vertices_offset: has_vertices_range.then_some(feature.vertices_offset),
            vertices_length: has_vertices_range.then_some(feature.vertices_length),
            member_ranges_json,
            bounds: FeatureBounds {
                min_x: 0.0,
                max_x: 0.0,
                min_y: 0.0,
                max_y: 0.0,
                min_z: 0.0,
                max_z: 0.0,
            },
        })
    }
}

struct OpenedIndex {
    resolved: ResolvedDataset,
    index: CityIndex,
}

impl OpenedIndex {
    fn open(dataset_dir: &Path, index_path: Option<PathBuf>) -> Result<Self, AbiError> {
        let resolved = resolve_dataset(dataset_dir, index_path).map_err(AbiError::from)?;
        let index = CityIndex::open(resolved.storage_layout(), resolved.index_path.as_path())
            .map_err(AbiError::from)?;
        Ok(Self { resolved, index })
    }

    fn status(&self) -> Result<cjx_index_status_t, AbiError> {
        let inspection = self.resolved.inspect().map_err(AbiError::from)?;
        Ok(cjx_index_status_t {
            exists: inspection.index.exists,
            needs_reindex: !inspection.index.fresh.unwrap_or(false),
            indexed_feature_count: inspection.index.indexed_feature_count.unwrap_or(0),
            indexed_source_count: inspection.index.indexed_source_count.unwrap_or(0),
        })
    }

    fn reindex(&mut self) -> Result<(), AbiError> {
        self.index.reindex().map_err(AbiError::from)
    }

    fn feature_ref_count(&self) -> Result<usize, AbiError> {
        self.index.feature_ref_count().map_err(AbiError::from)
    }

    fn feature_ref_page(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<cjx_feature_ref_t>, AbiError> {
        self.index
            .feature_ref_page(offset, limit)
            .map(|refs| refs.into_iter().map(Into::into).collect())
            .map_err(AbiError::from)
    }

    fn lookup_feature_refs(&self, feature_id: &str) -> Result<Vec<cjx_feature_ref_t>, AbiError> {
        self.index
            .lookup_feature_refs(feature_id)
            .map(|refs| refs.into_iter().map(Into::into).collect())
            .map_err(AbiError::from)
    }

    fn get_bytes(&self, feature_id: &str) -> Result<Option<Vec<u8>>, AbiError> {
        self.index.get_bytes(feature_id).map_err(AbiError::from)
    }

    fn get_model_bytes(&self, feature_id: &str) -> Result<Option<Vec<u8>>, AbiError> {
        let Some(model) = self.index.get(feature_id).map_err(AbiError::from)? else {
            return Ok(None);
        };
        json::to_vec(&model).map(Some).map_err(AbiError::from)
    }

    fn read_feature_bytes(feature: &cjx_feature_ref_t) -> Result<Vec<u8>, AbiError> {
        let source_path = bytes_to_string(feature.source_path, "source_path")?;
        read_exact_range(Path::new(&source_path), feature.offset, feature.length)
    }

    fn read_feature_model_bytes(&self, feature: &cjx_feature_ref_t) -> Result<Vec<u8>, AbiError> {
        let feature = IndexedFeatureRef::try_from(feature)?;
        let model = self.index.read_feature(&feature).map_err(AbiError::from)?;
        json::to_vec(&model).map_err(AbiError::from)
    }
}

fn read_exact_range(path: &Path, offset: u64, length: u64) -> Result<Vec<u8>, AbiError> {
    let mut file = fs::File::open(path).map_err(|error| {
        AbiError::internal(format!("failed to open {}: {error}", path.display()))
    })?;
    read_exact_range_from_file(&mut file, path, offset, length)
}

fn read_exact_range_from_file(
    file: &mut fs::File,
    path: &Path,
    offset: u64,
    length: u64,
) -> Result<Vec<u8>, AbiError> {
    let length = usize::try_from(length).map_err(|_| {
        AbiError::internal(format!(
            "requested read of {length} bytes from {} exceeds the supported buffer size",
            path.display()
        ))
    })?;
    if length > isize::MAX as usize {
        return Err(AbiError::internal(format!(
            "requested read of {length} bytes from {} exceeds the supported buffer size",
            path.display()
        )));
    }

    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|error| {
        AbiError::internal(format!(
            "failed to allocate buffer for {} bytes from {}: {error}",
            length,
            path.display()
        ))
    })?;
    bytes.resize(length, 0);

    file.seek(SeekFrom::Start(offset)).map_err(|error| {
        AbiError::internal(format!(
            "failed to seek to byte offset {offset} in {}: {error}",
            path.display()
        ))
    })?;
    file.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            AbiError::internal(format!(
                "short read while reading {length} bytes at offset {offset} from {}",
                path.display()
            ))
        } else {
            AbiError::internal(format!(
                "failed to read {length} bytes at offset {offset} from {}: {error}",
                path.display()
            ))
        }
    })?;

    Ok(bytes)
}

fn bytes_to_string(bytes: cj_bytes_t, name: &'static str) -> Result<String, AbiError> {
    if bytes.data.is_null() {
        if bytes.len == 0 {
            return Ok(String::new());
        }
        return Err(AbiError::invalid_argument(format!(
            "{name} must not be null when len is non-zero"
        )));
    }

    // SAFETY: `bytes.data` is non-null and the caller promises `bytes.len` readable bytes.
    let slice = unsafe { slice::from_raw_parts(bytes.data, bytes.len) };
    let value = std::str::from_utf8(slice).map_err(|error| {
        AbiError::invalid_argument(format!("{name} must be valid UTF-8: {error}"))
    })?;
    Ok(value.to_owned())
}

fn required_string(
    data: *const c_char,
    len: usize,
    name: &'static str,
) -> Result<String, AbiError> {
    if len == 0 {
        return Err(AbiError::invalid_argument(format!(
            "{name} must not be empty"
        )));
    }
    let ptr = NonNull::new(data.cast_mut())
        .ok_or_else(|| AbiError::invalid_argument(format!("{name} must not be null")))?;
    // SAFETY: the caller promises `len` readable bytes when the pointer is non-null.
    let bytes = unsafe { slice::from_raw_parts(ptr.as_ptr().cast_const().cast::<u8>(), len) };
    let value = std::str::from_utf8(bytes).map_err(|error| {
        AbiError::invalid_argument(format!("{name} must be valid UTF-8: {error}"))
    })?;
    Ok(value.to_owned())
}

fn optional_path(
    data: *const c_char,
    len: usize,
    name: &'static str,
) -> Result<Option<PathBuf>, AbiError> {
    if len == 0 {
        return Ok(None);
    }
    required_string(data, len, name)
        .map(PathBuf::from)
        .map(Some)
}

fn write_value<T>(out: *mut T, name: &'static str, value: T) -> Result<(), AbiError> {
    let out = NonNull::new(out)
        .ok_or_else(|| AbiError::invalid_argument(format!("{name} must not be null")))?;
    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), value);
    }
    Ok(())
}

fn write_handle(out_index: *mut *mut cjx_index_t, index: OpenedIndex) -> Result<(), AbiError> {
    let out = NonNull::new(out_index)
        .ok_or_else(|| AbiError::invalid_argument("out_index must not be null"))?;
    let raw = Box::into_raw(Box::new(index)).cast::<cjx_index_t>();
    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), raw);
    }
    Ok(())
}

fn required_handle<'a>(handle: *const cjx_index_t) -> Result<&'a OpenedIndex, AbiError> {
    let ptr = NonNull::new(handle.cast_mut())
        .ok_or_else(|| AbiError::invalid_argument("index must not be null"))?;
    // SAFETY: the pointer originates from `write_handle`, which stores `OpenedIndex` as the
    // concrete allocation behind `cjx_index_t`.
    Ok(unsafe { &*ptr.as_ptr().cast::<OpenedIndex>() })
}

fn required_handle_mut<'a>(handle: *mut cjx_index_t) -> Result<&'a mut OpenedIndex, AbiError> {
    let ptr =
        NonNull::new(handle).ok_or_else(|| AbiError::invalid_argument("index must not be null"))?;
    // SAFETY: the pointer originates from `write_handle`, which stores `OpenedIndex` as the
    // concrete allocation behind `cjx_index_t`.
    Ok(unsafe { &mut *ptr.as_ptr().cast::<OpenedIndex>() })
}

fn free_feature_ref(feature: cjx_feature_ref_t) {
    // SAFETY: each field is an owned byte buffer allocated by this ABI.
    unsafe {
        bytes_free(feature.feature_id);
        bytes_free(feature.source_path);
        bytes_free(feature.member_ranges_json);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_clear_error() -> cj_status_t {
    clear_last_error();
    cj_status_t::CJ_STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_last_error_kind() -> cj_error_kind_t {
    last_error_kind()
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_last_error_message_len() -> usize {
    last_error_message_len()
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `buffer` must point to `capacity` writable bytes and `out_len` must be a
/// valid writable pointer when non-null.
pub unsafe extern "C" fn cjx_last_error_message_copy(
    buffer: *mut c_char,
    capacity: usize,
    out_len: *mut usize,
) -> cj_status_t {
    // SAFETY: the caller upholds the buffer contract.
    unsafe { copy_last_error_message(buffer, capacity, out_len) }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_bytes_free(bytes: cj_bytes_t) -> cj_status_t {
    // SAFETY: `bytes` originated from this ABI.
    unsafe {
        bytes_free(bytes);
    }
    cj_status_t::CJ_STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_open(
    dataset_dir: *const c_char,
    dataset_dir_len: usize,
    index_path: *const c_char,
    index_path_len: usize,
    out_index: *mut *mut cjx_index_t,
) -> cj_status_t {
    match run_ffi(|| {
        let dataset_dir = required_string(dataset_dir, dataset_dir_len, "dataset_dir")?;
        let index_path = optional_path(index_path, index_path_len, "index_path")?;
        let opened = OpenedIndex::open(Path::new(&dataset_dir), index_path)?;
        write_handle(out_index, opened)
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_free(handle: *mut cjx_index_t) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle_mut(handle)?;
        let raw = std::ptr::from_mut(handle);
        // SAFETY: `raw` originates from `Box::into_raw` in `write_handle`.
        unsafe {
            drop(Box::from_raw(raw));
        }
        Ok::<(), AbiError>(())
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_status(
    handle: *const cjx_index_t,
    out_status: *mut cjx_index_status_t,
) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle(handle)?;
        let status = handle.status()?;
        write_value(out_status, "out_status", status)
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_reindex(handle: *mut cjx_index_t) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle_mut(handle)?;
        handle.reindex()
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_feature_ref_count(
    handle: *const cjx_index_t,
    out_count: *mut usize,
) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle(handle)?;
        let count = handle.feature_ref_count()?;
        write_value(out_count, "out_count", count)
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_feature_ref_page(
    handle: *const cjx_index_t,
    offset: usize,
    limit: usize,
    out_refs: *mut *mut cjx_feature_ref_t,
    out_count: *mut usize,
) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle(handle)?;
        let refs = handle.feature_ref_page(offset, limit)?;
        let count = refs.len();

        write_value(out_count, "out_count", count)?;

        if count == 0 {
            let out_refs = NonNull::new(out_refs)
                .ok_or_else(|| AbiError::invalid_argument("out_refs must not be null"))?;
            // SAFETY: `out_refs` is validated to be non-null and points to writable storage.
            unsafe {
                ptr::write(out_refs.as_ptr(), ptr::null_mut());
            }
            return Ok(());
        }

        let boxed = refs.into_boxed_slice();
        let ptr = Box::into_raw(boxed).cast::<cjx_feature_ref_t>();
        write_value(out_refs, "out_refs", ptr)
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_lookup_feature_refs(
    handle: *const cjx_index_t,
    feature_id: *const c_char,
    feature_id_len: usize,
    out_refs: *mut *mut cjx_feature_ref_t,
    out_count: *mut usize,
) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle(handle)?;
        let feature_id = required_string(feature_id, feature_id_len, "feature_id")?;
        let refs = handle.lookup_feature_refs(&feature_id)?;
        let count = refs.len();

        write_value(out_count, "out_count", count)?;

        if count == 0 {
            let out_refs = NonNull::new(out_refs)
                .ok_or_else(|| AbiError::invalid_argument("out_refs must not be null"))?;
            // SAFETY: `out_refs` is validated to be non-null and points to writable storage.
            unsafe {
                ptr::write(out_refs.as_ptr(), ptr::null_mut());
            }
            return Ok(());
        }

        let boxed = refs.into_boxed_slice();
        let ptr = Box::into_raw(boxed).cast::<cjx_feature_ref_t>();
        write_value(out_refs, "out_refs", ptr)
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `refs` must either be null or point to `count` feature refs allocated by
/// `cjx_index_feature_ref_page`.
pub unsafe extern "C" fn cjx_feature_ref_page_free(
    refs: *mut cjx_feature_ref_t,
    count: usize,
) -> cj_status_t {
    match run_ffi(|| {
        if refs.is_null() || count == 0 {
            return Ok::<(), AbiError>(());
        }

        // SAFETY: the caller promises `count` valid feature refs starting at `refs`.
        let slice = unsafe { slice::from_raw_parts_mut(refs, count) };
        for feature_ref in slice.iter_mut() {
            free_feature_ref(*feature_ref);
        }

        // SAFETY: `refs` was allocated as a boxed slice by this ABI.
        let raw = ptr::slice_from_raw_parts_mut(refs, count);
        unsafe {
            drop(Box::from_raw(raw));
        }
        Ok::<(), AbiError>(())
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_get_bytes(
    handle: *const cjx_index_t,
    feature_id: *const c_char,
    feature_id_len: usize,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle(handle)?;
        let feature_id = required_string(feature_id, feature_id_len, "feature_id")?;
        let Some(bytes) = handle.get_bytes(&feature_id)? else {
            return Err(AbiError::invalid_argument(format!(
                "feature {feature_id} was not found"
            )));
        };
        write_value(out_bytes, "out_bytes", bytes_from_vec(bytes))
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_get_model_bytes(
    handle: *const cjx_index_t,
    feature_id: *const c_char,
    feature_id_len: usize,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle(handle)?;
        let feature_id = required_string(feature_id, feature_id_len, "feature_id")?;
        let Some(bytes) = handle.get_model_bytes(&feature_id)? else {
            return Err(AbiError::invalid_argument(format!(
                "feature {feature_id} was not found"
            )));
        };
        write_value(out_bytes, "out_bytes", bytes_from_vec(bytes))
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_read_feature_bytes(
    handle: *const cjx_index_t,
    feature: *const cjx_feature_ref_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    match run_ffi(|| {
        required_handle(handle)?;
        let feature = NonNull::new(feature.cast_mut())
            .ok_or_else(|| AbiError::invalid_argument("feature must not be null"))?;
        // SAFETY: `feature` is validated to be non-null and points to a valid `cjx_feature_ref_t`.
        let feature = unsafe { feature.as_ref() };
        let bytes = OpenedIndex::read_feature_bytes(feature)?;
        write_value(out_bytes, "out_bytes", bytes_from_vec(bytes))
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cjx_index_read_feature_model_bytes(
    handle: *const cjx_index_t,
    feature: *const cjx_feature_ref_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    match run_ffi(|| {
        let handle = required_handle(handle)?;
        let feature = NonNull::new(feature.cast_mut())
            .ok_or_else(|| AbiError::invalid_argument("feature must not be null"))?;
        // SAFETY: `feature` is validated to be non-null and points to a valid `cjx_feature_ref_t`.
        let feature = unsafe { feature.as_ref() };
        let bytes = handle.read_feature_model_bytes(feature)?;
        write_value(out_bytes, "out_bytes", bytes_from_vec(bytes))
    }) {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}
