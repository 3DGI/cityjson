//! A software library for working with semantic 3D city models, based on the
//! [CityJSON](https://cityjson.org) data model.

pub mod errors;

use crate::errors::{Error, Result};

use serde::{Deserialize, Serialize};
use serde_json::de::{Deserializer, StrRead, StreamDeserializer};
use std::collections::{hash_map, HashMap};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, LineWriter, Read};
use std::path::Path;
use std::str::FromStr;

/// A struct that represents a city model, which is conceptually equivalent to a
/// [CityJSON object](https://www.cityjson.org/specs/1.1.2/#cityjson-object).
///
/// # Examples
///
/// Create new, empty city models.
///
/// ```rust
/// let cm = cjlib::CityModel::new();
/// let cm2 = cjlib::CityModel::default();
/// ```
///
/// Deserialize a `CityJSON` document into a `CityModel`.
///
/// ```
/// use std::str::FromStr;
/// let cityjson_str = r#"{
///        "type": "CityJSON",
///        "version": "1.1",
///        "transform": {
///            "scale": [1.0, 1.0, 1.0],
///            "translate": [0.0, 0.0, 0.0]
///        },
///        "CityObjects": {},
///        "vertices": []
///    }"#;
/// let cm: cjlib::errors::Result<cjlib::CityModel> = cjlib::CityModel::from_str(cityjson_str);
/// println!("CityModel::from_str {:?}", cm);
///
/// let cm: serde_json::Result<cjlib::CityModel> = serde_json::from_str(cityjson_str);
/// println!("serde_json::from_str {:?}", cm);
///
/// let cm: serde_json::Result<cjlib::CityModel> = serde_json::from_slice(cityjson_str.as_bytes());
/// println!("serde_json::from_slice {:?}", cm);
///
/// // &[u8] implements Read
/// let cm: serde_json::Result<cjlib::CityModel> = serde_json::from_reader(cityjson_str.as_bytes());
/// println!("serde_json::from_reader {:?}", cm);
/// ```
///
pub struct CityModel {
    version: CityJSONVersion,
    transform: Option<Transform>,
    cityobjects: CityObjects,
}

impl CityModel {
    pub fn new() -> Self {
        Self {
            version: CityJSONVersion::V1_1,
            transform: None,
            cityobjects: Default::default(),
        }
    }

    /// Read from a file, either a regular JSON file or [JSON Lines text](https://jsonlines.org/).
    /// The parsing strategy is based on the file extension. Uses [`io::BufReader`](std::io::BufReader)
    /// for reading.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(&file);
        return match path.as_ref().extension() {
            None => Err(Error::InvalidExtension(path.as_ref().to_path_buf())),
            Some(extension_os_str) => {
                // todo: Make sure that when reading jsonl file, the reader reads a whole line at
                //  once (until the \n). Otherwise use the default bufreader.
                if extension_os_str == SupportedFileExtension::JSON
                    || extension_os_str == SupportedFileExtension::CITYJSON
                {
                    let cm: CityModel = serde_json::from_reader(reader)?;
                    Ok(cm)
                } else if extension_os_str == SupportedFileExtension::JSONL {
                    Self::from_stream(reader)
                } else {
                    // let's try parsing as a regular CityJSON file
                    let cm: CityModel = serde_json::from_reader(reader)?;
                    Ok(cm)
                }
            }
        };
    }

    /// Create a CityModel from a stream of CityJSONFeatures, aggregating them into the CityModel's
    /// CityObjects. Assumes that the first item in the stream is a CityJSON.
    pub fn from_stream<R>(cursor: R) -> Result<Self>
    where
        R: BufRead,
    {
        let mut stream_iter = cursor.lines();

        let mut cm: CityModel;
        // Do return an error if we cannot process the first item of the stream into a CityModel,
        // because the subsequent steps depend on it.
        if let Some(res) = stream_iter.next() {
            let cityjson_str = res?;
            cm = CityModel::from_str(&cityjson_str)?;
        } else {
            return Err(Error::StreamingError(String::from("empty stream")));
        }

        for res in stream_iter {
            // Don't break if for some reason we cannot process one feature from the stream,
            // but do notify about it.
            if let Ok(cityjsonfeature_str) = res {
                let cf = CityFeature::from_str(&cityjsonfeature_str)?;
                cm.cityobjects.insert(cf.id, CityObject);
            } else {
                // todo: log error
            }
        }
        Ok(cm)
    }

    /// Convert the CityModel to a CityJSON document string.
    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(&ICityModel::from(self))?)
    }

    /// Write the CityModel to a CityJSON file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file_out = File::create(path.as_ref())?;
        return match path.as_ref().extension() {
            None => Err(Error::InvalidExtension(path.as_ref().to_path_buf())),
            Some(extension_os_str) => {
                if extension_os_str == SupportedFileExtension::JSON
                    || extension_os_str == SupportedFileExtension::CITYJSON
                {
                    Ok(serde_json::to_writer(&file_out, &ICityModel::from(self))?)
                } else if extension_os_str == SupportedFileExtension::JSONL {
                    let cityjson = self.to_features_cityjson().unwrap();
                    let cityjsonfeatures = self
                        .to_features()
                        .flat_map(|cityfeature| cityfeature.to_string());
                    let mut file = LineWriter::new(file_out);
                    // The file must contain at least the first CityJSON object.
                    writeln!(file, "{}", cityjson)?;
                    for cf in cityjsonfeatures {
                        if writeln!(file, "{}", cf).is_err() {
                            todo!() // log error message here and skip the feature
                        }
                    }
                    Ok(())
                } else {
                    Err(Error::UnsupportedExtension)
                }
            }
        };
    }

    /// Convert the CityModel to a CityJSON object string, for passing as the first item in a
    /// CityJSONFeature stream. The new CityJSON object has empty "CityObjects" and "vertices"
    /// members, because these are supposed to be passed in subsequent CityJSONFeatures.
    pub fn to_features_cityjson(&self) -> Result<String> {
        Ok(serde_json::to_string(&ICityModel {
            id: None,
            type_cm: CityModelType::CityJSON,
            version: Some(self.version),
            transform: Some(self.transform.unwrap_or_default()),
            cityobjects: ICityObjects::new(),
            vertices: IVertices::new(),
        })?)
    }

    pub fn to_features(&self) -> CityFeatureIterator {
        CityFeatureIterator {
            cityobjects_iter: self.cityobjects.iter(),
        }
    }

    pub fn version(&self) -> &CityJSONVersion {
        &self.version
    }

    fn set_version(&mut self, version: CityJSONVersion) {
        self.version = version;
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    pub fn set_transform(&mut self, transform: &Transform) {
        self.transform = Some(*transform);
    }
}

impl Default for CityModel {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityModel")
            .field("version", &self.version)
            .field("transform", &self.transform)
            .field("cityobjects", &self.cityobjects)
            .finish()
    }
}

impl fmt::Display for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(\n\tversion: {}\n\tnr. cityobjects: {})",
            &self.version,
            &self.cityobjects.len()
        )
    }
}

impl<'de> Deserialize<'de> for CityModel {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let icm = ICityModel::deserialize(deserializer)?;
        match icm.type_cm {
            CityModelType::CityJSONFeature => Err(serde::de::Error::custom(
                Error::ExpectedCityJSON(icm.type_cm),
            )),
            CityModelType::CityJSON => Ok(Self {
                version: icm.version.unwrap(),
                transform: icm.transform,
                cityobjects: Default::default(),
            }),
        }
    }
}

impl FromStr for CityModel {
    type Err = Error;

    /// Parse a CityJSON document from a string, by calling [`serde_json::from_str`](serde_json::from_str).
    fn from_str(cityjson: &str) -> Result<Self> {
        let icm: ICityModel = serde_json::from_str(cityjson)?;
        match icm.type_cm {
            CityModelType::CityJSON => Ok(Self {
                version: icm.version.unwrap(),
                transform: icm.transform,
                cityobjects: Default::default(),
            }),
            CityModelType::CityJSONFeature => Err(Error::ExpectedCityJSON(icm.type_cm)),
        }
    }
}

pub struct CityFeatureIterator<'cityobjects> {
    // We borrow the CityObjects from the CityModel for this struct, because the CityObjects values
    // are cloned into the CityFeature-s.
    cityobjects_iter: hash_map::Iter<'cityobjects, String, CityObject>,
}

impl<'cityobjects> Iterator for CityFeatureIterator<'cityobjects> {
    type Item = CityFeature;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((coid, _)) = self.cityobjects_iter.next() {
            Some(CityFeature::new(coid.to_string()))
        } else {
            None
        }
    }
}

pub struct CityFeature {
    id: String,
}

impl CityFeature {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    /// Convert a CityFeature to a CityJSONFeature object string.
    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(&ICityModel::from(self))?)
    }
}

impl Default for CityFeature {
    fn default() -> Self {
        Self::new(String::from(""))
    }
}

impl fmt::Debug for CityFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityFeature").field("id", &self.id).finish()
    }
}

impl<'de> Deserialize<'de> for CityFeature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let icm = ICityModel::deserialize(deserializer)?;
        match icm.type_cm {
            CityModelType::CityJSON => Err(serde::de::Error::custom(
                Error::ExpectedCityJSONFeature(icm.type_cm),
            )),
            CityModelType::CityJSONFeature => Ok(Self {
                id: icm.id.unwrap(),
            }),
        }
    }
}

impl fmt::Display for CityFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CityFeature (\n\tid: {}\n)", &self.id)
    }
}

impl FromStr for CityFeature {
    type Err = Error;

    /// Deserialize a string of CityJSON text.
    fn from_str(cityjson: &str) -> Result<Self> {
        let icm: ICityModel = serde_json::from_str(cityjson)?;
        match icm.type_cm {
            CityModelType::CityJSON => Err(Error::ExpectedCityJSONFeature(icm.type_cm)),
            CityModelType::CityJSONFeature => Ok(Self {
                id: icm.id.unwrap(),
            }),
        }
    }
}

/// Deserialize a stream of CityJSONFeatures.
///
/// Deserializes a stream of `CityJSONFeature` into `Result<CityFeature>`. It returns an error if
/// it fails to deserialize an item into a `CityFeature`. If the error is a type error, and the
/// next thing in the stream was at least valid JSON, then deserialize the item into a
/// dynamically-typed [serde_json::Value](`serde_json::Value`), return the Value with the error and
/// skip the item.
/// If the item is invalid JSON, it returns an error and the iteration stops.
///
/// Note:
///     You may also the default serde_json::StreamDeserializer, however that will stop on the first
///     error. For instance if the `feature_stream` contains a malformed item, or an item that is
///     not `CityJSONFeature`, then
///     `Deserializer::from_str(&feature_stream).into_iter::<CityFeature>();` will stop after
///     returning any kind of error.
///     
/// Example
/// ```
/// use cjlib::CityFeatureStreamDeserializer;
///
/// let feature_sequence = r#"{"type":"CityJSONFeature","id":"id-1","CityObjects":{},"vertices":[]}
///         {"type":"CityJSON","CityObjects":{},"vertices":[]}
///         {"type":"CityJSONFeature","id":"id-3","CityObjects":{},"vertices":[]}
///         {"type": invalid json"#;
///
/// for result in CityFeatureStreamDeserializer::new(&feature_sequence) {
///     if let Ok(cityfeature) = result {
///         println!("cityfeature: {:#?}", cityfeature);
///     } else {
///         println!("not cityfeature: {:#?}", result);
///     }   
/// }
/// ```
///
/// Credit to ... from https://users.rust-lang.org/t/step-past-errors-in-serde-json-streamdeserializer/84228/8?u=balazsdukai
pub struct CityFeatureStreamDeserializer<'de> {
    json: &'de str,
    stream: StreamDeserializer<'de, StrRead<'de>, CityFeature>,
    last_ok_pos: usize,
}

impl<'de> CityFeatureStreamDeserializer<'de>
where
    CityFeature: Deserialize<'de>,
{
    pub fn new(json: &'de str) -> Self {
        let stream = Deserializer::from_str(json).into_iter();
        let last_ok_pos = 0;

        CityFeatureStreamDeserializer {
            json,
            stream,
            last_ok_pos,
        }
    }
}

impl<'de> Iterator for CityFeatureStreamDeserializer<'de>
where
    CityFeature: Deserialize<'de>,
{
    type Item = Result<CityFeature>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.next()? {
            Ok(value) => {
                self.last_ok_pos = self.stream.byte_offset();
                Some(Ok(value))
            }
            Err(error) => {
                // If an error happened, check whether it's a type error, i.e.
                // whether the next thing in the stream was at least valid JSON.
                // If so, return it as a dynamically-typed `Value` and skip it.
                let err_json = &self.json[self.last_ok_pos..];
                let mut err_stream =
                    Deserializer::from_str(err_json).into_iter::<serde_json::Value>();
                let value = err_stream.next()?.ok();
                let next_pos = if value.is_some() {
                    self.last_ok_pos + err_stream.byte_offset()
                } else {
                    self.json.len() // when JSON has a syntax error, prevent infinite stream of errors
                };
                self.json = &self.json[next_pos..];
                self.stream = Deserializer::from_str(self.json).into_iter();
                self.last_ok_pos = 0;
                Some(Err(Error::MalformedCityJSON(error, value)))
            }
        }
    }
}

/// A register of what file extensions are supported.
/// It allows comparison for equality with an [`std::ffi::OsStr`](std::ffi::OsStr), which we get when working with
/// [`std::path::Path`](std::path::Path)s.
/// There are two concepts that are important in this implementation,
/// [associated constants](https://doc.rust-lang.org/reference/items/associated-items.html#associated-constants)
/// and the [non_exhaustive attribute](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
/// which indicates that this type may have more fields or variants added in the future.
///
///
/// Alternative implementations that I considered:
///
/// The array of strings. However, I need to distinguish between 'json' and 'jsonl' and probably
/// other extensions in the future too. Thus, a simple containment test is not enough.
/// ```
/// static SUPPORTED_FILE_EXTENSION: [&str; 3] = [ "json", "cityjson", "jsonl" ];
/// let extension_of_input_file = "json";
/// let does_contain = SUPPORTED_FILE_EXTENSION.contains(&extension_of_input_file);
/// ```
///
/// An enum. While it achieves the same purpose as the struct implementation, it is much more
/// verbose.
/// ```
/// use std::ffi::OsStr;
///
/// #[derive(Debug, Copy, Clone)]
/// enum SupportedFileExtension {
///     Json,
///     CityJson,
///     Jsonl,
/// }
///
/// impl From<&SupportedFileExtension> for &str {
///     fn from(value: &SupportedFileExtension) -> Self {
///         match value {
///             SupportedFileExtension::Json => "json",
///             SupportedFileExtension::CityJson => "cityjson",
///             SupportedFileExtension::Jsonl => "jsonl",
///         }
///     }
/// }
///
/// impl SupportedFileExtension {
///     fn print_all() -> String {
///         format!("{:?}, {:?}, {:?}", Self::Json, Self::CityJson, Self::Jsonl).to_lowercase()
///     }
/// }
///
/// impl PartialEq<&OsStr> for SupportedFileExtension {
///     fn eq(&self, other: &&OsStr) -> bool {
///         let a: &str = self.into();
///         *other == a
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(Debug)]
struct SupportedFileExtension;

impl SupportedFileExtension {
    pub const JSON: &'static str = "json";
    pub const CITYJSON: &'static str = "cityjson";
    pub const JSONL: &'static str = "jsonl";
}

impl fmt::Display for SupportedFileExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}",
            Self::JSON,
            Self::CITYJSON,
            Self::JSONL
        )
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
#[serde(tag = "version", try_from = "String", into = "String")]
pub enum CityJSONVersion {
    V1_1,
}

impl fmt::Display for CityJSONVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CityJSONVersion::V1_1 => {
                write!(f, "1.1")
            }
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        CityJSONVersion::_from_str(value)
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        CityJSONVersion::_from_str(value.as_ref())
    }
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> Result<CityJSONVersion> {
        match value {
            "1.1" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(CityJSONVersion::V1_1),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "1.1, 1.1.1, 1.1.2, 1.1.3".to_string(),
            )),
        }
    }
}

/// This implementation is only used for serializing the CityJSON version, because serde cannot
/// serialize from 'try_into' (which is provided by the 'try_from' implementations).
/// So we need this Into, even though [std says that one should avoid implementing Into](https://doc.rust-lang.org/std/convert/trait.Into.html).
impl Into<String> for CityJSONVersion {
    fn into(self) -> String {
        match self {
            CityJSONVersion::V1_1 => String::from("1.1"),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CityModelType {
    CityJSON,
    CityJSONFeature,
}

impl fmt::Display for CityModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CityModelType::CityJSON => {
                write!(f, "CityJSON")
            }
            CityModelType::CityJSONFeature => {
                write!(f, "CityJSONFeature")
            }
        }
    }
}

// todo: "Transform members need to be private, so that we can change the internals if needed, eg f32 -> f64 or so"
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Transform {
    pub scale: [f32; 3],
    pub translate: [f32; 3],
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(scale: [{}, {}, {}], translate:[{}, {}, {}])",
            self.scale[0],
            self.scale[1],
            self.scale[2],
            self.translate[0],
            self.translate[1],
            self.translate[2]
        )
    }
}

impl Transform {
    pub fn new() -> Self {
        // NOTE: consider ::new(scale: &[f32;3], translate: &[f32;3])
        // NOTE: maybe use scale: [0.001, 0.001, 0.001] as default
        Self {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

// NOTE: Not sure HashMap is the best choice in terms of performance.
// I've seen BTreeMap used in many places.
// Would there be any advantage of a custom type for the CityObject Id, instead of String?
type CityObjects = HashMap<String, CityObject>;

// NOTE: I think a CityObject should know its own Id. That would make it much simpler to send
// around CityObjects, eg. when converting to CityFeatures.
#[derive(Default, Debug)]
struct CityObject;

/// Metadata for a city model.
///
/// There is only structural validation for the metadata items, the metadata values are not
/// validated at the moment. For instance, a contact website must be a string, but it is not
/// checked whether the string is a valid URL or not.
///
/// See also the
/// [CityJSON Metadata](https://www.cityjson.org/specs/1.1.2/#metadata).
///
/// # Examples
///
/// A metadata object can be built by invoking the `Metadata::builder`, chaining calls
/// to methods to set each metadata member, and finally calling `.build`.
/// ```
/// let metadata = cjlib::Metadata::builder()
///     .organization("3DGI")
///     .role(cjlib::ContactRole::Author)
///     .contact_name("Balázs Dukai")
///     .email_address("my@email.com")
///     .geographical_extent([1.0, 1.0, 1.0, 1.0, 1.0, 1.0])
///     .identifier("123-456-789")
///     .build();
/// ```
///
/// Alternatively, you can also create and empty metadata object and set the various members
/// individually. Note that the `Metadata::builder` and `.build` are not needed in this case.
///
/// ```
/// let mut metadata = cjlib::Metadata::new();
/// metadata.set_organization("3DGI");
/// metadata.set_role(cjlib::ContactRole::Author);
/// metadata.set_contact_name("Balázs Dukai");
/// metadata.set_email_address("my@email.com");
/// metadata.set_geographical_extent([1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
/// metadata.set_identifier("123-456-789");
///
/// println!("{:?}", metadata.identifier());
/// ```
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Metadata {
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier>,
    point_of_contact: Option<Contact>,
    reference_date: Option<Date>,
    reference_system: Option<CRS>,
    title: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata::default()
    }

    pub fn builder() -> MetadataBuilder {
        MetadataBuilder::new()
    }

    pub fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }
    pub fn set_geographical_extent(&mut self, bbox: BBox) {
        self.geographical_extent = Some(bbox);
    }

    pub fn identifier(&self) -> Option<&CityModelIdentifier> {
        self.identifier.as_ref()
    }
    pub fn set_identifier<S: AsRef<str>>(&mut self, identifier: S) {
        self.identifier = Some(identifier.as_ref().to_owned());
    }

    pub fn reference_date(&self) -> Option<&Date> {
        self.reference_date.as_ref()
    }
    pub fn set_reference_date<S: AsRef<str>>(&mut self, date: S) {
        self.reference_date = Some(date.as_ref().to_owned());
    }

    pub fn reference_system(&self) -> Option<&CRS> {
        self.reference_system.as_ref()
    }
    pub fn set_reference_system<S: AsRef<str>>(&mut self, crs: S) {
        self.reference_system = Some(crs.as_ref().to_owned());
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }
    pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
        self.title = Some(title.as_ref().to_owned());
    }

    fn point_of_contact(&self) -> Option<&Contact> {
        self.point_of_contact.as_ref()
    }

    pub fn contact_name(&self) -> Option<&String> {
        if let Some(ref poc) = self.point_of_contact {
            Some(&poc.contact_name)
        } else {
            None
        }
    }
    pub fn set_contact_name<S: AsRef<str>>(&mut self, name: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_name = name.as_ref().to_owned()
        } else {
            self.point_of_contact = Some(Contact {
                contact_name: name.as_ref().to_owned(),
                ..Default::default()
            })
        }
    }

    pub fn email_address(&self) -> Option<&String> {
        if let Some(ref poc) = self.point_of_contact {
            Some(&poc.email_address)
        } else {
            None
        }
    }

    pub fn set_email_address<S: AsRef<str>>(&mut self, email: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.email_address = email.as_ref().to_owned()
        } else {
            self.point_of_contact = Some(Contact {
                email_address: email.as_ref().to_owned(),
                ..Default::default()
            })
        }
    }

    pub fn role(&self) -> Option<&ContactRole> {
        if let Some(ref poc) = self.point_of_contact {
            poc.role.as_ref()
        } else {
            None
        }
    }

    pub fn set_role(&mut self, role: ContactRole) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.role = Some(role);
        } else {
            self.point_of_contact = Some(Contact {
                role: Some(role),
                ..Default::default()
            })
        }
    }

    pub fn website(&self) -> Option<&String> {
        if let Some(ref poc) = self.point_of_contact {
            poc.website.as_ref()
        } else {
            None
        }
    }

    pub fn set_website<S: AsRef<str>>(&mut self, website: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.website = Some(website.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                website: Some(website.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn contact_type(&self) -> Option<&ContactType> {
        if let Some(ref poc) = self.point_of_contact {
            poc.contact_type.as_ref()
        } else {
            None
        }
    }

    pub fn set_contact_type(&mut self, contact_type: ContactType) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_type = Some(contact_type);
        } else {
            self.point_of_contact = Some(Contact {
                contact_type: Some(contact_type),
                ..Default::default()
            })
        }
    }

    pub fn address(&self) -> Option<&String> {
        if let Some(ref poc) = self.point_of_contact {
            poc.address.as_ref()
        } else {
            None
        }
    }

    pub fn set_address<S: AsRef<str>>(&mut self, address: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.address = Some(address.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                address: Some(address.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn phone(&self) -> Option<&String> {
        if let Some(ref poc) = self.point_of_contact {
            poc.phone.as_ref()
        } else {
            None
        }
    }

    pub fn set_phone<S: AsRef<str>>(&mut self, phone: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.phone = Some(phone.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                phone: Some(phone.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn organization(&self) -> Option<&String> {
        if let Some(ref poc) = self.point_of_contact {
            poc.organization.as_ref()
        } else {
            None
        }
    }

    pub fn set_organization<S: AsRef<str>>(&mut self, organization: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.organization = Some(organization.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                organization: Some(organization.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }
}

/// Builds a `Metadata` object with keyword arguments.
///
/// The `MetadataBuilder` is basically for having keyword arguments for the `Metadata`, and maybe
/// some input validation. There many metadata members, and it is easy to confuse them if only
/// using the positional arguments that Rust allows.
#[derive(Default, Debug)]
pub struct MetadataBuilder {
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier>,
    reference_date: Option<Date>,
    reference_system: Option<CRS>,
    title: Option<String>,
    // point_of_contact members
    contact_name: Option<String>,
    email_address: Option<String>,
    role: Option<ContactRole>,
    website: Option<String>,
    contact_type: Option<ContactType>,
    address: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
}

impl MetadataBuilder {
    fn new() -> Self {
        MetadataBuilder {
            geographical_extent: None,
            identifier: None,
            reference_date: None,
            reference_system: None,
            title: None,
            contact_name: None,
            email_address: None,
            role: None,
            website: None,
            contact_type: None,
            address: None,
            phone: None,
            organization: None,
        }
    }

    pub fn geographical_extent(mut self, bbox: BBox) -> Self {
        self.geographical_extent = Some(bbox);
        self
    }

    pub fn identifier<S: AsRef<str>>(mut self, identifier: S) -> Self {
        self.identifier = Some(identifier.as_ref().to_owned());
        self
    }

    pub fn reference_date<S: AsRef<str>>(mut self, date: S) -> Self {
        self.reference_date = Some(date.as_ref().to_owned());
        self
    }

    pub fn reference_system<S: AsRef<str>>(mut self, crs: S) -> Self {
        self.reference_system = Some(crs.as_ref().to_owned());
        self
    }

    pub fn title<S: AsRef<str>>(mut self, title: S) -> Self {
        self.title = Some(title.as_ref().to_owned());
        self
    }

    pub fn contact_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.contact_name = Some(name.as_ref().to_owned());
        self
    }

    pub fn email_address<S: AsRef<str>>(mut self, email: S) -> Self {
        self.email_address = Some(email.as_ref().to_owned());
        self
    }

    pub fn role(mut self, role: ContactRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn website<S: AsRef<str>>(mut self, website: S) -> Self {
        self.website = Some(website.as_ref().to_owned());
        self
    }

    pub fn contact_type(mut self, contact_type: ContactType) -> Self {
        self.contact_type = Some(contact_type);
        self
    }

    pub fn address<S: AsRef<str>>(mut self, address: S) -> Self {
        self.address = Some(address.as_ref().to_owned());
        self
    }

    pub fn phone<S: AsRef<str>>(mut self, phone: S) -> Self {
        self.phone = Some(phone.as_ref().to_owned());
        self
    }

    pub fn organization<S: AsRef<str>>(mut self, org: S) -> Self {
        self.organization = Some(org.as_ref().to_owned());
        self
    }

    pub fn build(self) -> Result<Metadata> {
        let mut poc: Option<Contact> = None;
        if (self.role.is_some()
            || self.website.is_some()
            || self.contact_type.is_some()
            || self.address.is_some()
            || self.phone.is_some()
            || self.organization.is_some())
            && (self.contact_name.is_none() || self.email_address.is_none())
        {
            return Err(Error::MetadataError("both contact_name and email_address must be set when setting values for the point_of_contact".to_string()));
        } else if self.contact_name.is_some() || self.email_address.is_some() {
            poc = Some(Contact {
                contact_name: self.contact_name.expect(
                    "contact_name must be set when setting values for the point_of_contact",
                ),
                email_address: self.email_address.expect(
                    "email_address must be set when setting values for the point_of_contact",
                ),
                role: self.role,
                website: self.website,
                contact_type: self.contact_type,
                address: self.address,
                phone: self.phone,
                organization: self.organization,
            });
        }
        Ok(Metadata {
            geographical_extent: self.geographical_extent,
            identifier: self.identifier,
            point_of_contact: poc,
            reference_date: self.reference_date,
            reference_system: self.reference_system,
            title: self.title,
        })
    }
}

/// Bounding Box.
///
/// An array of 6 values: `[minx, miny, minz, maxx, maxy, maxz]`.
/// See also the
/// [CityJSON geographicalExtent](https://www.cityjson.org/specs/1.1.2/#geographicalextent-bbox).
pub type BBox = [f32; 6];

/// An identifier for the dataset.
/// See also the
/// [CityJSON identifier](https://www.cityjson.org/specs/1.1.2/#identifier).
pub type CityModelIdentifier = String;

/// The point of contact for the city model.
/// See also the
/// [CityJSON pointOfContact](https://www.cityjson.org/specs/1.1.2/#pointofcontact).
#[derive(Clone, Default, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Contact {
    contact_name: String,
    email_address: String,
    role: Option<ContactRole>,
    website: Option<String>,
    contact_type: Option<ContactType>,
    address: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ContactRole {
    Author,
    CoAuthor,
    Collaborator,
    Contributor,
    Custodian,
    Distributor,
    Editor,
    Funder,
    Mediator,
    Originator,
    Owner,
    PointOfContact,
    PrincipalInvestigator,
    Processor,
    Publisher,
    ResourceProvider,
    RightsHolder,
    Sponsor,
    Stakeholder,
    User,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ContactType {
    Individual,
    Organization,
}

/// The date when the dataset was compiled.
///
/// The format is a `"full-date"` per the
/// [RFC 3339, Section 5.6](https://tools.ietf.org/html/rfc3339#section-5.6).
///
/// See also the
/// [CityJSON referenceDate](https://www.cityjson.org/specs/1.1.2/#referencedate).
///
/// # Examples
/// ```
/// let date: cjlib::Date = String::from("1977-02-28");
/// ```
pub type Date = String;

// Note: Could also have a CRS struct with named members but that's too much complication, because
// it brings a lot of implementation with it (Display, FromStr, Into<String>, ...), incl.
// validation. And the philosophy with the other Metadata members is that we accept almost any
// value, because too pedantic validation in cjlib might actually get in the way of building city
// models. And then it is better to push the validation down to specialized libraries, such as
// cjval.
// #[derive(Clone, Default, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
// pub struct CRS {
//     authority: String,
//     version: i8,
//     code: i16,
// }
//
// impl fmt::Display for CRS {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "http://www.opengis.net/def/crs/{authority}/{version}/{code}",
//             authority = self.authority,
//             version = self.version,
//             code = self.code
//         )
//     }
// }
/// The coordinate reference system (CRS) of the city model.
///
/// Must be formatted as a URL, according to the
/// [OGC Name Type Specification](https://docs.opengeospatial.org/pol/09-048r5.html#_production_rule_for_specification_element_names).
/// See also the
/// [CityJSON referenceSystem](https://www.cityjson.org/specs/1.1.2/#referencesystem-crs).
///
/// # Examples
/// ```
/// let crs: cjlib::CRS = String::from("https://www.opengis.net/def/crs/EPSG/0/7415");
/// ```
pub type CRS = String;

/// Indexed-structures.
/// Parsing a CityJSON document is (internally) a two-step process with cjlib.
/// In the first step, the document is deserialized into structures that are much alike the
/// CityJSON schema, and use indexed-vertices. These structures are private and they are
/// prefixed with an `I`, eg. `ICityModel`.
/// In the second step, the indexed-structures are transformed to a `CityModel`, through
/// de-referencing the vertices and other operations.

/// Indexed-CityModel, which is an intermediary struct that is directly deserialized from CityJSON
/// document and converted to a CityModel.
#[derive(Debug, Deserialize, Serialize)]
struct ICityModel {
    // NOTE: consider https://docs.rs/serde_with/latest/serde_with/index.html#skip_serializing_none
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "type")]
    type_cm: CityModelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<CityJSONVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transform: Option<Transform>,
    #[serde(skip_deserializing, rename = "CityObjects")]
    cityobjects: ICityObjects,
    vertices: IVertices,
}

impl From<&CityModel> for ICityModel {
    fn from(cm: &CityModel) -> Self {
        Self {
            id: None,
            type_cm: CityModelType::CityJSON,
            version: Some(cm.version),
            transform: Some(cm.transform.unwrap_or_default()),
            cityobjects: ICityObjects::new(),
            vertices: IVertices::new(),
        }
    }
}

impl From<&CityFeature> for ICityModel {
    fn from(cf: &CityFeature) -> Self {
        Self {
            id: Some(cf.id.clone()),
            type_cm: CityModelType::CityJSONFeature,
            version: None,
            transform: None,
            cityobjects: ICityObjects::new(),
            vertices: IVertices::new(),
        }
    }
}

type ICityObjects = HashMap<String, ICityObject>;

#[derive(Debug, Serialize)]
struct ICityObject;

/// Vertex coordinates, deserialized from a CityJSON document.
/// Uses i32, which is way too much, but i16 not enough, because a CityModel can easily have
/// transformed coordinates beyond +/-32767.
type IVertices = Vec<[i32; 3]>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Deserializer;
    use std::io::Cursor;
    use std::path::PathBuf;

    fn test_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("data")
    }

    fn test_output_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("output")
    }

    #[test]
    fn instantiate_citymodel() {
        let _cm = CityModel::new();
        let _cm2 = CityModel::default();
    }

    #[test]
    fn citymodel_from_str_minimal() {
        let cityjson_str = r#"{
            "type": "CityJSON",
            "version": "1.1",
            "transform": {
                "scale": [1.0, 1.0, 1.0],
                "translate": [0.0, 0.0, 0.0]
            },
            "CityObjects": {},
            "vertices": []
        }"#;
        let cm: crate::errors::Result<CityModel> = CityModel::from_str(cityjson_str);
        println!("CityModel::from_str {:?}", cm);

        let cm: serde_json::Result<CityModel> = serde_json::from_str(cityjson_str);
        println!("serde_json::from_str {:?}", cm);

        let cm: serde_json::Result<CityModel> = serde_json::from_slice(cityjson_str.as_bytes());
        println!("serde_json::from_slice {:?}", cm);

        // &[u8] implements Read
        let cm: serde_json::Result<CityModel> = serde_json::from_reader(cityjson_str.as_bytes());
        println!("serde_json::from_reader {:?}", cm);
    }

    #[test]
    fn citymodel_from_file() {
        let pb: PathBuf = test_data_dir().join("minimal_valid.city.json");
        let _ = CityModel::from_file(&pb);
        let ps: &str = pb.to_str().unwrap();
        let _ = CityModel::from_file(ps);
    }

    #[test]
    fn citymodel_to_string() {
        let tr = Transform {
            scale: [0.001, 0.001, 0.001],
            translate: [0.0, 0.0, 0.0],
        };
        let mut cm = CityModel::new();
        cm.set_transform(&tr);
        let cj = cm.to_string().unwrap();
        println!("CityJSON: {}", cj);
        println!("CityModel: {}", cm);
    }

    #[test]
    fn citymodel_to_file() {
        let pb: PathBuf = test_output_dir().join(".test_out.city.json");
        let _ = CityModel::new().to_file(pb);
    }

    #[test]
    fn debug_citymodel() {
        let cm = CityModel::new();
        println!("{:?}", cm);
    }

    #[test]
    fn display_citymodel() {
        let cm = CityModel::new();
        println!("{}", cm);
    }

    #[test]
    fn set_get_transform() {
        let mut cm = CityModel::default();
        println!("{:?}", cm);
        let t = Transform {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        };
        cm.set_transform(&t);
        println!("{:?}", cm);
    }

    #[test]
    fn cityjsonversion() {
        let vr = CityJSONVersion::try_from("1.1");
        assert_eq!(vr.unwrap(), CityJSONVersion::V1_1);
        let s: String = CityJSONVersion::V1_1.into();
        println!("CityJSONVersion.into(): {}", s);
        println!(
            "CityJSONVersion.to_string(): {}",
            CityJSONVersion::V1_1.to_string()
        );
        let v2 = CityJSONVersion::try_from("1.0");
        v2.expect_err("Unsupported CityJSON version.");
    }

    /// Can we deserialize a CityJSONFeature into an ICityModel?
    #[test]
    fn cityjsonfeature() {
        let cityjsonfeature_str = r#"{
            "type": "CityJSONFeature",
            "id": "id-1",
            "CityObjects": {},
            "vertices": []
        }"#;
        let _: ICityModel = serde_json::from_str(cityjsonfeature_str).unwrap();
    }

    #[test]
    fn features_from_stream() {
        let feature_sequence = r#"{"type":"CityJSON","version":"1.1","transform":{"scale":[0.1,0.1,0.1],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}
            {"type":"CityJSONFeature","id":"id-1","CityObjects":{},"vertices":[]}
            {"type":"CityJSONFeature","id":"id-2","CityObjects":{},"vertices":[]}"#;
        let stream = Cursor::new(feature_sequence);
        let cm = CityModel::from_stream(stream);
        println!("From stream: {:?}", cm);
    }

    #[test]
    fn features_streamdeserializer() {
        let feature_sequence = r#"{"type":"CityJSONFeature","id":"id-1","CityObjects":{},"vertices":[]}
        {"type":"CityJSON","CityObjects":{},"vertices":[]}
        {"type":"CityJSONFeature","id":"id-3","CityObjects":{},"vertices":[]}
        {"type":"CityJSONFeature","id":"id-4","CityObjects":{},"vertices":[]}"#;

        for result in CityFeatureStreamDeserializer::new(&feature_sequence) {
            println!("{:#?}", result)
        }

        // // from slice
        // for result in CityFeatureStreamDeserializer::new(feature_sequence.as_bytes()) {
        //     println!("{:#?}", result)
        // }

        // Using a Cursor, flatten (panics) and from_str
        let stream = Cursor::new(feature_sequence);
        for s in stream.lines().flatten() {
            let _ = CityFeature::from_str(&s);
        }

        // Using the default serde_json::StreamDeserializer, but it stops after returning the first
        // error
        let stream = Deserializer::from_str(&feature_sequence).into_iter::<CityFeature>();
        for cityfeature_res in stream.skip_while(|cf| cf.is_err()) {
            match cityfeature_res {
                Ok(cf) => println!("{}", cf),
                Err(e) => println!("error: {}", e),
            }
        }
    }

    #[test]
    fn features_from_file() {
        let pb: PathBuf = test_data_dir().join("minimal_valid.city.jsonl");
        let cm = CityModel::from_file(&pb);
        println!("From jsonl: {:?}", cm);
    }

    #[test]
    fn citymodel_to_features_iter() {
        let mut cityobjects = CityObjects::new();
        cityobjects.insert("id-1".to_string(), CityObject);
        cityobjects.insert("id-2".to_string(), CityObject);
        cityobjects.insert("id-3".to_string(), CityObject);
        let cm = CityModel {
            version: CityJSONVersion::V1_1,
            transform: None,
            cityobjects,
        };
        let cityfeature_iter: CityFeatureIterator = cm.to_features();
        for cf in cityfeature_iter {
            println!("{:?}", cf)
        }
        // The CityModel should still own its CityObject-s
        assert_eq!(cm.cityobjects.len(), 3);
        println!("{:?}", cm.cityobjects["id-1"]);

        let cityfeature_iter: CityFeatureIterator = cm.to_features();
        let cityjsonfeature_iter = cityfeature_iter.map(|cityfeature| cityfeature.to_string());
        for cityjsonfeature in cityjsonfeature_iter.flatten() {
            println!("{}", cityjsonfeature);
        }
    }

    #[test]
    fn features_to_file() {
        let mut cityobjects = CityObjects::new();
        cityobjects.insert("id-1".to_string(), CityObject);
        cityobjects.insert("id-2".to_string(), CityObject);
        cityobjects.insert("id-3".to_string(), CityObject);
        let cm = CityModel {
            version: CityJSONVersion::V1_1,
            transform: None,
            cityobjects,
        };

        let pb: PathBuf = test_output_dir().join(".test_out.city.jsonl");
        let _ = cm.to_file(pb);
    }

    #[test]
    fn metadata_setters() {
        let mut metadata = Metadata::new();
        metadata.set_geographical_extent([1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        metadata.set_identifier("123-456-789");
        metadata.set_reference_date("1977-02-28");
        metadata.set_reference_system("https://www.opengis.net/def/crs/EPSG/0/7415");
        metadata.set_title("My Model");

        metadata.set_role(ContactRole::Author);
        metadata.set_contact_name("Balázs Dukai");
        metadata.set_email_address("my@email.com");
        metadata.set_website("http://3dbag.nl");
        metadata.set_contact_type(ContactType::Organization);
        metadata.set_address("24 Sussex Drive, Ottawa, Canada");
        metadata.set_phone("+1-613-992-4211");
        metadata.set_organization("3DGI");

        println!("{:?}", metadata.identifier());
    }
}
