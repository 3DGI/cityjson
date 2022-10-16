//! Dereference architecture.

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use datasize::{data_size, DataSize};
use memmap2::Mmap;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};

/*
    Static geometry
*/
type Point = [f32; 3];
type LineString = Vec<Point>;
type Surface = Vec<LineString>;

#[derive(Debug, DataSize)]
enum LoD {
    LoD0,
    LoD1,
    LoD1_2,
    LoD1_3,
    LoD2_2,
}

// TODO: How to represent 'null' values in the semantics/appearance values arrays?
#[derive(Debug, DataSize)]
enum Geometry {
    MultiPoint {
        lod: Option<LoD>,
        boundaries: Vec<Point>,
    },
    MultiLineString {
        lod: Option<LoD>,
        boundaries: Vec<LineString>,
    },
    MultiSurface {
        lod: Option<LoD>,
        boundaries: Vec<Surface>,
        semantics_values: Option<Vec<Option<Rc<Semantic>>>>,
        textures_values: Option<Vec<Option<Rc<Texture>>>>,
        materials_values: Option<Vec<Option<Rc<Material>>>>,
    },
    Solid {
        lod: Option<LoD>,
        boundaries: Vec<Vec<Surface>>,
        semantics_values: Option<Vec<Vec<Option<Rc<Semantic>>>>>,
        textures_values: Option<Vec<Vec<Option<Rc<Texture>>>>>,
        materials_values: Option<Vec<Vec<Option<Rc<Material>>>>>,
    },
}

#[derive(Debug, DataSize, PartialEq, Eq)]
enum Semantic {
    TransportationHole,
    TransportationMarking,
    RoofSurface,
    GroundSurface,
    WallSurface,
    Unknown,
}

#[derive(Default, Debug, DataSize)]
struct Material {
    name: String,
    ambient_intensity: Option<f32>,
    diffuse_color: Option<[f32; 3]>,
    emissive_color: Option<[f32; 3]>,
    specular_color: Option<[f32; 3]>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    is_smooth: Option<bool>,
}

#[derive(Debug, DataSize)]
enum ImageType {
    Png,
    Jpg,
}
impl Default for ImageType {
    fn default() -> Self {
        ImageType::Png
    }
}
#[derive(Debug, DataSize)]
enum WrapMode {
    Wrap,
    Mirror,
    Clamp,
    Border,
}
#[derive(Debug, DataSize)]
enum TextureType {
    Unknown,
    Specific,
    Typical,
}

#[derive(Default, Debug, DataSize)]
struct Texture {
    image_type: ImageType,
    image: String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<[f32; 4]>,
}

#[derive(Default, Debug, DataSize)]
struct CityObject {
    cotype: String,
    children: Option<Vec<String>>,
    parents: Option<Vec<String>>,
    geometry: Option<Vec<Geometry>>,
}

#[derive(Debug, DataSize)]
pub struct CityModel {
    pub cmtype: String,
    pub version: String,
    cityobjects: HashMap<String, CityObject>,
    semantics: Option<Vec<Rc<Semantic>>>,
    textures: Option<Vec<Rc<Texture>>>,
    materials: Option<Vec<Rc<Material>>>,
    default_theme_texture: Option<String>,
    default_theme_material: Option<String>,
}

/*
    Deserializes from CityJSON
*/
// Deserialize into indexed CityJSON-like structures with serde
#[derive(Deserialize, Debug, DataSize)]
pub struct SemanticSurface {
    #[serde(rename = "type")]
    pub semtype: String,
}

#[derive(Deserialize, Debug, DataSize)]
pub struct ISemantics {
    pub surfaces: Vec<SemanticSurface>,
    pub values: Vec<Vec<usize>>,
}

pub type Vertices = Vec<[f32; 3]>;

// Indexed geometry
pub type IVertex = usize;
pub type IRing = Vec<IVertex>;
pub type ISurface = Vec<IRing>;
pub type IShell = Vec<ISurface>;
pub type IMultiSurface = Vec<ISurface>;
pub type ISolid = Vec<IShell>;

#[derive(Deserialize, Debug, DataSize)]
#[serde(tag = "type")]
pub enum IGeometry {
    MultiSurface {
        lod: String,
        boundaries: Vec<Vec<Vec<usize>>>,
        semantics: Option<ISemantics>,
    },
    Solid {
        lod: String,
        boundaries: Vec<Vec<Vec<Vec<usize>>>>,
        semantics: Option<ISemantics>,
    },
}

#[derive(Deserialize, Debug, DataSize)]
pub struct ICityObject {
    #[serde(rename = "type")]
    pub cotype: String,
    pub geometry: Vec<IGeometry>,
}

#[derive(Deserialize, Debug, DataSize, Default)]
struct Transform {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Deserialize, Debug, DataSize)]
pub struct CMVertices {
    #[serde(rename = "type")]
    pub cmtype: String,
    pub version: String,
    transform: Transform,
    #[serde(skip)]
    pub cityobjects: Vec<(String, ICityObject)>,
    pub vertices: Vertices,
}

fn boundary_dereference(
    vertices: &Vertices,
    geom: &IGeometry,
    cmsemantics: &mut Option<Vec<Rc<Semantic>>>,
) -> Option<Geometry> {
    match geom {
        IGeometry::Solid {
            lod,
            boundaries,
            semantics,
        } => {
            let mut semval: Option<Vec<Vec<Option<Rc<Semantic>>>>> = None;
            let mut new_solid_bdry = Vec::with_capacity(boundaries.len());
            for (shi, shell) in boundaries.iter().enumerate() {
                let mut new_shell = Vec::with_capacity(shell.len());
                for (sui, surface) in shell.iter().enumerate() {
                    let mut surface_bdry: Surface = Vec::with_capacity(surface.len());
                    for ring in surface {
                        let mut new_ring: LineString = Vec::with_capacity(ring.len());
                        for vtx_idx in ring {
                            let new_vertex: Point = vertices[*vtx_idx];
                            new_ring.push(new_vertex);
                        }
                        surface_bdry.push(new_ring);
                    }
                    new_shell.push(surface_bdry);
                }
                new_solid_bdry.push(new_shell);
            }
            // This could be moved inside the boundary loop, but having it here outside makes the
            // code more simple.
            let mut local_global_semantics_idx: Vec<usize> = Vec::new();
            if let Some(sem) = semantics {
                for (si, ss) in sem.surfaces.iter().enumerate() {
                    let _sem: Rc<Semantic> = Rc::new(match ss.semtype.as_str() {
                        "GroundSurface" => Semantic::GroundSurface,
                        "WallSurface" => Semantic::WallSurface,
                        "RoofSurface" => Semantic::RoofSurface,
                        &_ => {
                            // This is a hack of sorts, because we must return a Semantic, or
                            // use Option<Semantic>
                            Semantic::Unknown
                        }
                    });
                    let mut _cmsemantics_idx: usize;
                    if let Some(ref mut _csm) = cmsemantics {
                        if let Some(sidx) = _csm.iter().position(|r| r == &_sem) {
                            _cmsemantics_idx = sidx.clone();
                        } else {
                            _csm.push(_sem);
                            _cmsemantics_idx = _csm.len() - 1;
                        }
                    } else {
                        *cmsemantics = Some(vec![_sem]);
                        _cmsemantics_idx = 0;
                    }
                    local_global_semantics_idx.push(_cmsemantics_idx);
                }
                // TODO: How to handle null values?
                if let Some(ref _csm) = cmsemantics {
                    let mut _sv: Vec<Vec<Option<Rc<Semantic>>>> = Vec::new();
                    for shi in &sem.values {
                        let mut _suv: Vec<Option<Rc<Semantic>>> = Vec::new();
                        for sui in shi {
                            _suv.push(Some(
                                _csm[local_global_semantics_idx[sui.to_owned()]].clone(),
                            ));
                        }
                        _sv.push(_suv)
                    }
                    semval = Some(_sv);
                }
            }
            let lod_enum = match lod.as_str() {
                "0" => LoD::LoD0,
                "2.2" => LoD::LoD2_2,
                "1.2" => LoD::LoD1_2,
                "1.3" => LoD::LoD1_3,
                _ => LoD::LoD0,
            };
            Some(Geometry::Solid {
                lod: Some(lod_enum),
                boundaries: new_solid_bdry,
                semantics_values: semval,
                textures_values: None,
                materials_values: None,
            })
        }
        IGeometry::MultiSurface {
            lod,
            boundaries,
            semantics,
        } => {
            let mut semval: Option<Vec<Option<Rc<Semantic>>>> = None;
            let mut new_msrf_bdry = Vec::with_capacity(boundaries.len());
            for (sui, surface) in boundaries.iter().enumerate() {
                let mut surface_bdry: Surface = Vec::with_capacity(surface.len());
                for ring in surface {
                    let mut new_ring: LineString = Vec::with_capacity(ring.len());
                    for vtx_idx in ring {
                        let new_vertex: Point = vertices[*vtx_idx];
                        new_ring.push(new_vertex);
                    }
                    surface_bdry.push(new_ring);
                }
                new_msrf_bdry.push(surface_bdry);
            }
            Some(Geometry::MultiSurface {
                lod: Some(LoD::LoD2_2),
                boundaries: new_msrf_bdry,
                semantics_values: semval,
                textures_values: None,
                materials_values: None,
            })
        }
        _ => {
            println!("Geometry type not implemented");
            None
        }
    }
}

struct CityObjectsMap<'a>(
    &'a mut HashMap<String, CityObject>,
    &'a CMVertices,
    &'a mut Option<Vec<Rc<Semantic>>>,
);

impl<'de, 'a> DeserializeSeed<'de> for CityObjectsMap<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CityObjectsMapVisitor<'a>(
            &'a mut HashMap<String, CityObject>,
            &'a CMVertices,
            &'a mut Option<Vec<Rc<Semantic>>>,
        );

        impl<'de, 'a> Visitor<'de> for CityObjectsMapVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a Geometry object")
            }

            //noinspection RsMainFunctionNotFound
            fn visit_map<A>(mut self, mut map: A) -> Result<(), A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut nr_geometries: usize = 0;
                let mut boundaries_sizes: usize = 0;
                let mut nr_surfaces: usize = 0;
                let mut surfaces_sizes: usize = 0;
                let mut srf_size_boundary: usize = 0;
                let mut nr_surfaces_per_geom: usize = 0;
                let mut nr_points: usize = 0;
                let mut empty_allocation: usize = 0;
                while let Some((coid, co)) = map.next_entry::<String, ICityObject>()? {
                    let mut new_geoms: Vec<Geometry> = Vec::with_capacity(co.geometry.len());
                    for geom in &co.geometry {
                        if let Some(g) = boundary_dereference(&self.1.vertices, geom, &mut self.2) {
                            nr_geometries += 1;
                            let mut geomsrf: usize = 0;
                            match &g {
                                Geometry::Solid {
                                    lod, boundaries, ..
                                } => {
                                    boundaries_sizes += data_size(boundaries);
                                    empty_allocation += boundaries.capacity() - boundaries.len();
                                    for shell in boundaries {
                                        empty_allocation += shell.capacity() - shell.len();
                                        for srf in shell {
                                            nr_surfaces += 1;
                                            surfaces_sizes +=
                                                data_size(srf) + std::mem::size_of_val(srf);
                                            srf_size_boundary +=
                                                data_size(&srf) + std::mem::size_of_val(&srf);
                                            geomsrf += 1;
                                            empty_allocation += srf.capacity() - srf.capacity();
                                            for ring in srf {
                                                nr_points += ring.len();
                                                empty_allocation += ring.capacity() - ring.len();
                                            }
                                        }
                                    }
                                }
                                Geometry::MultiSurface {
                                    lod, boundaries, ..
                                } => {
                                    boundaries_sizes += data_size(boundaries);
                                    empty_allocation += boundaries.capacity() - boundaries.len();
                                    for srf in boundaries {
                                        nr_surfaces += 1;
                                        surfaces_sizes +=
                                            data_size(srf) + std::mem::size_of_val(srf);
                                        srf_size_boundary +=
                                            data_size(&srf) + std::mem::size_of_val(&srf);
                                        geomsrf += 1;
                                        empty_allocation += srf.capacity() - srf.capacity();
                                        for ring in srf {
                                            nr_points += ring.len();
                                            empty_allocation += ring.capacity() - ring.len();
                                        }
                                    }
                                }
                                _ => {}
                            }
                            nr_surfaces_per_geom += geomsrf;
                            new_geoms.push(g);
                        }
                    }
                    new_geoms.shrink_to_fit();
                    self.0.insert(
                        coid,
                        CityObject {
                            cotype: co.cotype,
                            geometry: Some(new_geoms),
                            children: None,
                            parents: None,
                        },
                    );
                }
                println!("size of a Surface [b] {}", std::mem::size_of::<Surface>());
                println!(
                    "size of a Surface.boundaries [b] {}",
                    std::mem::size_of::<Vec<LineString>>()
                );
                println!(
                    "size of a Surface.semantics [b] {}",
                    std::mem::size_of::<Semantic>()
                );
                println!(
                    "size of a Surface.textures [b] {}",
                    std::mem::size_of::<Texture>()
                );
                println!(
                    "size of a Surface.materials [b] {}",
                    std::mem::size_of::<Material>()
                );
                println!("nr. of points in boundaries {}", nr_points);
                println!(
                    "total size of point allocations [Mb] {}",
                    (nr_points * 24) as f32 / 1e+6
                );
                println!("nr. surfaces {}", nr_surfaces);
                println!("surfaces sizes [Mb] {}", surfaces_sizes as f32 / 1e+6);
                println!(
                    "avg. surfaces size [b] {}",
                    surfaces_sizes as f32 / nr_surfaces as f32
                );
                println!("srf boundary [Mb] {}", srf_size_boundary as f32 / 1e+6);
                println!("nr. geometries {}", nr_geometries);
                println!(
                    "total boundary (Geometry) size [Mb] {}",
                    boundaries_sizes as f32 / 1e+6
                );
                println!(
                    "avg. boundary (Geometry) size [b] {}",
                    boundaries_sizes as f32 / nr_geometries as f32
                );
                println!("empty allocation in boundary vectors {}", empty_allocation);
                Ok(())
            }
        }
        deserializer.deserialize_map(CityObjectsMapVisitor(self.0, self.1, self.2))
    }
}

struct CityModelMap<'a>(&'a mut CityModel, &'a CMVertices);

impl<'de, 'a> DeserializeSeed<'de> for CityModelMap<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        // Custom Visitor for doing the second pass over the data and getting to the entries of the
        // "geometries" object.
        struct CityModelMapVisitor<'a>(&'a mut CityModel, &'a CMVertices);

        impl<'de, 'a> Visitor<'de> for CityModelMapVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a valid file")
            }

            fn visit_map<A>(self, mut map: A) -> Result<(), A::Error>
            where
                A: MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<String>()? {
                    if key == "CityObjects" {
                        let a = map.next_value_seed(CityObjectsMap(
                            &mut self.0.cityobjects,
                            &self.1,
                            &mut self.0.semantics,
                        ))?;
                        self.0.cityobjects.shrink_to_fit();
                    } else {
                        let ignore_value: IgnoredAny = map.next_value::<IgnoredAny>()?;
                    }
                }
                Ok(())
            }
        }
        self.0.cmtype = self.1.cmtype.clone();
        self.0.version = self.1.version.clone();
        deserializer.deserialize_map(CityModelMapVisitor(self.0, self.1));
        Ok(())
    }
}

pub fn parse_dereferece(path_in: PathBuf) -> CityModel {
    let mut cm_vertices: CMVertices;
    {
        let mut file = File::open(&path_in).expect("Couldn't open CityJSON file");
        let mmap = unsafe { Mmap::map(&file).expect("Cannot memmap the file") };
        cm_vertices = serde_json::from_slice(&mmap).expect("Couldn't deserialize into CMVertices");
    }

    let mut cm = CityModel {
        cmtype: Default::default(),
        version: Default::default(),
        cityobjects: HashMap::new(),
        semantics: None,
        textures: None,
        materials: None,
        default_theme_material: None,
        default_theme_texture: None,
    };

    let cm_map = CityModelMap(&mut cm, &cm_vertices);

    let file = File::open(&path_in).expect("Couldn't open CityJSON file");
    // let reader = BufReader::new(&file);
    // let mut deserializer = serde_json::Deserializer::from_reader(reader);
    let mmap = unsafe { Mmap::map(&file).expect("Cannot memmap the file") };
    let mut deserializer = serde_json::Deserializer::from_slice(&mmap);
    cm_map.deserialize(&mut deserializer);

    println!("nr cityobjects {}", cm.cityobjects.len());
    println!(
        "estimated heap allocation of indexed-citymodel [Mb]: {}",
        data_size(&cm_vertices) as f32 / 1e+6
    );
    println!(
        "estimated heap allocation of target citymodel [Mb]: {}",
        data_size(&cm) as f32 / 1e+6
    );
    println!(
        "avg. cityobject heap allocation [b] {}",
        data_size(&cm.cityobjects) as f32 / cm.cityobjects.len() as f32
    );
    if let Some(ref _s) = cm.semantics {
        for i in _s.iter() {
            println!("{:?}", i);
        }
    }

    cm
}

pub fn deref_deserialize(path_in: PathBuf) {
    let cm = parse_dereferece(path_in);
}

pub fn deref_geometry(path_in: PathBuf) {
    /*    let mut containter: Vec<[f64; 3]> = Vec::new();
    let cm = parse_dereferece(path_in);
    for (coid, co) in cm.cityobjects {
        for geom in co.geometry {
            match geom {
                geom_static::Geometry::MultiPoint { .. } => {}
                geom_static::Geometry::MultiLineString { .. } => {}
                geom_static::Geometry::MultiSurface { boundaries, .. } => {
                    for surface in boundaries {
                        for ring in surface.boundaries {
                            for vtx in ring {
                                containter.push(vtx);
                            }
                        }
                    }
                }
                geom_static::Geometry::CompositeSurface { .. } => {}
                geom_static::Geometry::Solid { boundaries, .. } => {
                    for shell in boundaries {
                        for surface in shell {
                            for ring in surface.boundaries {
                                for vtx in ring {
                                    containter.push(vtx);
                                }
                            }
                        }
                    }
                }
                geom_static::Geometry::MultiSolid { .. } => {}
                geom_static::Geometry::CompositeSolid { .. } => {}
            }
        }
    }
    println!(
        "In total there are {} points in the citymodel",
        containter.len()
    )*/
}

pub fn deref_semantics(path_in: PathBuf) {
    /*   let semantic_type = "RoofSurface";
    println!("extracting the geometry of {}", semantic_type);
    let mut containter: Vec<[f64; 3]> = Vec::new();
    let cm = parse_dereferece(path_in);
    for (coid, co) in cm.cityobjects {
        for geom in co.geometry {
            match geom {
                geom_static::Geometry::MultiPoint { .. } => {}
                geom_static::Geometry::MultiLineString { .. } => {}
                geom_static::Geometry::MultiSurface { boundaries, .. } => {
                    for surface in boundaries {
                        if let Some(semsrf) = surface.semantics {
                            match semsrf {
                                geom_static::SemanticSurface::RoofSurface => {
                                    for ring in surface.boundaries {
                                        for vtx in ring {
                                            containter.push(vtx);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                geom_static::Geometry::CompositeSurface { .. } => {}
                geom_static::Geometry::Solid { boundaries, .. } => {
                    for shell in boundaries {
                        for surface in shell {
                            if let Some(semsrf) = surface.semantics {
                                match semsrf {
                                    geom_static::SemanticSurface::RoofSurface => {
                                        for ring in surface.boundaries {
                                            for vtx in ring {
                                                containter.push(vtx);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                geom_static::Geometry::MultiSolid { .. } => {}
                geom_static::Geometry::CompositeSolid { .. } => {}
            }
        }
    }
    println!(
        "In total there are {} points in the citymodel",
        containter.len()
    )*/
}

/// Creates an indexed boundary representation (as in CityJSON) from the dereferenced,
/// Simple Feature-like boundary.
fn index_boundaries(
    geometry: &Geometry,
    vtx_lookup: &mut HashMap<String, usize>,
    vtx_idx: &mut usize,
) -> Option<serde_json::Result<Value>> {
    match geometry {
        Geometry::MultiSurface { boundaries, .. } => {
            // indexed multisurface
            let mut imsurface = Vec::new();
            for (srfi, surface) in boundaries.iter().enumerate() {
                let mut isurface = Vec::new();
                for ring in surface {
                    let mut iring: Vec<usize> = Vec::new();
                    for vtx in ring {
                        let coord_str: String =
                            format!("{:.3} {:.3} {:.3}", vtx[0], vtx[1], vtx[2]);
                        match vtx_lookup.get(&coord_str) {
                            Some(existing_idx) => iring.push(existing_idx.clone()),
                            None => {
                                vtx_lookup.insert(coord_str, vtx_idx.clone());
                                iring.push(vtx_idx.clone());
                                *vtx_idx += 1;
                            }
                        }
                    }
                    isurface.push(iring);
                }
                imsurface.push(isurface);
            }
            // serde_json::to_string(&imsurface)
            Some(Ok(Value::from(imsurface)))
        }
        Geometry::Solid { boundaries, .. } => {
            let mut isolid = Vec::new();
            for shell in boundaries {
                let mut ishell = Vec::new();
                for surface in shell {
                    let mut isurface = Vec::new();
                    for ring in surface {
                        let mut iring: Vec<usize> = Vec::new();
                        for vtx in ring {
                            let coord_str: String =
                                format!("{:.3} {:.3} {:.3}", vtx[0], vtx[1], vtx[2]);
                            match vtx_lookup.get(&coord_str) {
                                Some(existing_idx) => iring.push(existing_idx.clone()),
                                None => {
                                    vtx_lookup.insert(coord_str, vtx_idx.clone());
                                    iring.push(vtx_idx.clone());
                                    *vtx_idx += 1;
                                }
                            }
                        }
                        isurface.push(iring);
                    }
                    ishell.push(isurface);
                }
                isolid.push(ishell);
            }
            // serde_json::to_string(&isolid)
            Some(Ok(Value::from(isolid)))
        }
        _ => None,
    }
}

/// Creates a CityJSON Semantic Object from a dereferenced Geometry
fn index_semantics(geometry: &Geometry) -> Option<serde_json::Result<serde_json::Value>> {
    match geometry {
        Geometry::MultiSurface {
            semantics_values, ..
        } => {
            if let Some(ref semval) = semantics_values {
                let mut surfaces: Vec<serde_json::Value> = Vec::new();
                let mut values: Vec<Option<usize>> = Vec::new();
                let mut sem_idx: HashMap<String, usize> = HashMap::new();
                let mut sem_idx_ctr: usize = 0;
                for (srfi, semop) in semval.iter().enumerate() {
                    if let Some(sem) = semop {
                        let semstr = format!("{:?}", sem);
                        if let Some(_sidx) = sem_idx.get(&semstr) {
                            values.push(Some(*_sidx));
                        } else {
                            sem_idx.insert(semstr.clone(), sem_idx_ctr);
                            values.push(Some(sem_idx_ctr));
                            surfaces.push(json!({
                                "type": semstr.clone()
                            }));
                            sem_idx_ctr += 1;
                        }
                    } else {
                        values.push(None);
                    }
                }
                let a = json!({
                    "surfaces": surfaces,
                    "values": values
                });
                Some(Ok(a))
            } else {
                None
            }
        }
        Geometry::Solid {
            semantics_values, ..
        } => {
            if let Some(ref semval) = semantics_values {
                let mut surfaces: Vec<serde_json::Value> = Vec::new();
                let mut values: Vec<Vec<Option<usize>>> = Vec::new();
                let mut sem_idx: HashMap<String, usize> = HashMap::new();
                let mut sem_idx_ctr: usize = 0;
                for (shi, shell_vec) in semval.iter().enumerate() {
                    let mut surface_values: Vec<Option<usize>> = Vec::new();
                    for (srfi, semop) in shell_vec.iter().enumerate() {
                        if let Some(sem) = semop {
                            let semstr = format!("{:?}", sem);
                            if let Some(_sidx) = sem_idx.get(&semstr) {
                                surface_values.push(Some(*_sidx));
                            } else {
                                sem_idx.insert(semstr.clone(), sem_idx_ctr);
                                surface_values.push(Some(sem_idx_ctr));
                                surfaces.push(json!({
                                    "type": semstr.clone()
                                }));
                                sem_idx_ctr += 1;
                            }
                        } else {
                            surface_values.push(None);
                        }
                    }
                    values.push(surface_values);
                }
                let a = json!({
                    "surfaces": surfaces,
                    "values": values
                });
                Some(Ok(a))
            } else {
                None
            }
        }
        // Not implemented
        _ => None,
    }
}

pub fn deref_serialize(path_in: PathBuf) {
    let cm = parse_dereferece(path_in);

    let mut vtx_lookup: HashMap<String, usize> = HashMap::new();
    let mut vtx_idx: usize = 0;
    let mut vertices: Vertices = Vec::new();

    let mut cityobjects_val: HashMap<String, Value> = HashMap::new();
    for (coid, co) in cm.cityobjects.iter() {
        let mut cityobject_val: Value = json!({
            "type": co.cotype.clone()
        });
        if co.cotype == "BuildingPart" {
            cityobject_val["parents"] = json!([]);
        }
        if let Some(ref geometry) = co.geometry {
            let mut geometry_val: Vec<Value> = Vec::new();
            for geometry_variant in geometry.iter() {
                let boundary = index_boundaries(geometry_variant, &mut vtx_lookup, &mut vtx_idx);
                let semantics = index_semantics(geometry_variant);
                let geomlod: String;
                let geomtype: String;
                match geometry_variant {
                    Geometry::MultiSurface { lod, .. } => {
                        geomtype = String::from("MultiSurface");
                        if let Some(lod) = lod {
                            match lod {
                                LoD::LoD0 => geomlod = String::from("0"),
                                LoD::LoD1 => geomlod = String::from("1"),
                                LoD::LoD1_2 => geomlod = String::from("1.2"),
                                LoD::LoD1_3 => geomlod = String::from("1.3"),
                                LoD::LoD2_2 => geomlod = String::from("2.2"),
                                _ => todo!(),
                            }
                        } else {
                            geomlod = String::from("");
                        }
                    }
                    Geometry::Solid { lod, .. } => {
                        geomtype = String::from("Solid");
                        if let Some(lod) = lod {
                            match lod {
                                LoD::LoD0 => geomlod = String::from("0"),
                                LoD::LoD1 => geomlod = String::from("1"),
                                LoD::LoD1_2 => geomlod = String::from("1.2"),
                                LoD::LoD1_3 => geomlod = String::from("1.3"),
                                LoD::LoD2_2 => geomlod = String::from("2.2"),
                                _ => todo!(),
                            }
                        } else {
                            geomlod = String::from("");
                        }
                    }
                    _ => {
                        geomtype = String::from("Not implemented");
                        geomlod = String::from("Unknown");
                    }
                };
                let mut g = json!({
                    "type": geomtype,
                    "lod": geomlod,
                });
                if let Some(boundary) = boundary {
                    g["boundaries"] = boundary.unwrap_or(json!([]));
                }
                if let Some(semantics) = semantics {
                    g["semantics"] = semantics.unwrap_or(json!({}));
                }
                geometry_val.push(g);
            }
            cityobject_val["geometry"] = Value::from(geometry_val);
        }
        cityobjects_val.insert(coid.clone(), cityobject_val);
    }

    for vtx in vtx_lookup.keys() {
        let mut v: [f32; 3] = [0.0, 0.0, 0.0];
        for (i, c) in vtx.split_whitespace().enumerate() {
            v[i] = c.parse::<f32>().unwrap();
        }
        vertices.push(v);
    }

    let citymodel_val = json!({
        "type": cm.cmtype.clone(),
        "version": cm.version.clone(),
        "CityObjects": cityobjects_val,
        "transform": {
            "scale": [1.0, 1.0, 1.0],
            "translate": [0.0, 0.0, 0.0]
        },
        "vertices": vertices,
    });

    let mut file_out =
        File::create("outfile.city.json").expect("Couldn't create file outfile.city.json");
    let res = serde_json::to_writer(&file_out, &citymodel_val);
}

pub fn deref_create(path_in: PathBuf) {
    /*    let mut vtx_lookup: HashMap<String, usize> = HashMap::new();
        let mut vtx_idx: usize = 0;
        let mut vertices: geom_static::Vertices = Vec::new();

        let g = geom_static::Geometry::Solid {
            lod: "1.2".to_string(),
            boundaries: vec![vec![geom_static::Surface {
                #[rustfmt::skip]
                boundaries: vec![vec![
                    [557299.0, 477667.0, 10693.0], [55799.0, 477667.0, 10693.0], [57299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 1063.0],[557299.0, 47767.0, 10693.0], [55799.0, 477667.0, 10693.0], [55799.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 1693.0], [557299.0, 477667.0, 1069.0],[557299.0, 47767.0, 10693.0], [57299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0]
                ]],
                semantics: Some(geom_static::SemanticSurface::RoofSurface),
                material: Some(geom_static::Material {
                    name: "SomeMaterial".to_string(),
                    ambient_intensity: Some(3.0),
                    diffuse_color: None,
                    emissive_color: None,
                    specular_color: None,
                    shininess: None,
                    transparency: None,
                    is_smooth: Some(true),
                }),
                texture: None,
            }]],
        };
        let res = index_boundaries(&g, &mut vtx_lookup, &mut vtx_idx);
        for vtx in vtx_lookup.keys() {
            let mut v: [f64; 3] = [0.0, 0.0, 0.0];
            for (i, c) in vtx.split_whitespace().enumerate() {
                v[i] = c.parse::<f64>().unwrap();
            }
            vertices.push(v);
        }
        let co = geom_static::CityObjectSer {
            cotype: "Building".to_string(),
            geometry: vec![res.unwrap()],
        };
        let mut cos: HashMap<String, geom_static::CityObjectSer> = HashMap::new();
        cos.insert("id-1".to_string(), co);
        let mut cityjson = json!({
          "type": "CityJSON",
          "version": "1.1",
          "CityObjects": cos,
          "vertices": vertices
        });

        println!("{}", cityjson.to_string());
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> PathBuf {
        Path::new("../data/3dbag_v210908_fd2cee53_5786_bench.city.json")
            .canonicalize()
            .expect("Could not find the INPUT file.")
    }

    #[test]
    fn test_deref_deserialize() {
        let path_in = get_data();
        deref_deserialize(path_in)
    }

    /*    #[test]
    fn test_deref_geometry() {
        let path_in = get_data();
        deref_geometry(path_in)
    }*/
}
