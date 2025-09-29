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
