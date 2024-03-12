//! Estimate the stack and heap size of serde_citjson's structs that are holding value. This
//! module only used for performance optimization during the development of the library. The data
//! size estimation has a significant runtime overhead, so don't enable the corresponding "datasize"
//! feature unless you need it.
use crate::v1_1::*;
use datasize::{data_size, DataSize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::Write;

/// Stores the size of [CityModel] and its members. The members `CityModelDataSize` hold the size in
/// bytes of their corresponding [CityModel] member.
#[derive(Debug, Default)]
pub(crate) struct CityModelDataSize {
    id: usize,
    type_cm: usize,
    version: usize,
    transform: usize,
    cityobjects: CityObjectsDataSize,
    vertices: usize,
    metadata: usize,
    appearance: usize,
    geometry_templates: usize,
    extra: usize,
    extensions: usize,
}

impl CityModelDataSize {
    /// Calculate the size of a [CityModel].
    pub(crate) fn compute_from(cm: &CityModel) -> Self {
        let mut cm_size = CityModelDataSize {
            ..Default::default()
        };

        cm_size.id = total_heap_stack_size(&cm.id);
        cm_size.type_cm = total_heap_stack_size(&cm.type_cm);
        cm_size.version = total_heap_stack_size(&cm.version);
        cm_size.transform = total_heap_stack_size(&cm.transform);
        cm_size.vertices = total_heap_stack_size(&cm.vertices);
        cm_size.metadata = total_heap_stack_size(&cm.metadata);
        cm_size.appearance = total_heap_stack_size(&cm.appearance);
        cm_size.geometry_templates = total_heap_stack_size(&cm.geometry_templates);
        cm_size.extra = total_heap_stack_size(&cm.extra.as_ref());
        cm_size.extensions = total_heap_stack_size(&cm.extensions);

        let mut co_size = CityObjectsDataSize {
            ..Default::default()
        };
        let mut geometries_size: HashMap<LoD, GeometryDataSize> = HashMap::new();
        for (coid, co) in cm.cityobjects.iter() {
            co_size.count += 1;
            co_size.total_coid += total_heap_stack_size(coid);
            for geom in co.geometry.iter() {
                co_size.count_geometry += 1;
                co_size.total_geometry += total_heap_stack_size(geom);
                match geom {
                    Geometry::MultiSurface {
                        lod,
                        boundaries,
                        semantics,
                        texture,
                        material,
                    } => {
                        if geometries_size.contains_key(lod) {
                            let mut geomsize = geometries_size.get_mut(lod).unwrap();
                            geomsize.count += 1;
                            geomsize.total += total_heap_stack_size(geom);
                            geomsize.add_geometry(geom);
                        } else {
                            geometries_size.insert(
                                *lod,
                                GeometryDataSize {
                                    lod: *lod,
                                    count: 1,
                                    total: total_heap_stack_size(geom),
                                    boundaries: total_heap_stack_size(boundaries),
                                    semantics: total_heap_stack_size(semantics),
                                    texture: total_heap_stack_size(texture),
                                    material: total_heap_stack_size(material),
                                },
                            );
                        }
                    }
                    Geometry::Solid {
                        lod,
                        boundaries,
                        semantics,
                        texture,
                        material,
                    } => {
                        if geometries_size.contains_key(lod) {
                            let mut geomsize = geometries_size.get_mut(lod).unwrap();
                            geomsize.count += 1;
                            geomsize.total += total_heap_stack_size(geom);
                            geomsize.add_geometry(geom);
                        } else {
                            geometries_size.insert(
                                *lod,
                                GeometryDataSize {
                                    lod: *lod,
                                    count: 1,
                                    total: total_heap_stack_size(geom),
                                    boundaries: total_heap_stack_size(boundaries),
                                    semantics: total_heap_stack_size(semantics),
                                    texture: total_heap_stack_size(texture),
                                    material: total_heap_stack_size(material),
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
            if let Some(ref attributes) = co.attributes {
                co_size.count_attributes += attributes.len();
            }
            co_size.total_attributes +=
                sizeof_attributes_option(&co.attributes) + std::mem::size_of_val(&co.attributes);
            if co.geographical_extent.is_some() {
                co_size.count_geographical_extent += 1;
            }
            co_size.total_geographical_extent += total_heap_stack_size(&co.geographical_extent);
            if let Some(ref children) = co.children {
                co_size.count_children += children.len();
            }
            co_size.total_children_id += total_heap_stack_size(&co.children);
            if let Some(ref parents) = co.parents {
                co_size.count_parents += parents.len();
            }
            co_size.total_parents_id += total_heap_stack_size(&co.parents);
        }
        co_size.geometries = geometries_size.values().cloned().collect();
        cm_size.cityobjects = co_size;
        cm_size
    }
}

impl Display for CityModelDataSize {
    /// Prints a hierarchy of members, including the amount of heap memory used by them.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

/// Stores the data size of all CityObjects.
#[derive(Debug, Default)]
pub(crate) struct CityObjectsDataSize {
    /// CityObject count
    count: usize,
    /// Size of all CityObject IDs
    total_coid: usize,
    /// Count of all geometries
    count_geometry: usize,
    /// Size of all geometries of all CityObjects
    total_geometry: usize,
    /// Vec of Geometry sizes, one per Geometry of the CityObjects. For instance if the CityObjects contain four LoDs, this Vec has four GeometryDataSize.
    geometries: Vec<GeometryDataSize>,
    /// Count of all attributes in all CityObjects
    count_attributes: usize,
    /// Size of all attributes
    total_attributes: usize,
    /// Count of all geographical_extent in all CityObjects
    count_geographical_extent: usize,
    /// Size of all geographical_extent
    total_geographical_extent: usize,
    /// Count of all children IDs in all CityObjects
    count_children: usize,
    /// Size of all children IDs
    total_children_id: usize,
    /// Count of all parents IDs in all CityObjects
    count_parents: usize,
    /// Size of all parents IDs
    total_parents_id: usize,
}

/// Stores the data size of one geometric representation, e.g. all Geometry objects with the same LoD.
#[derive(Debug, Clone)]
pub(crate) struct GeometryDataSize {
    /// Count of geometries of this LoD
    count: usize,
    /// Size of all geometries of this LoD
    total: usize,
    lod: LoD,
    /// Size of all boundaries of this LoD
    boundaries: usize,
    /// Size of all semantics of this LoD
    semantics: usize,
    /// Size of all materials of this LoD
    material: usize,
    /// Size of all textures of this LoD
    texture: usize,
}

impl Default for GeometryDataSize {
    fn default() -> Self {
        Self {
            count: 0,
            total: 0,
            lod: LoD::LoD0,
            boundaries: 0,
            semantics: 0,
            material: 0,
            texture: 0,
        }
    }
}

impl GeometryDataSize {
    /// Compute the size of a Geometry and add the values.
    pub(crate) fn add_geometry(&mut self, geom: &Geometry) {
        match &geom {
            Geometry::MultiSurface {
                boundaries,
                semantics,
                material,
                texture,
                ..
            } => {
                self.boundaries += total_heap_stack_size(boundaries);
                self.semantics += total_heap_stack_size(semantics);
                self.texture += total_heap_stack_size(texture);
                self.material += total_heap_stack_size(material);
            }
            Geometry::Solid {
                boundaries,
                semantics,
                material,
                texture,
                ..
            } => {
                self.boundaries += total_heap_stack_size(boundaries);
                self.semantics += total_heap_stack_size(semantics);
                self.texture += total_heap_stack_size(texture);
                self.material += total_heap_stack_size(material);
            }
            _ => {}
        }
    }
}

/// Calculate the total heap and stack size of a variable.
fn total_heap_stack_size<T: DataSize>(data: &T) -> usize {
    data_size(data) + std::mem::size_of_val(data)
}

/// Compute the heap size of the optional Attributes.
pub(crate) fn sizeof_attributes_option(a: &Option<Attributes>) -> usize {
    if let Some(ref attributes) = a {
        attributes
            .iter()
            .map(|(k, v)| {
                std::mem::size_of::<String>()
                    + k.capacity()
                    + sizeof_serde_value(v)
                    + std::mem::size_of::<usize>() * 3
            })
            .sum()
    } else {
        0
    }
}

/// Compute the heap size of a serde_json::Value.
///
/// From https://stackoverflow.com/a/76456111
pub(crate) fn sizeof_serde_value(v: &serde_json::Value) -> usize {
    std::mem::size_of::<serde_json::Value>()
        + match v {
            serde_json::Value::Null => 0,
            serde_json::Value::Bool(_) => 0,
            serde_json::Value::Number(_) => 0, // Incorrect if arbitrary_precision is enabled. oh well
            serde_json::Value::String(s) => s.capacity(),
            serde_json::Value::Array(a) => a.iter().map(sizeof_serde_value).sum(),
            serde_json::Value::Object(o) => o
                .iter()
                .map(|(k, v)| {
                    std::mem::size_of::<String>()
                        + k.capacity()
                        + sizeof_serde_value(v)
                        + std::mem::size_of::<usize>() * 3 // As a crude approximation, I pretend each map entry has 3 words of overhead
                })
                .sum(),
        }
}

/// Wrapper type over a serde_json::Value that DataSize can be implemented for the inner Value.
pub(crate) struct CityModelSerdeValue {
    #[data_size(with = sizeof_serde_value)]
    inner: serde_json::Value,
}

mod test {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn bag3d() {
        let dummy_complete = PathBuf::from("resources")
            .join("data")
            .join("downloaded")
            .join("10-356-724.city.json");
        let mut file = File::open(dummy_complete).unwrap();
        let mut cityjson_json = String::new();
        file.read_to_string(&mut cityjson_json).unwrap();
        let cm_serde_value = CityModelSerdeValue {
            inner: serde_json::from_str(&cityjson_json).unwrap(),
        };
        let cm: CityModel = serde_json::from_str(&cityjson_json).unwrap();
        let cm_size = CityModelDataSize::compute_from(&cm);

        println!("CityJSON string: {}", total_heap_stack_size(&cityjson_json));
        println!(
            "CityModel serde_json::Value : {}",
            total_heap_stack_size(&cm_serde_value)
        );
        println!("CityModel serde_cityjson: {}", total_heap_stack_size(&cm));
        println!("{}", &cm_size);
    }

    #[test]
    fn serde_value_string_size() {
        let val: serde_json::Value = serde_json::from_str(r#""abcd""#).unwrap();
        dbg!(sizeof_serde_value(&val));
        let val: serde_json::Value = serde_json::from_str(r#""abcdefgh""#).unwrap();
        dbg!(sizeof_serde_value(&val));
    }
}
