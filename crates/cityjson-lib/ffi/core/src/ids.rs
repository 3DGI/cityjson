use crate::abi::{
    cj_cityobject_id_t, cj_geometry_id_t, cj_geometry_template_id_t, cj_material_id_t,
    cj_semantic_id_t, cj_texture_id_t,
};
use cityjson_lib::cityjson_types::resources::handles::{
    CityObjectHandle, GeometryHandle, GeometryTemplateHandle, MaterialHandle, SemanticHandle,
    TextureHandle,
};

macro_rules! define_id {
    ($name:ident, $handle:ty) => {
        impl $name {
            pub const fn null() -> Self {
                Self {
                    slot: 0,
                    generation: 0,
                    reserved: 0,
                }
            }

            pub const fn is_null(self) -> bool {
                self.slot == 0 && self.generation == 0
            }
        }

        impl From<$handle> for $name {
            fn from(value: $handle) -> Self {
                let (slot, generation) = value.raw_parts();
                Self {
                    slot,
                    generation,
                    reserved: 0,
                }
            }
        }

        impl From<$name> for $handle {
            fn from(value: $name) -> Self {
                // SAFETY: FFI ids are only constructed from trusted handle raw parts.
                unsafe { <$handle>::from_raw_parts_unchecked(value.slot, value.generation) }
            }
        }
    };
}

define_id!(cj_cityobject_id_t, CityObjectHandle);
define_id!(cj_geometry_id_t, GeometryHandle);
define_id!(cj_geometry_template_id_t, GeometryTemplateHandle);
define_id!(cj_semantic_id_t, SemanticHandle);
define_id!(cj_material_id_t, MaterialHandle);
define_id!(cj_texture_id_t, TextureHandle);
