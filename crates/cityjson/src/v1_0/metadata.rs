use crate::prelude::{Attributes, ResourceRef};

#[allow(type_alias_bounds)]
pub type Metadata<SS, RR: ResourceRef> = Attributes<SS, RR>;
