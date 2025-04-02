use crate::prelude::{Attributes, MetadataTrait, ResourceRef, StringStorage};

pub type Metadata<SS, RR> = Attributes<SS, RR>;

impl<SS: StringStorage, RR: ResourceRef> MetadataTrait<SS> for Metadata<SS, RR> {}

