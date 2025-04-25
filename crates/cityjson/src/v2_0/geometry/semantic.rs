use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::traits::semantic::{SemanticTrait, SemanticTypeTrait};
use crate::format_option;
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
    attributes: Option<Attributes<SS, RR>>,
}

impl<RR: ResourceRef, SS: StringStorage> SemanticTrait<RR, SS, SemanticType<SS>>
    for Semantic<RR, SS>
{
    #[inline]
    fn new(type_semantic: SemanticType<SS>) -> Self {
        Self {
            type_semantic,
            children: None,
            parent: None,
            attributes: None,
        }
    }
    #[inline]
    fn type_semantic(&self) -> &SemanticType<SS> {
        &self.type_semantic
    }
    #[inline]
    fn has_children(&self) -> bool {
        self.children.as_ref().map_or(false, |c| !c.is_empty())
    }
    #[inline]
    fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    #[inline]
    fn children(&self) -> Option<&Vec<RR>> {
        self.children.as_ref()
    }
    #[inline]
    fn children_mut(&mut self) -> &mut Vec<RR> {
        if self.children.is_none() {
            self.children = Some(Vec::new());
        }
        self.children.as_mut().unwrap()
    }
    #[inline]
    fn parent(&self) -> Option<&RR> {
        self.parent.as_ref()
    }
    #[inline]
    fn set_parent(&mut self, parent_ref: RR) {
        self.parent = Some(parent_ref);
    }
    #[inline]
    fn attributes(&self) -> Option<&Attributes<SS, RR>> {
        self.attributes.as_ref()
    }
    #[inline]
    fn attributes_mut(&mut self) -> &mut Attributes<SS, RR> {
        if self.attributes.is_none() {
            self.attributes = Some(Attributes::new());
        }
        self.attributes.as_mut().unwrap()
    }
}

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

/// Semantic surface type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
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
    InteriorWallSurface,
    CeilingSurface,
    FloorSurface,
    WaterSurface,
    WaterGroundSurface,
    WaterClosureSurface,
    TrafficArea,
    AuxiliaryTrafficArea,
    TransportationMarking,
    TransportationHole,
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
}
