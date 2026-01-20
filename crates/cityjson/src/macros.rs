macro_rules! impl_core_transform_methods {
    ($type:ty) => {
        impl $type {
            pub fn new() -> Self {
                Self(transform::TransformCore::new())
            }
            pub fn scale(&self) -> [f64; 3] {
                self.0.scale()
            }
            pub fn translate(&self) -> [f64; 3] {
                self.0.translate()
            }
            pub fn set_scale(&mut self, scale: [f64; 3]) {
                self.0.set_scale(scale);
            }
            pub fn set_translate(&mut self, translate: [f64; 3]) {
                self.0.set_translate(translate);
            }

            pub(crate) fn as_inner(&self) -> &transform::TransformCore {
                &self.0
            }
            #[allow(unused)]
            pub(crate) fn as_inner_mut(&mut self) -> &mut transform::TransformCore {
                &mut self.0
            }
        }
    };
}
pub(crate) use impl_core_transform_methods;

macro_rules! impl_geometry_methods {
    () => {
        impl<
            VR: crate::cityjson::core::vertex::VertexRef,
            RR: crate::resources::pool::ResourceRef,
            SS: crate::resources::storage::StringStorage,
        > Geometry<VR, RR, SS>
        {
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                type_geometry: crate::cityjson::core::geometry::GeometryType,
                lod: Option<crate::cityjson::core::geometry::LoD>,
                boundaries: Option<crate::cityjson::core::boundary::Boundary<VR>>,
                semantics: Option<crate::resources::mapping::SemanticMap<VR, RR>>,
                materials: Option<
                    Vec<(SS::String, crate::resources::mapping::MaterialMap<VR, RR>)>,
                >,
                textures: Option<Vec<(SS::String, crate::resources::mapping::TextureMap<VR, RR>)>>,
                instance_template: Option<RR>,
                instance_reference_point: Option<crate::cityjson::core::vertex::VertexIndex<VR>>,
                instance_transformation_matrix: Option<[f64; 16]>,
            ) -> Self {
                Self {
                    inner: crate::cityjson::core::geometry_struct::GeometryCore::new(
                        type_geometry,
                        lod,
                        boundaries,
                        semantics,
                        materials,
                        textures,
                        instance_template,
                        instance_reference_point,
                        instance_transformation_matrix,
                    ),
                }
            }

            pub fn type_geometry(&self) -> &crate::cityjson::core::geometry::GeometryType {
                self.inner.type_geometry()
            }

            pub fn lod(&self) -> Option<&crate::cityjson::core::geometry::LoD> {
                self.inner.lod()
            }

            pub fn boundaries(&self) -> Option<&crate::cityjson::core::boundary::Boundary<VR>> {
                self.inner.boundaries()
            }

            pub fn semantics(&self) -> Option<&crate::resources::mapping::SemanticMap<VR, RR>> {
                self.inner.semantics()
            }

            pub fn materials(
                &self,
            ) -> Option<&Vec<(SS::String, crate::resources::mapping::MaterialMap<VR, RR>)>> {
                self.inner.materials()
            }

            pub fn textures(
                &self,
            ) -> Option<&Vec<(SS::String, crate::resources::mapping::TextureMap<VR, RR>)>> {
                self.inner.textures()
            }

            pub fn instance_template(&self) -> Option<&RR> {
                self.inner.instance_template()
            }

            pub fn instance_reference_point(
                &self,
            ) -> Option<&crate::cityjson::core::vertex::VertexIndex<VR>> {
                self.inner.instance_reference_point()
            }

            pub fn instance_transformation_matrix(&self) -> Option<&[f64; 16]> {
                self.inner.instance_transformation_matrix()
            }
        }

        // Internal trait implementation for geometry construction
        impl<
            VR: crate::cityjson::core::vertex::VertexRef,
            RR: crate::resources::pool::ResourceRef,
            SS: crate::resources::storage::StringStorage,
        > crate::cityjson::core::geometry::GeometryConstructor<VR, RR, SS::String>
            for Geometry<VR, RR, SS>
        {
            #[allow(clippy::too_many_arguments)]
            fn new(
                type_geometry: crate::cityjson::core::geometry::GeometryType,
                lod: Option<crate::cityjson::core::geometry::LoD>,
                boundaries: Option<crate::cityjson::core::boundary::Boundary<VR>>,
                semantics: Option<crate::resources::mapping::SemanticMap<VR, RR>>,
                materials: Option<
                    Vec<(SS::String, crate::resources::mapping::MaterialMap<VR, RR>)>,
                >,
                textures: Option<Vec<(SS::String, crate::resources::mapping::TextureMap<VR, RR>)>>,
                instance_template: Option<RR>,
                instance_reference_point: Option<crate::cityjson::core::vertex::VertexIndex<VR>>,
                instance_transformation_matrix: Option<[f64; 16]>,
            ) -> Self {
                Self::new(
                    type_geometry,
                    lod,
                    boundaries,
                    semantics,
                    materials,
                    textures,
                    instance_template,
                    instance_reference_point,
                    instance_transformation_matrix,
                )
            }
        }
    };
}
pub(crate) use impl_geometry_methods;

macro_rules! impl_cityobject_methods {
    ($cityobject_type:ty) => {
        impl<SS: crate::resources::storage::StringStorage, RR: crate::resources::pool::ResourceRef>
            CityObject<SS, RR>
        {
            pub fn new(id: SS::String, type_cityobject: $cityobject_type) -> Self {
                Self {
                    inner: crate::cityjson::core::cityobject::CityObjectCore::new(
                        id,
                        type_cityobject,
                    ),
                }
            }

            pub fn id(&self) -> &SS::String {
                self.inner.id()
            }

            pub fn type_cityobject(&self) -> &$cityobject_type {
                self.inner.type_cityobject()
            }

            pub fn geometry(&self) -> Option<&Vec<RR>> {
                self.inner.geometry()
            }

            pub fn geometry_mut(&mut self) -> &mut Vec<RR> {
                self.inner.geometry_mut()
            }

            pub fn attributes(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
                self.inner.attributes()
            }

            pub fn attributes_mut(
                &mut self,
            ) -> &mut crate::cityjson::core::attributes::Attributes<SS> {
                self.inner.attributes_mut()
            }

            pub fn geographical_extent(&self) -> Option<&crate::cityjson::core::metadata::BBox> {
                self.inner.geographical_extent()
            }

            pub fn set_geographical_extent(
                &mut self,
                bbox: Option<crate::cityjson::core::metadata::BBox>,
            ) {
                self.inner.set_geographical_extent(bbox);
            }

            pub fn children(&self) -> Option<&Vec<RR>> {
                self.inner.children()
            }

            pub fn children_mut(&mut self) -> &mut Vec<RR> {
                self.inner.children_mut()
            }

            pub fn parents(&self) -> Option<&Vec<RR>> {
                self.inner.parents()
            }

            pub fn parents_mut(&mut self) -> &mut Vec<RR> {
                self.inner.parents_mut()
            }

            pub fn extra(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
                self.inner.extra()
            }

            pub fn extra_mut(&mut self) -> &mut crate::cityjson::core::attributes::Attributes<SS> {
                self.inner.extra_mut()
            }
        }
    };
}
pub(crate) use impl_cityobject_methods;

macro_rules! impl_cityobjects_methods {
    () => {
        impl<SS: crate::resources::storage::StringStorage, RR: crate::resources::pool::ResourceRef>
            CityObjects<SS, RR>
        {
            pub fn new() -> Self {
                Self {
                    inner: crate::cityjson::core::cityobject::CityObjectsCore::new(),
                }
            }

            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    inner: crate::cityjson::core::cityobject::CityObjectsCore::with_capacity(
                        capacity,
                    ),
                }
            }

            pub fn add(&mut self, city_object: CityObject<SS, RR>) -> RR {
                self.inner.add(city_object)
            }

            pub fn get(&self, id: RR) -> Option<&CityObject<SS, RR>> {
                self.inner.get(id)
            }

            pub fn get_mut(&mut self, id: RR) -> Option<&mut CityObject<SS, RR>> {
                self.inner.get_mut(id)
            }

            pub fn remove(&mut self, id: RR) -> Option<CityObject<SS, RR>> {
                self.inner.remove(id)
            }

            pub fn len(&self) -> usize {
                self.inner.len()
            }

            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }

            pub fn iter<'a>(&'a self) -> impl Iterator<Item = (RR, &'a CityObject<SS, RR>)>
            where
                CityObject<SS, RR>: 'a,
            {
                self.inner.iter()
            }

            pub fn iter_mut<'a>(
                &'a mut self,
            ) -> impl Iterator<Item = (RR, &'a mut CityObject<SS, RR>)>
            where
                CityObject<SS, RR>: 'a,
            {
                self.inner.iter_mut()
            }

            pub fn first(&self) -> Option<(RR, &CityObject<SS, RR>)> {
                self.inner.first()
            }

            pub fn last(&self) -> Option<(RR, &CityObject<SS, RR>)> {
                self.inner.last()
            }

            pub fn ids(&self) -> Vec<RR> {
                self.inner.ids()
            }

            pub fn add_many<I: IntoIterator<Item = CityObject<SS, RR>>>(
                &mut self,
                objects: I,
            ) -> Vec<RR> {
                self.inner.add_many(objects)
            }

            pub fn clear(&mut self) {
                self.inner.clear();
            }

            pub fn filter<F>(&self, predicate: F) -> Vec<(RR, &CityObject<SS, RR>)>
            where
                F: Fn(&CityObject<SS, RR>) -> bool,
            {
                self.inner.filter(predicate)
            }
        }

        impl<SS: crate::resources::storage::StringStorage, RR: crate::resources::pool::ResourceRef>
            Default for CityObjects<SS, RR>
        {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<SS: crate::resources::storage::StringStorage, RR: crate::resources::pool::ResourceRef>
            Extend<CityObject<SS, RR>> for CityObjects<SS, RR>
        {
            fn extend<T: IntoIterator<Item = CityObject<SS, RR>>>(&mut self, iter: T) {
                for obj in iter {
                    self.add(obj);
                }
            }
        }

        impl<SS: crate::resources::storage::StringStorage, RR: crate::resources::pool::ResourceRef>
            FromIterator<CityObject<SS, RR>> for CityObjects<SS, RR>
        {
            fn from_iter<T: IntoIterator<Item = CityObject<SS, RR>>>(iter: T) -> Self {
                let mut objects = Self::new();
                objects.extend(iter);
                objects
            }
        }
    };
}
pub(crate) use impl_cityobjects_methods;

macro_rules! impl_extension_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage> Extension<SS> {
            pub fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
                Self {
                    inner: crate::cityjson::core::extension::ExtensionCore::new(name, url, version),
                }
            }

            pub fn name(&self) -> &SS::String {
                self.inner.name()
            }

            pub fn url(&self) -> &SS::String {
                self.inner.url()
            }

            pub fn version(&self) -> &SS::String {
                self.inner.version()
            }
        }

        impl<SS: crate::resources::storage::StringStorage>
            crate::cityjson::core::extension::ExtensionItem<SS> for Extension<SS>
        {
            fn name(&self) -> &SS::String {
                self.inner.name()
            }
        }
    };
}
pub(crate) use impl_extension_trait;

macro_rules! impl_extensions_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage> Extensions<SS> {
            pub fn new() -> Self {
                Self {
                    inner: crate::cityjson::core::extension::ExtensionsCore::new(),
                }
            }

            pub fn add(&mut self, extension: Extension<SS>) -> &mut Self {
                self.inner.add(extension);
                self
            }

            pub fn remove(&mut self, name: SS::String) -> bool {
                self.inner.remove(name)
            }

            pub fn get(&self, name: &str) -> Option<&Extension<SS>> {
                self.inner.get(name)
            }

            pub fn len(&self) -> usize {
                self.inner.len()
            }

            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }
        }

        impl<SS: crate::resources::storage::StringStorage> IntoIterator for Extensions<SS> {
            type Item = Extension<SS>;
            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }

        impl<'a, SS: crate::resources::storage::StringStorage> IntoIterator for &'a Extensions<SS> {
            type Item = &'a Extension<SS>;
            type IntoIter = std::slice::Iter<'a, Extension<SS>>;

            fn into_iter(self) -> Self::IntoIter {
                (&self.inner).into_iter()
            }
        }

        impl<'a, SS: crate::resources::storage::StringStorage> IntoIterator
            for &'a mut Extensions<SS>
        {
            type Item = &'a mut Extension<SS>;
            type IntoIter = std::slice::IterMut<'a, Extension<SS>>;

            fn into_iter(self) -> Self::IntoIter {
                (&mut self.inner).into_iter()
            }
        }

        impl<SS: crate::resources::storage::StringStorage> std::fmt::Display for Extensions<SS> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.inner)
            }
        }
    };
}
pub(crate) use impl_extensions_trait;

macro_rules! impl_material_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage> Material<SS> {
            pub fn new(name: SS::String) -> Self {
                Self {
                    name,
                    ambient_intensity: None,
                    diffuse_color: None,
                    emissive_color: None,
                    specular_color: None,
                    shininess: None,
                    transparency: None,
                    is_smooth: None,
                }
            }
            #[inline]
            pub fn name(&self) -> &SS::String {
                &self.name
            }
            #[inline]
            pub fn set_name(&mut self, name: SS::String) {
                self.name = name;
            }
            #[inline]
            pub fn ambient_intensity(&self) -> Option<f32> {
                self.ambient_intensity
            }
            #[inline]
            pub fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>) {
                self.ambient_intensity = ambient_intensity;
            }
            #[inline]
            pub fn diffuse_color(&self) -> Option<&crate::cityjson::core::appearance::RGB> {
                self.diffuse_color.as_ref()
            }
            #[inline]
            pub fn set_diffuse_color(
                &mut self,
                diffuse_color: Option<crate::cityjson::core::appearance::RGB>,
            ) {
                self.diffuse_color = diffuse_color;
            }
            #[inline]
            pub fn emissive_color(&self) -> Option<&crate::cityjson::core::appearance::RGB> {
                self.emissive_color.as_ref()
            }
            #[inline]
            pub fn set_emissive_color(
                &mut self,
                emissive_color: Option<crate::cityjson::core::appearance::RGB>,
            ) {
                self.emissive_color = emissive_color;
            }
            #[inline]
            pub fn specular_color(&self) -> Option<&crate::cityjson::core::appearance::RGB> {
                self.specular_color.as_ref()
            }
            #[inline]
            pub fn set_specular_color(
                &mut self,
                specular_color: Option<crate::cityjson::core::appearance::RGB>,
            ) {
                self.specular_color = specular_color;
            }
            #[inline]
            pub fn shininess(&self) -> Option<f32> {
                self.shininess
            }
            #[inline]
            pub fn set_shininess(&mut self, shininess: Option<f32>) {
                self.shininess = shininess;
            }
            #[inline]
            pub fn transparency(&self) -> Option<f32> {
                self.transparency
            }
            #[inline]
            pub fn set_transparency(&mut self, transparency: Option<f32>) {
                self.transparency = transparency;
            }
            #[inline]
            pub fn is_smooth(&self) -> Option<bool> {
                self.is_smooth
            }
            #[inline]
            pub fn set_is_smooth(&mut self, is_smooth: Option<bool>) {
                self.is_smooth = is_smooth;
            }
        }
    };
}
pub(crate) use impl_material_trait;

macro_rules! impl_texture_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage> Texture<SS> {
            #[inline]
            pub fn new(
                image: SS::String,
                image_type: crate::cityjson::core::appearance::ImageType,
            ) -> Self {
                Self {
                    image_type,
                    image,
                    wrap_mode: None,
                    texture_type: None,
                    border_color: None,
                }
            }
            #[inline]
            pub fn image_type(&self) -> &crate::cityjson::core::appearance::ImageType {
                &self.image_type
            }
            #[inline]
            pub fn set_image_type(
                &mut self,
                image_type: crate::cityjson::core::appearance::ImageType,
            ) {
                self.image_type = image_type;
            }
            #[inline]
            pub fn image(&self) -> &SS::String {
                &self.image
            }
            #[inline]
            pub fn set_image(&mut self, image: SS::String) {
                self.image = image;
            }
            #[inline]
            pub fn wrap_mode(&self) -> Option<crate::cityjson::core::appearance::WrapMode> {
                self.wrap_mode
            }
            #[inline]
            pub fn set_wrap_mode(
                &mut self,
                wrap_mode: Option<crate::cityjson::core::appearance::WrapMode>,
            ) {
                self.wrap_mode = wrap_mode;
            }
            #[inline]
            pub fn texture_type(&self) -> Option<crate::cityjson::core::appearance::TextureType> {
                self.texture_type
            }
            #[inline]
            pub fn set_texture_type(
                &mut self,
                texture_type: Option<crate::cityjson::core::appearance::TextureType>,
            ) {
                self.texture_type = texture_type;
            }
            #[inline]
            pub fn border_color(&self) -> Option<crate::cityjson::core::appearance::RGBA> {
                self.border_color
            }
            #[inline]
            pub fn set_border_color(
                &mut self,
                border_color: Option<crate::cityjson::core::appearance::RGBA>,
            ) {
                self.border_color = border_color;
            }
        }
    };
}
pub(crate) use impl_texture_trait;

macro_rules! impl_semantic_trait {
    ($semantic_type:ty) => {
        impl<RR: crate::resources::pool::ResourceRef, SS: crate::resources::storage::StringStorage>
            Semantic<RR, SS>
        {
            #[inline]
            pub fn new(type_semantic: $semantic_type) -> Self {
                Self {
                    type_semantic,
                    children: None,
                    parent: None,
                    attributes: None,
                }
            }
            #[inline]
            pub fn type_semantic(&self) -> &$semantic_type {
                &self.type_semantic
            }
            #[inline]
            pub fn has_children(&self) -> bool {
                self.children.as_ref().is_some_and(|c| !c.is_empty())
            }
            #[inline]
            pub fn has_parent(&self) -> bool {
                self.parent.is_some()
            }
            #[inline]
            pub fn children(&self) -> Option<&Vec<RR>> {
                self.children.as_ref()
            }
            #[inline]
            pub fn children_mut(&mut self) -> &mut Vec<RR> {
                if self.children.is_none() {
                    self.children = Some(Vec::new());
                }
                self.children.as_mut().unwrap()
            }
            #[inline]
            pub fn parent(&self) -> Option<&RR> {
                self.parent.as_ref()
            }
            #[inline]
            pub fn set_parent(&mut self, parent_ref: RR) {
                self.parent = Some(parent_ref);
            }
            #[inline]
            pub fn attributes(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
                self.attributes.as_ref()
            }
            #[inline]
            pub fn attributes_mut(
                &mut self,
            ) -> &mut crate::cityjson::core::attributes::Attributes<SS> {
                if self.attributes.is_none() {
                    self.attributes = Some(crate::cityjson::core::attributes::Attributes::new());
                }
                self.attributes.as_mut().unwrap()
            }
        }
    };
}
pub(crate) use impl_semantic_trait;

macro_rules! impl_metadata_methods {
    () => {
        impl<SS: crate::resources::storage::StringStorage> Metadata<SS> {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn geographical_extent(&self) -> Option<&crate::cityjson::core::metadata::BBox> {
                self.geographical_extent.as_ref()
            }

            pub fn identifier(
                &self,
            ) -> Option<&crate::cityjson::core::metadata::CityModelIdentifier<SS>> {
                self.identifier.as_ref()
            }

            pub fn reference_date(&self) -> Option<&crate::cityjson::core::metadata::Date<SS>> {
                self.reference_date.as_ref()
            }

            pub fn reference_system(&self) -> Option<&crate::cityjson::core::metadata::CRS<SS>> {
                self.reference_system.as_ref()
            }

            pub fn title(&self) -> Option<&str> {
                self.title.as_deref()
            }

            pub fn extra(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
                self.extra.as_ref()
            }

            pub fn extra_mut(
                &mut self,
            ) -> &mut Option<crate::cityjson::core::attributes::Attributes<SS>> {
                &mut self.extra
            }

            pub fn set_extra(
                &mut self,
                extra: Option<crate::cityjson::core::attributes::Attributes<SS>>,
            ) {
                self.extra = extra;
            }

            pub fn set_geographical_extent(&mut self, bbox: crate::cityjson::core::metadata::BBox) {
                self.geographical_extent = Some(bbox);
            }

            pub fn set_identifier(
                &mut self,
                identifier: crate::cityjson::core::metadata::CityModelIdentifier<SS>,
            ) {
                self.identifier = Some(identifier);
            }

            pub fn set_reference_date(&mut self, date: crate::cityjson::core::metadata::Date<SS>) {
                self.reference_date = Some(date);
            }

            pub fn set_reference_system(&mut self, crs: crate::cityjson::core::metadata::CRS<SS>) {
                self.reference_system = Some(crs);
            }

            pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
                self.title = Some(title.as_ref().to_owned());
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

        impl<SS: crate::resources::storage::StringStorage> std::fmt::Display for Metadata<SS> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "geographical_extent: {}, identifier: {}, point_of_contact: {},
            reference_date: {}, reference_system: {}, title: {}",
                    crate::format_option(&self.geographical_extent),
                    crate::format_option(&self.identifier),
                    crate::format_option(&self.point_of_contact),
                    crate::format_option(&self.reference_date),
                    crate::format_option(&self.reference_system),
                    crate::format_option(&self.title)
                )
            }
        }
    };
}
pub(crate) use impl_metadata_methods;

macro_rules! impl_contact_common_methods {
    () => {
        pub fn new() -> Self {
            Self {
                contact_name: "".to_string(),
                email_address: "".to_string(),
                role: None,
                website: None,
                contact_type: None,
                address: None,
                phone: None,
                organization: None,
            }
        }

        pub fn contact_name(&self) -> &str {
            &self.contact_name
        }

        pub fn email_address(&self) -> &str {
            &self.email_address
        }

        pub fn role(&self) -> Option<ContactRole> {
            self.role
        }

        pub fn website(&self) -> &Option<String> {
            &self.website
        }

        pub fn contact_type(&self) -> Option<ContactType> {
            self.contact_type
        }

        pub fn phone(&self) -> &Option<String> {
            &self.phone
        }

        pub fn organization(&self) -> &Option<String> {
            &self.organization
        }

        pub fn set_contact_name(&mut self, contact_name: String) {
            self.contact_name = contact_name;
        }

        pub fn set_email_address(&mut self, email_address: String) {
            self.email_address = email_address;
        }

        pub fn set_role(&mut self, role: Option<ContactRole>) {
            self.role = role;
        }

        pub fn set_website(&mut self, website: Option<String>) {
            self.website = website;
        }

        pub fn set_contact_type(&mut self, contact_type: Option<ContactType>) {
            self.contact_type = contact_type;
        }

        pub fn set_phone(&mut self, phone: Option<String>) {
            self.phone = phone;
        }

        pub fn set_organization(&mut self, organization: Option<String>) {
            self.organization = organization;
        }
    };
}
pub(crate) use impl_contact_common_methods;

macro_rules! impl_citymodel_methods {
    ($coordinate_type:ty, $version:expr, $metadata_type:ty) => {
        impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModel<VR, RR, SS> {
            pub fn new(type_citymodel: crate::CityModelType) -> Self {
                Self {
                    inner: crate::cityjson::core::citymodel::CityModelCore::new(
                        type_citymodel,
                        Some($version),
                    ),
                }
            }

            pub fn with_capacity(
                type_citymodel: crate::CityModelType,
                cityobjects_capacity: usize,
                vertex_capacity: usize,
                semantic_capacity: usize,
                material_capacity: usize,
                texture_capacity: usize,
                geometry_capacity: usize,
            ) -> Self {
                Self {
                    inner: crate::cityjson::core::citymodel::CityModelCore::with_capacity(
                        type_citymodel,
                        Some($version),
                        cityobjects_capacity,
                        vertex_capacity,
                        semantic_capacity,
                        material_capacity,
                        texture_capacity,
                        geometry_capacity,
                        |capacity| CityObjects::with_capacity(capacity),
                    ),
                }
            }

            // ==================== SEMANTICS ====================

            pub fn get_semantic(&self, id: RR) -> Option<&Semantic<RR, SS>> {
                self.inner.get_semantic(id)
            }

            pub fn get_semantic_mut(&mut self, id: RR) -> Option<&mut Semantic<RR, SS>> {
                self.inner.get_semantic_mut(id)
            }

            pub fn add_semantic(&mut self, semantic: Semantic<RR, SS>) -> RR {
                self.inner.add_semantic(semantic)
            }

            pub fn semantic_count(&self) -> usize {
                self.inner.semantic_count()
            }

            pub fn has_semantics(&self) -> bool {
                self.inner.has_semantics()
            }

            pub fn iter_semantics(&self) -> impl Iterator<Item = (RR, &Semantic<RR, SS>)> + '_ {
                self.inner.iter_semantics()
            }

            pub fn iter_semantics_mut(&mut self) -> impl Iterator<Item = (RR, &mut Semantic<RR, SS>)> + '_ {
                self.inner.iter_semantics_mut()
            }

            pub fn find_semantic(&self, semantic: &Semantic<RR, SS>) -> Option<RR>
            where
                Semantic<RR, SS>: PartialEq,
            {
                self.inner.find_semantic(semantic)
            }

            pub fn remove_semantic(&mut self, id: RR) -> Option<Semantic<RR, SS>> {
                self.inner.remove_semantic(id)
            }

            pub fn clear_semantics(&mut self) {
                self.inner.clear_semantics();
            }

            pub fn get_or_insert_semantic(&mut self, semantic: Semantic<RR, SS>) -> RR
            where
                Semantic<RR, SS>: PartialEq,
            {
                self.inner.get_or_insert_semantic(semantic)
            }

            // ==================== MATERIALS ====================

            pub fn get_material(&self, id: RR) -> Option<&Material<SS>> {
                self.inner.get_material(id)
            }

            pub fn get_material_mut(&mut self, id: RR) -> Option<&mut Material<SS>> {
                self.inner.get_material_mut(id)
            }

            pub fn add_material(&mut self, material: Material<SS>) -> RR {
                self.inner.add_material(material)
            }

            pub fn material_count(&self) -> usize {
                self.inner.material_count()
            }

            pub fn iter_materials(&self) -> impl Iterator<Item = (RR, &Material<SS>)> + '_ {
                self.inner.iter_materials()
            }

            pub fn iter_materials_mut(&mut self) -> impl Iterator<Item = (RR, &mut Material<SS>)> + '_ {
                self.inner.iter_materials_mut()
            }

            pub fn find_material(&self, material: &Material<SS>) -> Option<RR>
            where
                Material<SS>: PartialEq,
            {
                self.inner.find_material(material)
            }

            pub fn remove_material(&mut self, id: RR) -> Option<Material<SS>> {
                self.inner.remove_material(id)
            }

            pub fn clear_materials(&mut self) {
                self.inner.clear_materials();
            }

            pub fn get_or_insert_material(&mut self, material: Material<SS>) -> RR
            where
                Material<SS>: PartialEq,
            {
                self.inner.get_or_insert_material(material)
            }

            // ==================== TEXTURES ====================

            pub fn get_texture(&self, id: RR) -> Option<&Texture<SS>> {
                self.inner.get_texture(id)
            }

            pub fn get_texture_mut(&mut self, id: RR) -> Option<&mut Texture<SS>> {
                self.inner.get_texture_mut(id)
            }

            pub fn add_texture(&mut self, texture: Texture<SS>) -> RR {
                self.inner.add_texture(texture)
            }

            pub fn texture_count(&self) -> usize {
                self.inner.texture_count()
            }

            pub fn iter_textures(&self) -> impl Iterator<Item = (RR, &Texture<SS>)> + '_ {
                self.inner.iter_textures()
            }

            pub fn iter_textures_mut(&mut self) -> impl Iterator<Item = (RR, &mut Texture<SS>)> + '_ {
                self.inner.iter_textures_mut()
            }

            pub fn find_texture(&self, texture: &Texture<SS>) -> Option<RR>
            where
                Texture<SS>: PartialEq,
            {
                self.inner.find_texture(texture)
            }

            pub fn remove_texture(&mut self, id: RR) -> Option<Texture<SS>> {
                self.inner.remove_texture(id)
            }

            pub fn clear_textures(&mut self) {
                self.inner.clear_textures();
            }

            pub fn get_or_insert_texture(&mut self, texture: Texture<SS>) -> RR
            where
                Texture<SS>: PartialEq,
            {
                self.inner.get_or_insert_texture(texture)
            }

            // ==================== GEOMETRIES ====================

            pub fn get_geometry(&self, id: RR) -> Option<&Geometry<VR, RR, SS>> {
                self.inner.get_geometry(id)
            }

            pub fn get_geometry_mut(&mut self, id: RR) -> Option<&mut Geometry<VR, RR, SS>> {
                self.inner.get_geometry_mut(id)
            }

            pub fn add_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR {
                self.inner.add_geometry(geometry)
            }

            pub fn geometry_count(&self) -> usize {
                self.inner.geometry_count()
            }

            pub fn iter_geometries(&self) -> impl Iterator<Item = (RR, &Geometry<VR, RR, SS>)> + '_ {
                self.inner.iter_geometries()
            }

            pub fn iter_geometries_mut(&mut self) -> impl Iterator<Item = (RR, &mut Geometry<VR, RR, SS>)> + '_ {
                self.inner.iter_geometries_mut()
            }

            pub fn remove_geometry(&mut self, id: RR) -> Option<Geometry<VR, RR, SS>> {
                self.inner.remove_geometry(id)
            }

            pub fn clear_geometries(&mut self) {
                self.inner.clear_geometries();
            }

            pub fn vertices(&self) -> &crate::cityjson::core::coordinate::Vertices<VR, $coordinate_type> {
                self.inner.vertices()
            }

            pub fn vertices_mut(&mut self) -> &mut crate::cityjson::core::coordinate::Vertices<VR, $coordinate_type> {
                self.inner.vertices_mut()
            }

            pub fn clear_vertices(&mut self) {
                self.inner.clear_vertices();
            }

            pub fn add_vertex(&mut self, coordinate: $coordinate_type) -> crate::error::Result<crate::cityjson::core::vertex::VertexIndex<VR>> {
                self.inner.add_vertex(coordinate)
            }

            pub fn get_vertex(&self, index: crate::cityjson::core::vertex::VertexIndex<VR>) -> Option<&$coordinate_type> {
                self.inner.get_vertex(index)
            }

            pub fn metadata(&self) -> Option<&$metadata_type> {
                self.inner.metadata()
            }

            pub fn metadata_mut(&mut self) -> &mut $metadata_type {
                self.inner.metadata_mut()
            }

            pub fn extra(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
                self.inner.extra()
            }

            pub fn extra_mut(&mut self) -> &mut crate::cityjson::core::attributes::Attributes<SS> {
                self.inner.extra_mut()
            }

            pub fn transform(&self) -> Option<&Transform> {
                self.inner.transform()
            }

            pub fn transform_mut(&mut self) -> &mut Transform {
                self.inner.transform_mut()
            }

            pub fn extensions(&self) -> Option<&Extensions<SS>> {
                self.inner.extensions()
            }

            pub fn extensions_mut(&mut self) -> &mut Extensions<SS> {
                self.inner.extensions_mut()
            }

            pub fn cityobjects(&self) -> &CityObjects<SS, RR> {
                self.inner.cityobjects()
            }

            pub fn cityobjects_mut(&mut self) -> &mut CityObjects<SS, RR> {
                self.inner.cityobjects_mut()
            }

            pub fn clear_cityobjects(&mut self) {
                self.inner.cityobjects_mut().clear();
            }

            pub fn add_uv_coordinate(&mut self, uvcoordinate: crate::cityjson::core::coordinate::UVCoordinate) -> crate::error::Result<crate::cityjson::core::vertex::VertexIndex<VR>> {
                self.inner.add_uv_coordinate(uvcoordinate)
            }

            pub fn get_uv_coordinate(&self, index: crate::cityjson::core::vertex::VertexIndex<VR>) -> Option<&crate::cityjson::core::coordinate::UVCoordinate> {
                self.inner.get_uv_coordinate(index)
            }

            pub fn vertices_texture(&self) -> &crate::cityjson::core::coordinate::Vertices<VR, crate::cityjson::core::coordinate::UVCoordinate> {
                self.inner.vertices_texture()
            }

            pub fn vertices_texture_mut(&mut self) -> &mut crate::cityjson::core::coordinate::Vertices<VR, crate::cityjson::core::coordinate::UVCoordinate> {
                self.inner.vertices_texture_mut()
            }

            pub fn add_template_vertex(
                &mut self,
                coordinate: crate::prelude::RealWorldCoordinate,
            ) -> crate::error::Result<crate::cityjson::core::vertex::VertexIndex<VR>> {
                self.inner.add_template_vertex(coordinate)
            }

            pub fn get_template_vertex(&self, index: crate::cityjson::core::vertex::VertexIndex<VR>) -> Option<&crate::prelude::RealWorldCoordinate> {
                self.inner.get_template_vertex(index)
            }

            pub fn template_vertices(&self) -> &crate::cityjson::core::coordinate::Vertices<VR, crate::prelude::RealWorldCoordinate> {
                self.inner.template_vertices()
            }

            pub fn template_vertices_mut(&mut self) -> &mut crate::cityjson::core::coordinate::Vertices<VR, crate::prelude::RealWorldCoordinate> {
                self.inner.template_vertices_mut()
            }

            pub fn clear_template_vertices(&mut self) {
                self.inner.clear_template_vertices();
            }

            // ==================== TEMPLATE GEOMETRIES ====================

            pub fn get_template_geometry(&self, id: RR) -> Option<&Geometry<VR, RR, SS>> {
                self.inner.get_template_geometry(id)
            }

            pub fn get_template_geometry_mut(&mut self, id: RR) -> Option<&mut Geometry<VR, RR, SS>> {
                self.inner.get_template_geometry_mut(id)
            }

            pub fn add_template_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR {
                self.inner.add_template_geometry(geometry)
            }

            pub fn template_geometry_count(&self) -> usize {
                self.inner.template_geometry_count()
            }

            pub fn iter_template_geometries(&self) -> impl Iterator<Item = (RR, &Geometry<VR, RR, SS>)> + '_ {
                self.inner.iter_template_geometries()
            }

            pub fn iter_template_geometries_mut(&mut self) -> impl Iterator<Item = (RR, &mut Geometry<VR, RR, SS>)> + '_ {
                self.inner.iter_template_geometries_mut()
            }

            pub fn remove_template_geometry(&mut self, id: RR) -> Option<Geometry<VR, RR, SS>> {
                self.inner.remove_template_geometry(id)
            }

            pub fn clear_template_geometries(&mut self) {
                self.inner.clear_template_geometries();
            }

            // ==================== ATTRIBUTES ====================

            pub fn attributes(&self) -> &crate::cityjson::core::attributes::AttributePool<SS, RR> {
                self.inner.attributes()
            }

            pub fn attributes_mut(&mut self) -> &mut crate::cityjson::core::attributes::AttributePool<SS, RR> {
                self.inner.attributes_mut()
            }

            pub fn attribute_count(&self) -> usize {
                self.inner.attribute_count()
            }

            pub fn has_attributes(&self) -> bool {
                self.inner.has_attributes()
            }

            pub fn clear_attributes(&mut self) {
                self.inner.clear_attributes();
            }

            pub fn type_citymodel(&self) -> crate::CityModelType {
                self.inner.type_citymodel()
            }

            pub fn version(&self) -> Option<crate::CityJSONVersion> {
                self.inner.version()
            }

            pub fn default_theme_material(&self) -> Option<RR> {
                self.inner.default_theme_material()
            }

            pub fn set_default_theme_material(&mut self, material_ref: Option<RR>) {
                self.inner.set_default_theme_material(material_ref);
            }

            pub fn default_theme_texture(&self) -> Option<RR> {
                self.inner.default_theme_texture()
            }

            pub fn set_default_theme_texture(&mut self, texture_ref: Option<RR>) {
                self.inner.set_default_theme_texture(texture_ref);
            }
        }
    };
}
pub(crate) use impl_citymodel_methods;
