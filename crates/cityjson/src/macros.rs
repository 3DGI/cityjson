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

macro_rules! impl_extension_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage>
            crate::cityjson::traits::extension::ExtensionTrait<SS> for Extension<SS>
        {
            fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
                Self {
                    inner: crate::cityjson::core::extension::ExtensionCore::new(name, url, version),
                }
            }

            fn name(&self) -> &SS::String {
                self.inner.name()
            }

            fn url(&self) -> &SS::String {
                self.inner.url()
            }

            fn version(&self) -> &SS::String {
                self.inner.version()
            }
        }
    };
}
pub(crate) use impl_extension_trait;

macro_rules! impl_extensions_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage>
            crate::cityjson::traits::extension::ExtensionsTrait<SS, Extension<SS>>
            for Extensions<SS>
        {
            fn new() -> Self {
                Self {
                    inner: crate::cityjson::core::extension::ExtensionsCore::new(),
                }
            }

            fn add(&mut self, extension: Extension<SS>) -> &mut Self {
                self.inner.add(extension);
                self
            }

            fn remove(&mut self, name: SS::String) -> bool {
                self.inner.remove(name)
            }

            fn get(&self, name: &str) -> Option<&Extension<SS>> {
                self.inner.get(name)
            }

            fn len(&self) -> usize {
                self.inner.len()
            }

            fn is_empty(&self) -> bool {
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
        impl<SS: crate::resources::storage::StringStorage>
            crate::cityjson::traits::appearance::material::MaterialTrait<SS> for Material<SS>
        {
            fn new(name: SS::String) -> Self {
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
            fn name(&self) -> &SS::String {
                &self.name
            }
            #[inline]
            fn set_name(&mut self, name: SS::String) {
                self.name = name;
            }
            #[inline]
            fn ambient_intensity(&self) -> Option<f32> {
                self.ambient_intensity
            }
            #[inline]
            fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>) {
                self.ambient_intensity = ambient_intensity;
            }
            #[inline]
            fn diffuse_color(&self) -> Option<&crate::cityjson::core::appearance::RGB> {
                self.diffuse_color.as_ref()
            }
            #[inline]
            fn set_diffuse_color(
                &mut self,
                diffuse_color: Option<crate::cityjson::core::appearance::RGB>,
            ) {
                self.diffuse_color = diffuse_color;
            }
            #[inline]
            fn emissive_color(&self) -> Option<&crate::cityjson::core::appearance::RGB> {
                self.emissive_color.as_ref()
            }
            #[inline]
            fn set_emissive_color(
                &mut self,
                emissive_color: Option<crate::cityjson::core::appearance::RGB>,
            ) {
                self.emissive_color = emissive_color;
            }
            #[inline]
            fn specular_color(&self) -> Option<&crate::cityjson::core::appearance::RGB> {
                self.specular_color.as_ref()
            }
            #[inline]
            fn set_specular_color(
                &mut self,
                specular_color: Option<crate::cityjson::core::appearance::RGB>,
            ) {
                self.specular_color = specular_color;
            }
            #[inline]
            fn shininess(&self) -> Option<f32> {
                self.shininess
            }
            #[inline]
            fn set_shininess(&mut self, shininess: Option<f32>) {
                self.shininess = shininess;
            }
            #[inline]
            fn transparency(&self) -> Option<f32> {
                self.transparency
            }
            #[inline]
            fn set_transparency(&mut self, transparency: Option<f32>) {
                self.transparency = transparency;
            }
            #[inline]
            fn is_smooth(&self) -> Option<bool> {
                self.is_smooth
            }
            #[inline]
            fn set_is_smooth(&mut self, is_smooth: Option<bool>) {
                self.is_smooth = is_smooth;
            }
        }
    };
}
pub(crate) use impl_material_trait;

macro_rules! impl_texture_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage>
            crate::cityjson::traits::appearance::texture::TextureTrait<SS> for Texture<SS>
        {
            #[inline]
            fn new(
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
            fn image_type(&self) -> &crate::cityjson::core::appearance::ImageType {
                &self.image_type
            }
            #[inline]
            fn set_image_type(&mut self, image_type: crate::cityjson::core::appearance::ImageType) {
                self.image_type = image_type;
            }
            #[inline]
            fn image(&self) -> &SS::String {
                &self.image
            }
            #[inline]
            fn set_image(&mut self, image: SS::String) {
                self.image = image;
            }
            #[inline]
            fn wrap_mode(&self) -> Option<crate::cityjson::core::appearance::WrapMode> {
                self.wrap_mode
            }
            #[inline]
            fn set_wrap_mode(
                &mut self,
                wrap_mode: Option<crate::cityjson::core::appearance::WrapMode>,
            ) {
                self.wrap_mode = wrap_mode;
            }
            #[inline]
            fn texture_type(&self) -> Option<crate::cityjson::core::appearance::TextureType> {
                self.texture_type
            }
            #[inline]
            fn set_texture_type(
                &mut self,
                texture_type: Option<crate::cityjson::core::appearance::TextureType>,
            ) {
                self.texture_type = texture_type;
            }
            #[inline]
            fn border_color(&self) -> Option<crate::cityjson::core::appearance::RGBA> {
                self.border_color
            }
            #[inline]
            fn set_border_color(
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
            crate::cityjson::traits::semantic::SemanticTrait<RR, SS, $semantic_type>
            for Semantic<RR, SS>
        {
            #[inline]
            fn new(type_semantic: $semantic_type) -> Self {
                Self {
                    type_semantic,
                    children: None,
                    parent: None,
                    attributes: None,
                }
            }
            #[inline]
            fn type_semantic(&self) -> &$semantic_type {
                &self.type_semantic
            }
            #[inline]
            fn has_children(&self) -> bool {
                self.children.as_ref().is_some_and(|c| !c.is_empty())
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
            fn attributes(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS, RR>> {
                self.attributes.as_ref()
            }
            #[inline]
            fn attributes_mut(
                &mut self,
            ) -> &mut crate::cityjson::core::attributes::Attributes<SS, RR> {
                if self.attributes.is_none() {
                    self.attributes = Some(crate::cityjson::core::attributes::Attributes::new());
                }
                self.attributes.as_mut().unwrap()
            }
        }
    };
}
pub(crate) use impl_semantic_trait;
