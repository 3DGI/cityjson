//! Tests for semantic-related functionality.

use cityjson::resources::handles::SemanticHandle;
use cityjson::v2_0::{OwnedAttributeValue, OwnedSemantic, SemanticType};

/// Two identical semantic objects should be considered equal.
#[test]
fn semantic_equality() {
    let mut semantic1 = OwnedSemantic::new(SemanticType::RoofSurface);
    semantic1.children_mut().push(SemanticHandle::default());
    semantic1.set_parent(SemanticHandle::default());
    semantic1.attributes_mut().insert(
        "material".to_string(),
        OwnedAttributeValue::String("tile".to_string()),
    );

    let mut semantic2 = OwnedSemantic::new(SemanticType::RoofSurface);
    semantic2.children_mut().push(SemanticHandle::default());
    semantic2.set_parent(SemanticHandle::default());
    semantic2.attributes_mut().insert(
        "material".to_string(),
        OwnedAttributeValue::String("tile".to_string()),
    );

    assert_eq!(semantic1, semantic2);

    let mut semantic3 = OwnedSemantic::new(SemanticType::GroundSurface);
    semantic3.children_mut().push(SemanticHandle::default());
    semantic3.set_parent(SemanticHandle::default());
    semantic3.attributes_mut().insert(
        "material".to_string(),
        OwnedAttributeValue::String("tile".to_string()),
    );
    assert_ne!(semantic1, semantic3);

    let mut semantic4 = OwnedSemantic::new(SemanticType::RoofSurface);
    semantic4.children_mut().push(SemanticHandle::default());
    semantic4.set_parent(SemanticHandle::default());
    semantic4.attributes_mut().insert(
        "material".to_string(),
        OwnedAttributeValue::String("metal".to_string()),
    );
    assert_ne!(semantic1, semantic4);

    let mut semantic5 = OwnedSemantic::new(SemanticType::RoofSurface);
    semantic5.set_parent(SemanticHandle::default());
    semantic5.attributes_mut().insert(
        "material".to_string(),
        OwnedAttributeValue::String("tile".to_string()),
    );
    assert_ne!(semantic1, semantic5);
}
