use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::traits::semantic::SemanticTypeTrait;
use crate::format_option;
use crate::macros::impl_semantic_trait;
use crate::resources::pool::ResourceRef;
use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Semantic<RR: ResourceRef, SS: StringStorage> {
    /// The type of the semantic surface
    type_semantic: SemanticType<SS>,
    /// Indices to child semantics in the global semantics pool
    children: Option<Vec<RR>>,
    /// Index to parent semantic in the global semantics pool
    parent: Option<RR>,
    /// Additional attributes of the semantic surface
    attributes: Option<Attributes<SS>>,
}

impl_semantic_trait!(SemanticType<SS>);

impl<RR: ResourceRef, SS: StringStorage> Display for Semantic<RR, SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type: {}, children: {:?}, parent: {:?}, attributes: {}",
            self.type_semantic,
            self.children,
            self.parent,
            format_option(&self.attributes)
        )
    }
}

#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum SemanticType<SS: StringStorage> {
    #[default]
    Default,
    RoofSurface,
    GroundSurface,
    WallSurface,
    ClosureSurface,
    OuterCeilingSurface,
    OuterFloorSurface,
    Window,
    Door,
    WaterSurface,
    WaterGroundSurface,
    WaterClosureSurface,
    TrafficArea,
    AuxiliaryTrafficArea,
    Extension(SS::String),
}

impl<SS: StringStorage> Display for SemanticType<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<SS: StringStorage> SemanticTypeTrait for SemanticType<SS> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::core::attributes::AttributeValue;
    use crate::resources::pool::ResourceId32;
    use crate::resources::storage::OwnedStringStorage;

    #[test]
    fn test_semantic_creation() {
        let semantic = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        assert!(!semantic.has_children());
        assert!(!semantic.has_parent());
        assert!(semantic.children().is_none());
        assert!(semantic.parent().is_none());
        assert!(semantic.attributes().is_none());
    }

    #[test]
    fn test_semantic_attributes() {
        let mut semantic =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        // Initially no attributes
        assert!(semantic.attributes().is_none());

        // Get mutable reference and add attributes
        let attrs = semantic.attributes_mut();
        attrs.insert(
            "material".to_string(),
            AttributeValue::String("brick".to_string()),
        );
        attrs.insert(
            "color".to_string(),
            AttributeValue::String("red".to_string()),
        );

        // Now attributes should exist
        assert!(semantic.attributes().is_some());
        match semantic.attributes().unwrap().get("material") {
            Some(AttributeValue::String(v)) => assert_eq!(v, "brick"),
            _ => panic!("Expected string value"),
        }
        match semantic.attributes().unwrap().get("color") {
            Some(AttributeValue::String(v)) => assert_eq!(v, "red"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_semantic_children() {
        let mut semantic =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        // Initially no children
        assert!(!semantic.has_children());

        // Add children
        let children = semantic.children_mut();
        children.push(ResourceId32::new(1, 0));
        children.push(ResourceId32::new(2, 0));

        // Now should have children
        assert!(semantic.has_children());
        assert_eq!(semantic.children().unwrap().len(), 2);
        assert_eq!(semantic.children().unwrap()[0], ResourceId32::new(1, 0));
        assert_eq!(semantic.children().unwrap()[1], ResourceId32::new(2, 0));
    }

    #[test]
    fn test_semantic_parent() {
        let mut semantic = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);

        // Initially no parent
        assert!(!semantic.has_parent());
        assert!(semantic.parent().is_none());

        // Set parent manually
        semantic.parent = Some(ResourceId32::new(5, 0));

        // Now should have parent
        assert!(semantic.has_parent());
        assert_eq!(*semantic.parent().unwrap(), ResourceId32::new(5, 0));

        semantic.set_parent(ResourceId32::new(10, 0));
        assert_eq!(*semantic.parent().unwrap(), ResourceId32::new(10, 0));
    }

    #[test]
    fn test_semantic_display() {
        let mut semantic =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        let display_str = format!("{}", semantic);
        assert!(display_str.contains("RoofSurface"));

        // Add attributes and check display again
        let attrs = semantic.attributes_mut();
        attrs.insert(
            "material".to_string(),
            AttributeValue::String("tile".to_string()),
        );

        let display_str = format!("{}", semantic);
        assert!(display_str.contains("RoofSurface"));
        assert!(display_str.contains("attributes"));
        println!("{}", semantic);
    }

    #[test]
    fn test_semantic_type_extension() {
        let extension_type = SemanticType::Extension("CustomType".to_string());
        let semantic = Semantic::<ResourceId32, OwnedStringStorage>::new(extension_type);
        let display_str = format!("{}", semantic);
        assert!(display_str.contains("Extension"));
    }

    #[test]
    fn test_semantic_equality() {
        // Test 1: Two semantics with same type and no other fields are equal
        let semantic1 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        let semantic2 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        assert_eq!(semantic1, semantic2);

        // Test 2: Two semantics with different types are not equal
        let semantic3 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        assert_ne!(semantic1, semantic3);

        // Test 3: Two semantics with same type and same children are equal
        let mut semantic4 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        let mut semantic5 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        semantic4.children_mut().push(ResourceId32::new(1, 0));
        semantic4.children_mut().push(ResourceId32::new(2, 0));
        semantic5.children_mut().push(ResourceId32::new(1, 0));
        semantic5.children_mut().push(ResourceId32::new(2, 0));
        assert_eq!(semantic4, semantic5);

        // Test 4: Two semantics with different children are not equal
        let mut semantic6 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        semantic6.children_mut().push(ResourceId32::new(3, 0));
        assert_ne!(semantic4, semantic6);

        // Test 5: Two semantics with same parent are equal
        let mut semantic7 = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);
        let mut semantic8 = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);
        semantic7.set_parent(ResourceId32::new(10, 0));
        semantic8.set_parent(ResourceId32::new(10, 0));
        assert_eq!(semantic7, semantic8);

        // Test 6: Two semantics with different parents are not equal
        let mut semantic9 = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);
        semantic9.set_parent(ResourceId32::new(20, 0));
        assert_ne!(semantic7, semantic9);

        // Test 7: Two semantics with same attributes are equal
        let mut semantic10 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        let mut semantic11 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);

        semantic10.attributes_mut().insert(
            "material".to_string(),
            AttributeValue::String("tile".to_string()),
        );
        semantic10
            .attributes_mut()
            .insert("year".to_string(), AttributeValue::Integer(2020));

        semantic11.attributes_mut().insert(
            "material".to_string(),
            AttributeValue::String("tile".to_string()),
        );
        semantic11
            .attributes_mut()
            .insert("year".to_string(), AttributeValue::Integer(2020));
        assert_eq!(semantic10, semantic11);

        // Test 8: Two semantics with different attributes are not equal
        let mut semantic12 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        semantic12.attributes_mut().insert(
            "material".to_string(),
            AttributeValue::String("slate".to_string()),
        );
        assert_ne!(semantic10, semantic12);

        // Test 9: Two semantics with all fields equal are equal
        let mut semantic13 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        let mut semantic14 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        semantic13.children_mut().push(ResourceId32::new(1, 0));
        semantic13.set_parent(ResourceId32::new(5, 0));
        semantic13.attributes_mut().insert(
            "color".to_string(),
            AttributeValue::String("blue".to_string()),
        );

        semantic14.children_mut().push(ResourceId32::new(1, 0));
        semantic14.set_parent(ResourceId32::new(5, 0));
        semantic14.attributes_mut().insert(
            "color".to_string(),
            AttributeValue::String("blue".to_string()),
        );
        assert_eq!(semantic13, semantic14);
    }
}
