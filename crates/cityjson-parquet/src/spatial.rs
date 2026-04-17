//! Spatial indexing for `CityParquet` files using Hilbert curves.
//!
//! Provides [`SpatialIndex`] - a lightweight spatial index over `CityObject`
//! bounding boxes, sorted by Hilbert curve value. This enables efficient
//! view-frustum culling: given a 2D query rectangle, the index returns only
//! the objects whose bounding boxes intersect it.
//!
//! # Example
//!
//! ```ignore
//! let parts = cityjson_parquet::read_package_parts_file("model.cityjson-parquet")?;
//! let index = SpatialIndex::build(&parts);
//! let visible = index.query(&BBox2D::new(80_000.0, 440_000.0, 81_000.0, 441_000.0));
//! ```

use arrow_array::RecordBatch;
use arrow_array::{
    Array, FixedSizeListArray, Float64Array, LargeStringArray, ListArray, StringArray, UInt32Array,
    UInt64Array,
};
use cityjson_arrow::schema::CityModelArrowParts;
use std::collections::HashMap;

/// Hilbert curve order — grid resolution is 2^ORDER per axis.
const HILBERT_ORDER: u32 = 16;

/// 2D axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct BBox2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BBox2D {
    #[must_use]
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    #[must_use]
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }

    #[must_use]
    pub fn intersects(&self, other: &BBox2D) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    #[must_use]
    pub fn union(&self, other: &BBox2D) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

/// An entry in the spatial index representing one `CityObject`.
#[derive(Debug, Clone)]
pub struct SpatialEntry {
    /// Hilbert curve index for this object's centroid.
    pub hilbert_index: u64,
    /// Row index in the `cityobjects` Arrow table.
    pub row_index: usize,
    /// `CityObject` index (`cityobject_ix` column value).
    pub cityobject_ix: u64,
    /// 2D bounding box of the object.
    pub bbox: BBox2D,
    /// Full 3D extent `[xmin, ymin, zmin, xmax, ymax, zmax]`.
    pub extent_3d: [f64; 6],
    /// `CityObject` identifier.
    pub id: String,
    /// `CityObject` type (e.g. `"Building"`).
    pub object_type: String,
}

/// Spatial index over `CityObjects`, sorted by Hilbert curve value.
///
/// Objects are indexed by their 2D bounding box (XY plane). The Hilbert
/// curve ordering ensures spatially nearby objects are stored adjacently,
/// which improves cache locality during spatial queries.
pub struct SpatialIndex {
    /// Overall 2D extent of all indexed objects.
    pub extent: BBox2D,
    /// Full 3D extent of all indexed objects.
    pub extent_3d: [f64; 6],
    /// Total number of `CityObjects` in the source table (including any
    /// without bounding boxes that were skipped).
    pub total_objects: usize,
    /// Entries sorted by Hilbert curve index.
    entries: Vec<SpatialEntry>,
}

impl SpatialIndex {
    /// Build a spatial index from the Arrow tables in `parts`.
    ///
    /// Per-object bounding boxes come from the `geographical_extent` column
    /// in the `cityobjects` table. Objects that lack a stored extent get one
    /// computed from their geometry vertices.
    ///
    /// # Panics
    ///
    /// Panics if the expected Arrow columns are missing or have the wrong
    /// concrete array type.
    #[must_use]
    pub fn build(parts: &CityModelArrowParts) -> Self {
        let batch = &parts.cityobjects;
        let num_rows = batch.num_rows();

        let object_id_col = batch
            .column_by_name("cityobject_id")
            .expect("cityobject_id column");
        let object_id_arr = object_id_col
            .as_any()
            .downcast_ref::<LargeStringArray>()
            .expect("cityobject_id as LargeUtf8");

        let row_index_col = batch
            .column_by_name("cityobject_ix")
            .expect("cityobject_ix column");
        let row_index_arr = row_index_col
            .as_any()
            .downcast_ref::<UInt64Array>()
            .expect("cityobject_ix as UInt64");

        let type_col = batch
            .column_by_name("object_type")
            .expect("object_type column");
        let type_arr = type_col
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("object_type as Utf8");

        // Attempt to read geographical_extent from the cityobjects table.
        let ext_col = batch.column_by_name("geographical_extent");
        let (ext_list, ext_values) = ext_col
            .and_then(|c| {
                let list = c.as_any().downcast_ref::<FixedSizeListArray>()?;
                let vals = list.values().as_any().downcast_ref::<Float64Array>()?;
                Some((list, vals))
            })
            .unzip();

        // Precompute fallback bboxes from vertices for objects that have no
        // stored geographical_extent.
        let fallback = compute_fallback_bboxes(parts);

        let mut entries = Vec::with_capacity(num_rows);
        let mut overall_extent: Option<BBox2D> = None;
        let mut overall_3d: Option<[f64; 6]> = None;

        for row in 0..num_rows {
            let cityobject_ix = row_index_arr.value(row);

            // Try stored extent first, then fallback.
            let ext = ext_list
                .filter(|l| !l.is_null(row))
                .and_then(|_| {
                    let vals = ext_values.as_ref()?;
                    let o = row * 6;
                    Some([
                        vals.value(o),
                        vals.value(o + 1),
                        vals.value(o + 2),
                        vals.value(o + 3),
                        vals.value(o + 4),
                        vals.value(o + 5),
                    ])
                })
                .or_else(|| fallback.get(&cityobject_ix).copied());

            let Some(extent_3d) = ext else { continue };

            let bbox = BBox2D::new(extent_3d[0], extent_3d[1], extent_3d[3], extent_3d[4]);

            overall_extent = Some(match overall_extent {
                Some(e) => e.union(&bbox),
                None => bbox,
            });
            overall_3d = Some(match overall_3d {
                Some(e) => [
                    e[0].min(extent_3d[0]),
                    e[1].min(extent_3d[1]),
                    e[2].min(extent_3d[2]),
                    e[3].max(extent_3d[3]),
                    e[4].max(extent_3d[4]),
                    e[5].max(extent_3d[5]),
                ],
                None => extent_3d,
            });

            let id = object_id_arr.value(row).to_string();
            let object_type = type_arr.value(row).to_string();

            entries.push(SpatialEntry {
                hilbert_index: 0,
                row_index: row,
                cityobject_ix,
                bbox,
                extent_3d,
                id,
                object_type,
            });
        }

        let extent = overall_extent.unwrap_or(BBox2D::new(0.0, 0.0, 1.0, 1.0));
        let extent_3d = overall_3d.unwrap_or([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);

        // Compute Hilbert indices from centroids and sort.
        let n = 1u32 << HILBERT_ORDER;
        for entry in &mut entries {
            let (cx, cy) = entry.bbox.center();
            let (gx, gy) = normalize_to_grid(cx, cy, &extent, n);
            entry.hilbert_index = xy_to_hilbert(n, gx, gy);
        }
        entries.sort_by_key(|e| e.hilbert_index);

        SpatialIndex {
            extent,
            extent_3d,
            total_objects: num_rows,
            entries,
        }
    }

    /// Return every entry whose bounding box intersects `bbox`.
    #[must_use]
    pub fn query(&self, bbox: &BBox2D) -> Vec<&SpatialEntry> {
        self.entries
            .iter()
            .filter(|e| e.bbox.intersects(bbox))
            .collect()
    }

    /// All indexed entries, in Hilbert-curve order.
    #[must_use]
    pub fn entries(&self) -> &[SpatialEntry] {
        &self.entries
    }

    /// Number of indexed entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Fallback bbox computation from geometry vertices
// ---------------------------------------------------------------------------

/// For each `cityobject_ix`, compute a 3D bbox from the vertices referenced
/// by its geometries. Returns a map from `cityobject_ix` to
/// `[xmin, ymin, zmin, xmax, ymax, zmax]`.
fn compute_fallback_bboxes(parts: &CityModelArrowParts) -> HashMap<u64, [f64; 6]> {
    // 1. Build vertex coordinate arrays.
    let vb = &parts.vertices;
    let x_arr = col_f64(vb, "x");
    let y_arr = col_f64(vb, "y");
    let z_arr = col_f64(vb, "z");

    // 2. Map geometry_id → cityobject_ix.
    let gb = &parts.geometries;
    let geometry_id_arr = col_u64(gb, "geometry_id");
    let geometry_cityobject_ix_arr = col_u64(gb, "cityobject_ix");
    let mut geom_to_obj: HashMap<u64, u64> = HashMap::with_capacity(gb.num_rows());
    for i in 0..gb.num_rows() {
        geom_to_obj.insert(
            geometry_id_arr.value(i),
            geometry_cityobject_ix_arr.value(i),
        );
    }

    // 3. Walk geometry_boundaries and accumulate vertex extents per object.
    let bb = &parts.geometry_boundaries;
    let bgid_arr = col_u64(bb, "geometry_id");
    let vi_col = bb
        .column_by_name("vertex_indices")
        .expect("vertex_indices column");
    let vi_arr = vi_col
        .as_any()
        .downcast_ref::<ListArray>()
        .expect("vertex_indices as List");

    let mut bboxes: HashMap<u64, [f64; 6]> = HashMap::new();

    for row in 0..bb.num_rows() {
        let gid = bgid_arr.value(row);
        let Some(&obj_ix) = geom_to_obj.get(&gid) else {
            continue;
        };
        if vi_arr.is_null(row) {
            continue;
        }
        let vi_values = vi_arr.value(row);
        let vi_u32 = vi_values
            .as_any()
            .downcast_ref::<UInt32Array>()
            .expect("vertex index as UInt32");

        let bb_entry = bboxes.entry(obj_ix).or_insert([
            f64::INFINITY,
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ]);

        for i in 0..vi_u32.len() {
            let vid = vi_u32.value(i) as usize;
            if vid < x_arr.len() {
                let x = x_arr.value(vid);
                let y = y_arr.value(vid);
                let z = z_arr.value(vid);
                bb_entry[0] = bb_entry[0].min(x);
                bb_entry[1] = bb_entry[1].min(y);
                bb_entry[2] = bb_entry[2].min(z);
                bb_entry[3] = bb_entry[3].max(x);
                bb_entry[4] = bb_entry[4].max(y);
                bb_entry[5] = bb_entry[5].max(z);
            }
        }
    }

    bboxes
}

// ---------------------------------------------------------------------------
// Arrow column helpers
// ---------------------------------------------------------------------------

fn col_f64<'a>(batch: &'a RecordBatch, name: &str) -> &'a Float64Array {
    batch
        .column_by_name(name)
        .unwrap_or_else(|| panic!("{name} column"))
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap_or_else(|| panic!("{name} as Float64"))
}

fn col_u64<'a>(batch: &'a RecordBatch, name: &str) -> &'a UInt64Array {
    batch
        .column_by_name(name)
        .unwrap_or_else(|| panic!("{name} column"))
        .as_any()
        .downcast_ref::<UInt64Array>()
        .unwrap_or_else(|| panic!("{name} as UInt64"))
}

// ---------------------------------------------------------------------------
// Hilbert curve
// ---------------------------------------------------------------------------

/// Normalize a world coordinate to a grid cell in `[0, n)`.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]
fn normalize_to_grid(x_coord: f64, y_coord: f64, extent: &BBox2D, grid_size: u32) -> (u32, u32) {
    let width = extent.max_x - extent.min_x;
    let height = extent.max_y - extent.min_y;
    let max_cell = grid_size - 1;
    let grid_x = if width > 0.0 {
        (((x_coord - extent.min_x) / width * f64::from(max_cell)) as u32).min(max_cell)
    } else {
        0
    };
    let grid_y = if height > 0.0 {
        (((y_coord - extent.min_y) / height * f64::from(max_cell)) as u32).min(max_cell)
    } else {
        0
    };
    (grid_x, grid_y)
}

/// Convert `(x, y)` grid coordinates in `[0, n)` to a Hilbert curve index
/// in `[0, n²)`.  `n` must be a power of two.
///
/// Uses the standard algorithm from *"Hilbert Curves"* (Wikipedia) which
/// processes one bit-level at a time from MSB to LSB, rotating the
/// sub-quadrant at each step.
fn xy_to_hilbert(grid_size: u32, mut grid_x: u32, mut grid_y: u32) -> u64 {
    debug_assert!(grid_size.is_power_of_two());
    let mut distance: u64 = 0;
    let mut step = grid_size >> 1;
    while step > 0 {
        let rx = u32::from((grid_x & step) > 0);
        let ry = u32::from((grid_y & step) > 0);
        distance += u64::from(step) * u64::from(step) * u64::from((3 * rx) ^ ry);
        // Rotate / flip the quadrant.
        if ry == 0 {
            if rx == 1 {
                grid_x = (step - 1).wrapping_sub(grid_x);
                grid_y = (step - 1).wrapping_sub(grid_y);
            }
            std::mem::swap(&mut grid_x, &mut grid_y);
        }
        step >>= 1;
    }
    distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hilbert_2x2() {
        assert_eq!(xy_to_hilbert(2, 0, 0), 0);
        assert_eq!(xy_to_hilbert(2, 0, 1), 1);
        assert_eq!(xy_to_hilbert(2, 1, 1), 2);
        assert_eq!(xy_to_hilbert(2, 1, 0), 3);
    }

    #[test]
    fn hilbert_covers_all_cells() {
        let n = 8u32;
        let mut seen = std::collections::HashSet::new();
        for y in 0..n {
            for x in 0..n {
                let d = xy_to_hilbert(n, x, y);
                assert!(d < u64::from(n) * u64::from(n));
                seen.insert(d);
            }
        }
        assert_eq!(seen.len(), (n * n) as usize);
    }

    #[test]
    fn hilbert_large_order() {
        let n = 1u32 << 16;
        let d = xy_to_hilbert(n, 0, 0);
        assert_eq!(d, 0);
        let d = xy_to_hilbert(n, n - 1, n - 1);
        assert!(d < u64::from(n) * u64::from(n));
    }

    #[test]
    fn bbox_intersects() {
        let a = BBox2D::new(0.0, 0.0, 2.0, 2.0);
        let b = BBox2D::new(1.0, 1.0, 3.0, 3.0);
        let c = BBox2D::new(3.5, 3.5, 4.0, 4.0);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn bbox_union() {
        let a = BBox2D::new(0.0, 0.0, 1.0, 1.0);
        let b = BBox2D::new(2.0, 3.0, 4.0, 5.0);
        let u = a.union(&b);
        assert!((u.min_x - 0.0_f64).abs() < f64::EPSILON);
        assert!((u.min_y - 0.0_f64).abs() < f64::EPSILON);
        assert!((u.max_x - 4.0_f64).abs() < f64::EPSILON);
        assert!((u.max_y - 5.0_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn normalize_grid_extremes() {
        let extent = BBox2D::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(normalize_to_grid(0.0, 0.0, &extent, 256), (0, 0));
        assert_eq!(normalize_to_grid(100.0, 100.0, &extent, 256), (255, 255));
    }

    #[test]
    fn normalize_grid_zero_extent() {
        let extent = BBox2D::new(5.0, 5.0, 5.0, 5.0);
        assert_eq!(normalize_to_grid(5.0, 5.0, &extent, 256), (0, 0));
    }
}
