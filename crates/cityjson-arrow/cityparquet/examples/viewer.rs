//! Simple 3D web viewer for CityParquet files.
//!
//! Opens a `.cityparquet` file, builds a Hilbert-curve spatial index, and
//! serves a three.js viewer on `http://localhost:8080`.  The viewer requests
//! geometry for the visible area only, using the spatial index for culling.
//!
//! ```text
//! cargo run --example viewer -- path/to/model.cityparquet
//! ```

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Write as _};
use std::net::{TcpListener, TcpStream};

use arrow::array::{
    Array, Float64Array, LargeStringArray, ListArray, StringArray, UInt32Array, UInt64Array,
};
use arrow::record_batch::RecordBatch;
use cityarrow::schema::CityModelArrowParts;
use cityparquet::spatial::{BBox2D, SpatialIndex};

// ---------------------------------------------------------------------------
// Scene data — precomputed geometry lookup tables
// ---------------------------------------------------------------------------

struct SceneData {
    /// Vertex positions indexed by vertex_id: `[x, y, z]` (centered).
    vertices: Vec<[f64; 3]>,
    /// `cityobject_ix` -> (id, object_type).
    objects: HashMap<u64, (String, String)>,
    /// `geometry_id` -> list of surfaces (exterior rings only, as vertex indices).
    surfaces: HashMap<u64, Vec<Vec<u32>>>,
    /// `cityobject_ix` -> list of geometry_ids.
    object_geoms: HashMap<u64, Vec<u64>>,
    /// Centroid used to center coordinates (world coords).
    centroid: [f64; 3],
}

impl SceneData {
    fn build(parts: &CityModelArrowParts, centroid: [f64; 3]) -> Self {
        let vertices = Self::build_vertices(&parts.vertices, centroid);
        let objects = Self::build_objects(&parts.cityobjects);
        let object_geoms = Self::build_object_geoms(&parts.geometries);
        let surfaces = Self::build_surfaces(&parts.geometry_boundaries);

        Self {
            vertices,
            objects,
            surfaces,
            object_geoms,
            centroid,
        }
    }

    fn build_vertices(vb: &RecordBatch, c: [f64; 3]) -> Vec<[f64; 3]> {
        let x = col_f64(vb, "x");
        let y = col_f64(vb, "y");
        let z = col_f64(vb, "z");
        (0..vb.num_rows())
            .map(|i| [x.value(i) - c[0], y.value(i) - c[1], z.value(i) - c[2]])
            .collect()
    }

    fn build_objects(ob: &RecordBatch) -> HashMap<u64, (String, String)> {
        let id = ob
            .column_by_name("cityobject_id")
            .unwrap()
            .as_any()
            .downcast_ref::<LargeStringArray>()
            .unwrap();
        let ix = col_u64(ob, "cityobject_ix");
        let ty = ob
            .column_by_name("object_type")
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let mut map = HashMap::with_capacity(ob.num_rows());
        for i in 0..ob.num_rows() {
            map.insert(
                ix.value(i),
                (id.value(i).to_string(), ty.value(i).to_string()),
            );
        }
        map
    }

    fn build_object_geoms(gb: &RecordBatch) -> HashMap<u64, Vec<u64>> {
        let gid = col_u64(gb, "geometry_id");
        let oix = col_u64(gb, "cityobject_ix");
        let mut map: HashMap<u64, Vec<u64>> = HashMap::new();
        for i in 0..gb.num_rows() {
            map.entry(oix.value(i)).or_default().push(gid.value(i));
        }
        map
    }

    fn build_surfaces(bb: &RecordBatch) -> HashMap<u64, Vec<Vec<u32>>> {
        let gid = col_u64(bb, "geometry_id");
        let vi_col = bb.column_by_name("vertex_indices").unwrap();
        let vi_arr = vi_col.as_any().downcast_ref::<ListArray>().unwrap();
        let so_col = bb.column_by_name("surface_offsets");
        let ro_col = bb.column_by_name("ring_offsets");

        let mut map: HashMap<u64, Vec<Vec<u32>>> = HashMap::with_capacity(bb.num_rows());

        for row in 0..bb.num_rows() {
            let geometry_id = gid.value(row);
            if vi_arr.is_null(row) {
                continue;
            }
            let vi_vals = vi_arr.value(row);
            let vis: Vec<u32> = {
                let arr = vi_vals.as_any().downcast_ref::<UInt32Array>().unwrap();
                (0..arr.len()).map(|j| arr.value(j)).collect()
            };

            let surfaces = decode_surfaces(&vis, row, so_col, ro_col);
            map.insert(geometry_id, surfaces);
        }
        map
    }

    /// Produce a flat `[x, y, z, ...]` triangle buffer for one object.
    /// Coordinates are centered and swizzled for three.js (x, z_up, -y).
    fn triangulate_object(&self, cityobject_ix: u64) -> Vec<f64> {
        let mut tris = Vec::new();
        let Some(geom_ids) = self.object_geoms.get(&cityobject_ix) else {
            return tris;
        };
        for &gid in geom_ids {
            let Some(surfaces) = self.surfaces.get(&gid) else {
                continue;
            };
            for ring in surfaces {
                if ring.len() < 3 {
                    continue;
                }
                let v0 = self.vertex_swizzled(ring[0] as u64);
                for i in 1..ring.len() - 1 {
                    let v1 = self.vertex_swizzled(ring[i] as u64);
                    let v2 = self.vertex_swizzled(ring[i + 1] as u64);
                    tris.extend_from_slice(&v0);
                    tris.extend_from_slice(&v1);
                    tris.extend_from_slice(&v2);
                }
            }
        }
        tris
    }

    /// Get vertex and swizzle for three.js: CityJSON (x,y,z) -> three.js (x, z, -y).
    fn vertex_swizzled(&self, vid: u64) -> [f64; 3] {
        let v = self.vertices[vid as usize];
        [v[0], v[2], -v[1]]
    }
}

// ---------------------------------------------------------------------------
// Boundary decoding
// ---------------------------------------------------------------------------

/// Decode surfaces from offset-based boundary encoding (v3alpha2).
///
/// - `ring_offsets[i]` = start index into `vertex_indices` for ring `i`.
/// - `surface_offsets[i]` = start index into `ring_offsets` for surface `i`.
/// - End of element `i` = start of element `i+1`, or total child length if last.
fn decode_surfaces(
    vis: &[u32],
    row: usize,
    so_col: Option<&std::sync::Arc<dyn Array>>,
    ro_col: Option<&std::sync::Arc<dyn Array>>,
) -> Vec<Vec<u32>> {
    let mut surfaces = Vec::new();

    let ro_list = ro_col.and_then(|c| c.as_any().downcast_ref::<ListArray>());
    let so_list = so_col.and_then(|c| c.as_any().downcast_ref::<ListArray>());

    // surface_offsets + ring_offsets present: full multi-surface/solid.
    if let (Some(so), Some(ro)) = (so_list, ro_list) {
        if !so.is_null(row) && !ro.is_null(row) {
            let so_vals = list_u32(so, row);
            let ro_vals = list_u32(ro, row);

            for (si, &surf_start) in so_vals.iter().enumerate() {
                let surf_start = surf_start as usize;
                let surf_end = so_vals
                    .get(si + 1)
                    .map(|&v| v as usize)
                    .unwrap_or(ro_vals.len());
                // Exterior ring is the first ring of this surface.
                if surf_start < surf_end && surf_start < ro_vals.len() {
                    let ring_vi_start = ro_vals[surf_start] as usize;
                    let ring_vi_end = ro_vals
                        .get(surf_start + 1)
                        .map(|&v| v as usize)
                        .unwrap_or(vis.len());
                    let end = ring_vi_end.min(vis.len());
                    let start = ring_vi_start.min(end);
                    surfaces.push(vis[start..end].to_vec());
                }
            }
            return surfaces;
        }
    }

    // ring_offsets only: each ring is a surface.
    if let Some(ro) = ro_list {
        if !ro.is_null(row) {
            let ro_vals = list_u32(ro, row);
            for (ri, &ring_start) in ro_vals.iter().enumerate() {
                let start = (ring_start as usize).min(vis.len());
                let end = ro_vals
                    .get(ri + 1)
                    .map(|&v| (v as usize).min(vis.len()))
                    .unwrap_or(vis.len());
                surfaces.push(vis[start..end].to_vec());
            }
            return surfaces;
        }
    }

    // Fallback: entire vertex_indices is a single polygon.
    if !vis.is_empty() {
        surfaces.push(vis.to_vec());
    }
    surfaces
}

fn list_u32(arr: &ListArray, row: usize) -> Vec<u32> {
    let v = arr.value(row);
    let a = v.as_any().downcast_ref::<UInt32Array>().unwrap();
    (0..a.len()).map(|j| a.value(j)).collect()
}

// ---------------------------------------------------------------------------
// Arrow helpers
// ---------------------------------------------------------------------------

fn col_f64<'a>(batch: &'a RecordBatch, name: &str) -> &'a Float64Array {
    batch
        .column_by_name(name)
        .unwrap_or_else(|| panic!("{name}"))
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap_or_else(|| panic!("{name}"))
}

fn col_u64<'a>(batch: &'a RecordBatch, name: &str) -> &'a UInt64Array {
    batch
        .column_by_name(name)
        .unwrap_or_else(|| panic!("{name}"))
        .as_any()
        .downcast_ref::<UInt64Array>()
        .unwrap_or_else(|| panic!("{name}"))
}

// ---------------------------------------------------------------------------
// HTTP server
// ---------------------------------------------------------------------------

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: viewer <file.cityparquet>");
        std::process::exit(1);
    });

    eprintln!("Reading {path} ...");
    let parts = cityparquet::read_package_parts_file(&path).expect("failed to read package");

    eprintln!("Building spatial index ...");
    let index = SpatialIndex::build(&parts);
    eprintln!(
        "  {} objects indexed, extent: ({:.1}, {:.1}) - ({:.1}, {:.1})",
        index.len(),
        index.extent.min_x,
        index.extent.min_y,
        index.extent.max_x,
        index.extent.max_y,
    );

    let centroid = [
        (index.extent_3d[0] + index.extent_3d[3]) * 0.5,
        (index.extent_3d[1] + index.extent_3d[4]) * 0.5,
        (index.extent_3d[2] + index.extent_3d[5]) * 0.5,
    ];

    eprintln!("Pre-computing geometry ...");
    let scene = SceneData::build(&parts, centroid);
    drop(parts);

    let listener = TcpListener::bind("0.0.0.0:8080").expect("failed to bind :8080");
    eprintln!("Viewer ready at http://localhost:8080");

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        if let Err(e) = handle_request(&stream, &index, &scene) {
            eprintln!("request error: {e}");
        }
    }
}

fn handle_request(
    stream: &TcpStream,
    index: &SpatialIndex,
    scene: &SceneData,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    // Consume remaining headers.
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
    }

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    let (status, content_type, body) = route(path, index, scene);
    let mut out = stream.try_clone()?;
    write!(
        out,
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Connection: close\r\n\
         \r\n",
        body.len(),
    )?;
    out.write_all(body.as_bytes())?;
    out.flush()
}

fn route(path: &str, index: &SpatialIndex, scene: &SceneData) -> (&'static str, &'static str, String) {
    let (path_base, query) = path.split_once('?').unwrap_or((path, ""));

    match path_base {
        "/" => (
            "200 OK",
            "text/html; charset=utf-8",
            VIEWER_HTML.to_string(),
        ),
        "/api/extent" => ("200 OK", "application/json", extent_json(index, scene)),
        "/api/objects" => {
            let params = parse_qs(query);
            (
                "200 OK",
                "application/json",
                objects_json(index, scene, &params),
            )
        }
        _ => ("404 Not Found", "text/plain", "not found".to_string()),
    }
}

// ---------------------------------------------------------------------------
// API handlers
// ---------------------------------------------------------------------------

fn extent_json(index: &SpatialIndex, scene: &SceneData) -> String {
    let e = index.extent_3d;
    let c = scene.centroid;
    format!(
        r#"{{"min_x":{:.6},"min_y":{:.6},"min_z":{:.6},"max_x":{:.6},"max_y":{:.6},"max_z":{:.6},"centroid":[{:.6},{:.6},{:.6}],"count":{}}}"#,
        e[0] - c[0],
        e[1] - c[1],
        e[2] - c[2],
        e[3] - c[0],
        e[4] - c[1],
        e[5] - c[2],
        c[0],
        c[1],
        c[2],
        index.len(),
    )
}

fn objects_json(index: &SpatialIndex, scene: &SceneData, params: &HashMap<&str, &str>) -> String {
    let get_f64 = |k| params.get(k).and_then(|v| v.parse::<f64>().ok());
    let limit: usize = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);

    let (entries, truncated): (Vec<_>, bool) = if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = (
        get_f64("minx"),
        get_f64("miny"),
        get_f64("maxx"),
        get_f64("maxy"),
    ) {
        let query_box = BBox2D::new(
            min_x + scene.centroid[0],
            min_y + scene.centroid[1],
            max_x + scene.centroid[0],
            max_y + scene.centroid[1],
        );
        let mut hits = index.query(&query_box);
        let truncated = hits.len() > limit;
        hits.truncate(limit);
        (hits, truncated)
    } else {
        // No bbox: return all (up to limit).
        let entries: Vec<_> = index.entries().iter().take(limit).collect();
        (entries, index.len() > limit)
    };

    let mut buf = String::from("{\"truncated\":");
    buf.push_str(if truncated { "true" } else { "false" });
    buf.push_str(",\"objects\":[");
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let tris = scene.triangulate_object(entry.cityobject_ix);
        let (id, ty) = scene
            .objects
            .get(&entry.cityobject_ix)
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .unwrap_or(("?", "?"));
        buf.push_str(&format!(
            r#"{{"id":"{}","type":"{}","triangles":["#,
            escape_json(id),
            escape_json(ty)
        ));
        for (j, v) in tris.iter().enumerate() {
            if j > 0 {
                buf.push(',');
            }
            // Use limited precision for transfer size.
            write!(&mut buf, "{:.4}", v).ok();
        }
        buf.push_str("]}");
    }
    buf.push_str("]}");
    buf
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn parse_qs(qs: &str) -> HashMap<&str, &str> {
    qs.split('&')
        .filter_map(|pair| pair.split_once('='))
        .collect()
}

// ---------------------------------------------------------------------------
// Embedded HTML / JS viewer
// ---------------------------------------------------------------------------

const VIEWER_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>CityParquet Viewer</title>
<style>
  body { margin:0; overflow:hidden; background:#1a1a2e; font-family:monospace; }
  #info {
    position:absolute; top:12px; left:12px; color:#ccc; font-size:13px;
    background:rgba(0,0,0,0.55); padding:8px 12px; border-radius:6px;
    pointer-events:none; line-height:1.5;
  }
  #info b { color:#fff; }
</style>
</head>
<body>
<div id="info">Loading&hellip;</div>
<script type="importmap">
{
  "imports": {
    "three": "https://cdn.jsdelivr.net/npm/three@0.162.0/build/three.module.js",
    "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.162.0/examples/jsm/"
  }
}
</script>
<script type="module">
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

/* ---- renderer ---- */
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.setClearColor(0x1a1a2e);
document.body.appendChild(renderer.domElement);

/* ---- scene ---- */
const scene = new THREE.Scene();
scene.add(new THREE.AmbientLight(0x606080, 2.5));
const sun = new THREE.DirectionalLight(0xffffff, 1.8);
sun.position.set(1, 2, 1.5);
scene.add(sun);
scene.add(new THREE.HemisphereLight(0x8090b0, 0x303020, 0.8));

/* ---- camera ---- */
const camera = new THREE.PerspectiveCamera(50, window.innerWidth / window.innerHeight, 0.5, 100000);
const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;
controls.dampingFactor = 0.12;
// Start very close so the first visible-region query only loads a small slice
// of the model. This makes frustum culling and Hilbert-order traversal easier
// to observe in large datasets.
const STARTUP_VIEW_SCALE = 0.12;
const STARTUP_FOV = 32;

/* ---- state ---- */
const meshGroup = new THREE.Group();
scene.add(meshGroup);
const loadedMeshes = new Map();
let loading = false;
let extentData = null;

/* ---- colours ---- */
const TYPE_COLORS = {
  Building: 0x4a90d9,
  BuildingPart: 0x5ba3ec,
  BuildingInstallation: 0x7bbcf7,
  Road: 0x888888,
  Railway: 0x666666,
  LandUse: 0x4caf50,
  PlantCover: 0x2e7d32,
  SolitaryVegetationObject: 0x1b5e20,
  WaterBody: 0x2196f3,
  Bridge: 0xbcaaa4,
  Tunnel: 0x795548,
  CityFurniture: 0x9e9e9e,
  GenericCityObject: 0xbdbdbd,
  TINRelief: 0xa1887f,
};
function typeColor(t) { return TYPE_COLORS[t] || 0xcccccc; }

/* ---- info panel ---- */
const info = document.getElementById('info');
function setInfo(msg) { info.innerHTML = msg; }

/* ---- init ---- */
async function init() {
  const res = await fetch('/api/extent');
  extentData = await res.json();
  const e = extentData;

  // Centered extent (server already subtracted centroid).
  const sx = e.max_x - e.min_x;
  const sy = e.max_y - e.min_y;
  const sz = e.max_z - e.min_z;
  const size = Math.max(sx, sy, sz);
  const cx = (e.min_x + e.max_x) / 2;
  const cy = (e.min_y + e.max_y) / 2;   // → three.js z (negated)
  const cz = (e.min_z + e.max_z) / 2;   // → three.js y

  // three.js coords: (x, z_up, -y)
  camera.fov = STARTUP_FOV;
  camera.near = Math.max(size * 0.001, 0.05);
  camera.far = Math.max(size * 50.0, 1000.0);
  camera.updateProjectionMatrix();
  controls.target.set(cx, cz, -cy);
  camera.position.set(
    cx + size * 0.6 * STARTUP_VIEW_SCALE,
    cz + size * 0.8 * STARTUP_VIEW_SCALE,
    -cy + size * 0.6 * STARTUP_VIEW_SCALE,
  );
  controls.update();

  setInfo(`<b>${e.count}</b> objects &mdash; loading visible&hellip;`);
  await loadVisible();
  controls.addEventListener('change', debounce(loadVisible, 250));
}

/* ---- load visible objects ---- */
async function loadVisible() {
  if (loading) return;
  loading = true;

  // Compute a conservative 2D bbox around the camera view.
  const target = controls.target;
  const dist = camera.position.distanceTo(target);
  const halfFov = camera.fov * Math.PI / 360;
  const halfH = dist * Math.tan(halfFov);
  const halfW = halfH * camera.aspect;
  const r = Math.sqrt(halfW * halfW + halfH * halfH) * 1.2;

  // Map three.js (x, y_up, z) back to CityJSON (x, y) centered coords.
  // three.js x = CityJSON x, three.js z = -CityJSON y
  const minx = target.x - r;
  const miny = -target.z - r;   // un-negate z -> CityJSON y
  const maxx = target.x + r;
  const maxy = -target.z + r;

  try {
    const url = `/api/objects?minx=${minx}&miny=${miny}&maxx=${maxx}&maxy=${maxy}&limit=10000`;
    const data = await fetch(url).then(r => r.json());
    const visibleIds = new Set();
    let added = 0;
    for (const obj of data.objects) {
      visibleIds.add(obj.id);
      if (obj.triangles.length < 9) continue;
      if (loadedMeshes.has(obj.id)) continue;

      const positions = new Float32Array(obj.triangles);
      const geom = new THREE.BufferGeometry();
      geom.setAttribute('position', new THREE.BufferAttribute(positions, 3));
      geom.computeVertexNormals();

      const mat = new THREE.MeshPhongMaterial({
        color: typeColor(obj.type),
        side: THREE.DoubleSide,
        flatShading: true,
      });
      const mesh = new THREE.Mesh(geom, mat);
      meshGroup.add(mesh);
      loadedMeshes.set(obj.id, mesh);
      added++;
    }

    let removed = 0;
    if (!data.truncated) {
      for (const [id, mesh] of loadedMeshes.entries()) {
        if (visibleIds.has(id)) continue;
        meshGroup.remove(mesh);
        mesh.geometry.dispose();
        if (Array.isArray(mesh.material)) {
          for (const material of mesh.material) {
            material.dispose();
          }
        } else {
          mesh.material.dispose();
        }
        loadedMeshes.delete(id);
        removed++;
      }
    }

    const capNote = data.truncated ? ' (visible set capped)' : '';
    const removeNote = removed > 0 ? `, removed ${removed}` : '';
    setInfo(`<b>${loadedMeshes.size}</b> objects loaded${removeNote}${capNote}`);
  } catch (e) {
    console.error('load error', e);
  } finally {
    loading = false;
  }
}

/* ---- helpers ---- */
function debounce(fn, ms) {
  let timer;
  return () => { clearTimeout(timer); timer = setTimeout(fn, ms); };
}

/* ---- resize ---- */
window.addEventListener('resize', () => {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
});

/* ---- render loop ---- */
function animate() {
  requestAnimationFrame(animate);
  controls.update();
  renderer.render(scene, camera);
}

init();
animate();
</script>
</body>
</html>
"##;
