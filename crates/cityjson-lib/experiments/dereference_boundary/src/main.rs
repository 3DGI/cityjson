use crate::with_method::Dereference;

type IVertices = Vec<[i64; 3]>;
pub struct Transform {
    scale: [f64; 3],
    translate: [f64; 3],
}

fn transform_quantized(qc: &[i64; 3], transform: &Transform) -> [f64; 3] {
    [
        qc[0] as f64 * transform.scale[0] + transform.translate[0],
        qc[1] as f64 * transform.scale[1] + transform.translate[1],
        qc[2] as f64 * transform.scale[2] + transform.translate[2],
    ]
}

/// Type aliases with plain functions and static dispatch.
/// I assume this would be the most performant, lowest memory-use option, but also the most
/// verbose in code.
/// I like the clarity of static dispatch.
mod with_func {
    use super::{IVertices, Transform};

    // Target types
    pub type PointBoundary = [f64; 3];
    pub type MultiPointBoundary = Vec<PointBoundary>;
    // ... 11 types in total, similarly constructed with deeper and deeper nesting of vectors

    // Source types
    pub type IVertex = usize;
    pub type IMultiPointBoundary = Vec<IVertex>;

    // Would need 7 dereference_* functions for each type that needs it.
    pub fn dereference_imultipoint(
        imultipoint: &IMultiPointBoundary,
        vertices: &IVertices,
        transform: &Transform,
    ) -> MultiPointBoundary {
        let mut new_multipoint = MultiPointBoundary::with_capacity(imultipoint.len());
        for vtx in imultipoint {
            new_multipoint.push(crate::transform_quantized(&vertices[*vtx], transform));
        }
        new_multipoint
    }
}

/// Type aliases with traits and dynamic dispatch.
/// I find traits neat, but I'm concerned about the dynamic dispatch and the potential performance
/// overhead (or missed optimizations).
/// In addition, I don't actually need Boxed objects. Boxing them will cause an additional heap
/// allocation over the top of the already heap allocated nested vectors.
mod with_trait {
    use super::{IVertices, Transform};

    // Target types
    pub type PointBoundary = [f64; 3];
    pub type MultiPointBoundary = Vec<PointBoundary>;
    // ... 11 types in total, similarly constructed with deeper and deeper nesting of vectors

    pub trait Boundary: std::fmt::Debug {
        fn to_string(&self) -> String {
            format!("{self:?}")
        }
    }
    impl Boundary for MultiPointBoundary {}

    // Source types
    pub type IVertex = usize;
    pub type IMultiPointBoundary = Vec<IVertex>;

    pub trait Dereference {
        fn dereference(&self, vertices: &IVertices, transform: &Transform) -> Box<dyn Boundary>;
    }
    impl Dereference for IMultiPointBoundary {
        fn dereference(&self, vertices: &IVertices, transform: &Transform) -> Box<dyn Boundary> {
            let mut new_multipoint = MultiPointBoundary::with_capacity(self.len());
            for vtx in self {
                new_multipoint.push(crate::transform_quantized(&vertices[*vtx], transform));
            }
            Box::new(new_multipoint)
        }
    }
}

/// Type aliases with traits.
/// I think this is exactly what I'm looking for.
mod with_method {
    use super::{IVertices, Transform};

    // Target types
    pub type PointBoundary = [f64; 3];
    pub type MultiPointBoundary = Vec<PointBoundary>;
    pub type LineStringBoundary = Vec<PointBoundary>;
    pub type MultiLineStringBoundary = Vec<LineStringBoundary>;
    // ... 11 types in total, similarly constructed with deeper and deeper nesting of vectors

    pub trait Boundary: std::fmt::Debug {
        fn to_string(&self) -> String {
            format!("{self:?}")
        }
    }
    impl Boundary for MultiPointBoundary {}
    impl Boundary for MultiLineStringBoundary {}

    // Source types
    pub type IVertex = usize;
    pub type IMultiPointBoundary = Vec<IVertex>;
    pub type ILineStringBoundary = Vec<IVertex>;
    pub type IMultiLineStringBoundary = Vec<ILineStringBoundary>;

    pub trait Dereference<Boundary> {
        fn dereference(&self, vertices: &IVertices, transform: &Transform) -> Boundary;
    }
    impl Dereference<MultiPointBoundary> for IMultiPointBoundary {
        fn dereference(&self, vertices: &IVertices, transform: &Transform) -> MultiPointBoundary {
            let mut new_multipoint = MultiPointBoundary::with_capacity(self.len());
            for vtx in self {
                new_multipoint.push(crate::transform_quantized(&vertices[*vtx], transform));
            }
            new_multipoint
        }
    }
    impl Dereference<MultiLineStringBoundary> for IMultiLineStringBoundary {
        fn dereference(
            &self,
            vertices: &IVertices,
            transform: &Transform,
        ) -> MultiLineStringBoundary {
            let mut new_multilinestring = MultiLineStringBoundary::with_capacity(self.len());
            for linestring in self {
                let mut new_linestring = LineStringBoundary::with_capacity(linestring.len());
                for vtx in linestring {
                    new_linestring.push(crate::transform_quantized(&vertices[*vtx], transform));
                }
                new_multilinestring.push(new_linestring);
            }
            new_multilinestring
        }
    }
}

/// Type aliases bundled into an enum.
/// I prefer this over 'with_traits', but I'm concerned about the size of a Boundary, because of its
/// largest variant. The program will need to handle many, many Boundaries and the memory use is an
/// issue.
mod with_enum {
    use super::{IVertices, Transform};

    // Target types
    pub type PointBoundary = [f64; 3];
    pub type MultiPointBoundary = Vec<PointBoundary>;
    // ... there are 11 types in total, with increasing complexity (nesting)

    #[derive(Debug)]
    pub enum Boundary {
        MultiPoint(MultiPointBoundary),
        // There are 7 variants in total (not each type alias needs a variant).
        // The largest variant is Vec<Vec<Vec<Vec<Vec<[f64; 3]>>>>>.
        // Using an enum, each of its variants would use 6 * 24 bytes (5 * Vec + [f64; 3]), while a
        // MultiPoint would only require 2 * 24 bytes.
    }

    // Source types
    pub type IVertex = usize;
    pub type IMultiPointBoundary = Vec<IVertex>;

    pub enum IBoundary {
        MultiPoint(IMultiPointBoundary),
    }

    impl IBoundary {
        pub fn dereference(&self, vertices: &IVertices, transform: &Transform) -> Boundary {
            match self {
                IBoundary::MultiPoint(imultipoint) => {
                    let mut new_multipoint = MultiPointBoundary::with_capacity(imultipoint.len());
                    for vtx in imultipoint {
                        new_multipoint.push(crate::transform_quantized(&vertices[*vtx], transform));
                    }
                    Boundary::MultiPoint(new_multipoint)
                }
            }
        }
    }
}

fn main() {
    let vertices: IVertices = vec![[-10, -10, -10], [10, 10, 10], [20, 20, 20]];
    let transform = Transform {
        scale: [0.001, 0.001, 0.001],
        translate: [0.0, 0.0, 0.0],
    };

    // with_funcs
    let imp = with_func::IMultiPointBoundary::from([2, 1, 0]);
    let mp = with_func::dereference_imultipoint(&imp, &vertices, &transform);
    println!("MultiPointBoundary: {:?}", mp);

    // with_trait
    let imp = with_trait::IMultiPointBoundary::from([2, 1, 0]);
    // very clunky way of calling the method, just because there are overlapping names in the
    // with_trait and with_method modules
    let mp = <with_trait::IMultiPointBoundary as with_trait::Dereference>::dereference(
        &imp, &vertices, &transform,
    );
    println!("MultiPointBoundary: {:?}", mp);

    // with_method
    let imp = with_method::IMultiPointBoundary::from([2, 1, 0]);
    let mp = imp.dereference(&vertices, &transform);
    println!("MultiPointBoundary: {:?}", mp);
    let iml: with_method::IMultiLineStringBoundary = vec![imp];
    let ml = iml.dereference(&vertices, &transform);
    println!("MultiLineStringBoundary: {:?}", ml);

    // with_enum
    let imp = with_enum::IBoundary::MultiPoint(with_enum::IMultiPointBoundary::from([2, 1, 0]));
    let mp = imp.dereference(&vertices, &transform);
    println!("MultiPointBoundary: {:?}", mp);
}
