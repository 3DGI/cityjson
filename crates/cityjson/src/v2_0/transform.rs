use crate::cityjson::core::transform;
use crate::macros::impl_core_transform_methods;
use std::fmt::{Display, Formatter};

#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Transform(transform::TransformCore);

impl Display for Transform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_inner())
    }
}

impl_core_transform_methods!(Transform);
