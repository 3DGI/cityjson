use cityjson_lib::cityjson::resources::handles::{
    GeometryTemplateHandle, MaterialHandle, SemanticHandle, TextureHandle,
};
use cityjson_lib::cityjson::resources::storage::OwnedStringStorage;
use cityjson_lib::cityjson::v2_0::ThemeName;
use cityjson_lib::cityjson::v2_0::appearance::material::Material;
use cityjson_lib::cityjson::v2_0::appearance::texture::Texture;
use cityjson_lib::cityjson::v2_0::attributes::AttributeValue;
use cityjson_lib::cityjson::v2_0::coordinate::{RealWorldCoordinate, UVCoordinate};
use cityjson_lib::cityjson::v2_0::geometry::semantic::Semantic;
use cityjson_lib::cityjson::v2_0::geometry::{AffineTransform3D, LoD};
use cityjson_lib::cityjson::v2_0::geometry_draft::{
    GeometryDraft, LineStringDraft, PointDraft, RingDraft, ShellDraft, SolidDraft, SurfaceDraft,
    UvDraft, VertexDraft,
};
use cityjson_lib::cityjson::v2_0::metadata::Contact;
use cityjson_lib::cityjson::v2_0::vertex::VertexIndex;
use cityjson_lib::cityjson::v2_0::{CityObject, GeometryType};

pub type OwnedValue = AttributeValue<OwnedStringStorage>;
pub type OwnedContact = Contact<OwnedStringStorage>;
pub type OwnedMaterial = Material<OwnedStringStorage>;
pub type OwnedTexture = Texture<OwnedStringStorage>;
pub type OwnedSemantic = Semantic<OwnedStringStorage>;
pub type OwnedCityObject = CityObject<OwnedStringStorage>;
pub type OwnedGeometryDraft = GeometryDraft<u32, OwnedStringStorage>;

#[derive(Debug, Clone, PartialEq)]
pub enum VertexAuthoring {
    Existing(VertexIndex<u32>),
    New(RealWorldCoordinate),
}

impl VertexAuthoring {
    pub fn into_draft(self) -> VertexDraft<u32> {
        match self {
            Self::Existing(index) => VertexDraft::Existing(index),
            Self::New(vertex) => VertexDraft::New(vertex),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UvAuthoring {
    Existing(VertexIndex<u32>),
    New(UVCoordinate),
}

impl UvAuthoring {
    pub fn into_draft(self) -> UvDraft<u32> {
        match self {
            Self::Existing(index) => UvDraft::Existing(index),
            Self::New(uv) => UvDraft::New(uv),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RingTextureAuthoring {
    pub theme: String,
    pub texture: TextureHandle,
    pub uvs: Vec<UvAuthoring>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RingAuthoring {
    pub vertices: Vec<VertexAuthoring>,
    pub textures: Vec<RingTextureAuthoring>,
}

impl RingAuthoring {
    pub fn into_draft(self) -> RingDraft<u32, OwnedStringStorage> {
        let mut ring = RingDraft::new(self.vertices.into_iter().map(VertexAuthoring::into_draft));
        for texture in self.textures {
            ring = ring.with_texture(
                ThemeName::new(texture.theme),
                texture.texture,
                texture.uvs.into_iter().map(UvAuthoring::into_draft),
            );
        }
        ring
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceAuthoring {
    pub outer: RingAuthoring,
    pub inners: Vec<RingAuthoring>,
    pub semantic: Option<SemanticHandle>,
    pub materials: Vec<(String, MaterialHandle)>,
}

impl SurfaceAuthoring {
    pub fn into_draft(self) -> SurfaceDraft<u32, OwnedStringStorage> {
        let mut surface = SurfaceDraft::new(
            self.outer.into_draft(),
            self.inners.into_iter().map(RingAuthoring::into_draft),
        );
        if let Some(semantic) = self.semantic {
            surface = surface.with_semantic(semantic);
        }
        for (theme, material) in self.materials {
            surface = surface.with_material(ThemeName::new(theme), material);
        }
        surface
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellAuthoring {
    pub surfaces: Vec<SurfaceAuthoring>,
}

impl ShellAuthoring {
    pub fn into_draft(self) -> ShellDraft<u32, OwnedStringStorage> {
        ShellDraft::new(self.surfaces.into_iter().map(SurfaceAuthoring::into_draft))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolidAuthoring {
    pub outer: ShellAuthoring,
    pub inners: Vec<ShellAuthoring>,
}

impl SolidAuthoring {
    pub fn into_draft(self) -> SolidDraft<u32, OwnedStringStorage> {
        SolidDraft::new(
            self.outer.into_draft(),
            self.inners.into_iter().map(ShellAuthoring::into_draft),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointAuthoring {
    pub vertex: VertexAuthoring,
    pub semantic: Option<SemanticHandle>,
}

impl PointAuthoring {
    pub fn into_draft(self) -> PointDraft<u32> {
        if let Some(semantic) = self.semantic {
            PointDraft::new(self.vertex.into_draft()).with_semantic(semantic)
        } else {
            PointDraft::new(self.vertex.into_draft())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineStringAuthoring {
    pub vertices: Vec<VertexAuthoring>,
    pub semantic: Option<SemanticHandle>,
}

impl LineStringAuthoring {
    pub fn into_draft(self) -> LineStringDraft<u32> {
        if let Some(semantic) = self.semantic {
            LineStringDraft::new(self.vertices.into_iter().map(VertexAuthoring::into_draft))
                .with_semantic(semantic)
        } else {
            LineStringDraft::new(self.vertices.into_iter().map(VertexAuthoring::into_draft))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeometryAuthoring {
    MultiPoint {
        lod: Option<LoD>,
        points: Vec<PointAuthoring>,
    },
    MultiLineString {
        lod: Option<LoD>,
        linestrings: Vec<LineStringAuthoring>,
    },
    MultiSurface {
        lod: Option<LoD>,
        surfaces: Vec<SurfaceAuthoring>,
    },
    CompositeSurface {
        lod: Option<LoD>,
        surfaces: Vec<SurfaceAuthoring>,
    },
    Solid {
        lod: Option<LoD>,
        solid: Option<SolidAuthoring>,
    },
    MultiSolid {
        lod: Option<LoD>,
        solids: Vec<SolidAuthoring>,
    },
    CompositeSolid {
        lod: Option<LoD>,
        solids: Vec<SolidAuthoring>,
    },
    GeometryInstance {
        template: GeometryTemplateHandle,
        reference_point: VertexAuthoring,
        transformation: AffineTransform3D,
    },
}

impl GeometryAuthoring {
    pub fn new(geometry_type: GeometryType, lod: Option<LoD>) -> Self {
        match geometry_type {
            GeometryType::MultiPoint => Self::MultiPoint {
                lod,
                points: Vec::new(),
            },
            GeometryType::MultiLineString => Self::MultiLineString {
                lod,
                linestrings: Vec::new(),
            },
            GeometryType::MultiSurface => Self::MultiSurface {
                lod,
                surfaces: Vec::new(),
            },
            GeometryType::CompositeSurface => Self::CompositeSurface {
                lod,
                surfaces: Vec::new(),
            },
            GeometryType::Solid => Self::Solid { lod, solid: None },
            GeometryType::MultiSolid => Self::MultiSolid {
                lod,
                solids: Vec::new(),
            },
            GeometryType::CompositeSolid => Self::CompositeSolid {
                lod,
                solids: Vec::new(),
            },
            GeometryType::GeometryInstance => unreachable!("instance uses dedicated constructor"),
            _ => unreachable!("unsupported geometry type"),
        }
    }

    pub fn instance(
        template: GeometryTemplateHandle,
        reference_point: VertexAuthoring,
        transformation: AffineTransform3D,
    ) -> Self {
        Self::GeometryInstance {
            template,
            reference_point,
            transformation,
        }
    }

    pub fn into_draft(self) -> Option<OwnedGeometryDraft> {
        Some(match self {
            Self::MultiPoint { lod, points } => {
                GeometryDraft::multi_point(lod, points.into_iter().map(PointAuthoring::into_draft))
            }
            Self::MultiLineString { lod, linestrings } => GeometryDraft::multi_line_string(
                lod,
                linestrings.into_iter().map(LineStringAuthoring::into_draft),
            ),
            Self::MultiSurface { lod, surfaces } => GeometryDraft::multi_surface(
                lod,
                surfaces.into_iter().map(SurfaceAuthoring::into_draft),
            ),
            Self::CompositeSurface { lod, surfaces } => GeometryDraft::composite_surface(
                lod,
                surfaces.into_iter().map(SurfaceAuthoring::into_draft),
            ),
            Self::Solid { lod, solid } => {
                let solid = solid?;
                GeometryDraft::solid(
                    lod,
                    solid.outer.into_draft(),
                    solid.inners.into_iter().map(ShellAuthoring::into_draft),
                )
            }
            Self::MultiSolid { lod, solids } => {
                GeometryDraft::multi_solid(lod, solids.into_iter().map(SolidAuthoring::into_draft))
            }
            Self::CompositeSolid { lod, solids } => GeometryDraft::composite_solid(
                lod,
                solids.into_iter().map(SolidAuthoring::into_draft),
            ),
            Self::GeometryInstance {
                template,
                reference_point,
                transformation,
            } => GeometryDraft::instance(template, reference_point.into_draft(), transformation),
        })
    }
}
