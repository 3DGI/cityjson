pub trait TransformTrait {
    fn new() -> Self;
    fn scale(&self) -> [f64; 3];
    fn translate(&self) -> [f64; 3];
    fn set_scale(&mut self, scale: [f64; 3]);
    fn set_translate(&mut self, translate: [f64; 3]);
}
