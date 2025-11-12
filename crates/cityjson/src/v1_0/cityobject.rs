use crate::cityjson::core::cityobject::{CityObjectCore, CityObjectsCore};
use crate::prelude::*;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct CityObjects<SS: StringStorage, RR: ResourceRef> {
    inner: CityObjectsCore<SS, RR, CityObject<SS, RR>>,
}

crate::macros::impl_cityobjects_methods!();

/// A CityObjects container using owned strings.
pub type OwnedCityObjects<RR> = CityObjects<OwnedStringStorage, RR>;

/// A CityObjects container using borrowed strings.
pub type BorrowedCityObjects<'a, RR> = CityObjects<BorrowedStringStorage<'a>, RR>;

#[derive(Debug, Default, Clone)]
pub struct CityObject<SS: StringStorage, RR: ResourceRef> {
    inner: CityObjectCore<SS, RR, CityObjectType<SS>>,
}

crate::macros::impl_cityobject_methods!(CityObjectType<SS>);

impl<SS: StringStorage, RR: ResourceRef> Display for CityObject<SS, RR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum CityObjectType<SS: StringStorage> {
    Bridge,
    BridgePart,
    BridgeInstallation,
    BridgeConstructionElement,

    Building,
    BuildingPart,
    BuildingInstallation,

    CityFurniture,
    CityObjectGroup,
    #[default]
    Default,
    GenericCityObject,
    LandUse,

    PlantCover,
    SolitaryVegetationObject,
    TINRelief,
    WaterBody,
    Road,
    Railway,

    TransportSquare,
    Tunnel,
    TunnelPart,
    TunnelInstallation,

    Extension(SS::String),
}

impl<SS: StringStorage> Display for CityObjectType<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let CityObjectType::Extension(ext) = self {
            write!(f, "{}", ext)
        } else {
            write!(f, "{:#?}", self)
        }
    }
}

#[cfg(test)]
mod tests_cityobjects_container {
    use super::*;
    use crate::cityjson::core::attributes::AttributeValue;
    use crate::resources::pool::ResourceId32;

    #[test]
    fn test_basic_operations() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create a test object
        let obj = CityObject::new("id-1".to_string(), CityObjectType::Building);

        // Add the object
        let id = objects.add(obj);

        // Check length
        assert_eq!(objects.len(), 1);
        assert!(!objects.is_empty());

        // Get the object
        let retrieved_obj = objects.get(id).unwrap();
        assert_eq!(retrieved_obj.type_cityobject(), &CityObjectType::Building);

        // Modify the object
        if let Some(obj_mut) = objects.get_mut(id) {
            obj_mut.attributes_mut().insert(
                "test".to_string(),
                AttributeValue::String("value".to_string()),
            );
        }

        // Verify modification
        let modified_obj = objects.get(id).unwrap();
        assert!(modified_obj.attributes().is_some());

        // Remove the object
        let removed_obj = objects.remove(id).unwrap();
        assert_eq!(removed_obj.type_cityobject(), &CityObjectType::Building);

        // Check the container is empty
        // assert_eq!(objects.len(), 0);
        // assert!(objects.is_empty());
    }

    #[test]
    fn test_filtering() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Add various types of objects
        objects.add(CityObject::new(
            "id-1".to_string(),
            CityObjectType::Building,
        ));
        objects.add(CityObject::new("id-2".to_string(), CityObjectType::Bridge));
        objects.add(CityObject::new(
            "id-3".to_string(),
            CityObjectType::Building,
        ));
    }

    #[test]
    fn test_bulk_operations() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create multiple objects
        let objs = vec![
            CityObject::new("id-1".to_string(), CityObjectType::Building),
            CityObject::new("id-2".to_string(), CityObjectType::Bridge),
            CityObject::new("id-3".to_string(), CityObjectType::Road),
        ];

        // Add in bulk
        let ids = objects.add_many(objs);
        assert_eq!(ids.len(), 3);
        assert_eq!(objects.len(), 3);

        // Clear all
        // objects.clear();
        // assert_eq!(objects.len(), 0);
    }

    #[test]
    fn test_attribute_filtering() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create objects with attributes
        let mut building1 = CityObject::new("id-1".to_string(), CityObjectType::Building);
        building1
            .attributes_mut()
            .insert("height".to_string(), AttributeValue::Float(15.0));
        objects.add(building1);

        let mut building2 = CityObject::new("id-2".to_string(), CityObjectType::Building);
        building2
            .attributes_mut()
            .insert("width".to_string(), AttributeValue::Float(30.0));
        objects.add(building2);
    }
}
