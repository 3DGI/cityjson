//! WKT and EWKT conversion for flattened `CityJSON` boundaries.
//!
//! This module converts [`Boundary`] and [`Vertices`] directly, without an
//! intermediate geometry object, external codec, or WKB buffer. An offset starts a
//! child whose end is the next offset or the child buffer's length. Each `rings`
//! range therefore becomes a coordinate-sequence parenthesis level, each `surfaces`
//! range becomes the ring list inside a polygon, each `shells` range selects an
//! ordered list of surfaces, and each `solids` range selects ordered shells.
//! `MultiPoint` uses the vertex-reference buffer directly; every ring range becomes
//! one `MultiLineString` member.
//!
//! Surface boundaries map to `MULTIPOLYGON Z`. Since ISO WKT has no solid, solids
//! flatten by ordered `solids -> shells -> surfaces` traversal (or
//! `shells -> surfaces` for one solid), matching [`super::wkb`]'s polygon logic.
//! EWKT takes an explicit [`EwktType`] so surface-backed boundaries can instead use
//! `MULTIPOLYGON`, `POLYHEDRALSURFACE`, or `TIN` framing.
//!
//! This module and [`super::wkb`] share topology traversal, ring closing/opening,
//! ordering, repeated-reference handling, target-boundary reconstruction, and
//! validation semantics. Only framing and coordinate encoding differ: WKT uses
//! parentheses and finite decimal XYZ; WKB uses counted records and `f64` bytes.
//!
//! ISO WKT uses explicit `Z`. EWKT uses an optional top-level `SRID=<number>;` and
//! `PostGIS` XYZ without `Z`. Parsing is case-insensitive and accepts wrapped
//! `MULTIPOINT` coordinates (`((x y z), ...)`) and unwrapped coordinates
//! (`(x y z, ...)`). It rejects non-finite coordinates, malformed text, `EMPTY`,
//! XY/M/ZM dimensions, singular types, invalid rings, and trailing text.
//!
//! # Example
//!
//! ```
//! use cityjson_types::v2_0::{Boundary, BoundaryType};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let text = "MULTIPOLYGON Z (((0 0 0,1 0 0,0 1 0,0 0 0)))";
//!     let (boundary, vertices) = Boundary::<u32>::from_wkt(text)?;
//!     assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSurface);
//!     assert_eq!(
//!         boundary.to_nested_multi_or_composite_surface()?,
//!         vec![vec![vec![0, 1, 2]]],
//!     );
//!     assert_eq!(boundary.to_wkt(&vertices)?, text);
//!     Ok(())
//! }
//! ```

use super::{Boundary, BoundaryType};
use crate::cityjson::core::coordinate::RealWorldCoordinate;
use crate::cityjson::core::vertex::{VertexIndex, VertexRef};
use crate::cityjson::core::vertices::Vertices;
use crate::error;

/// EWKT geometry type written or parsed by this module.
#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub enum EwktType {
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    PolyhedralSurface,
    Tin,
}
impl std::fmt::Display for EwktType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MultiPoint => "MultiPoint",
            Self::MultiLineString => "MultiLineString",
            Self::MultiPolygon => "MultiPolygon",
            Self::PolyhedralSurface => "PolyhedralSurface",
            Self::Tin => "TIN",
        })
    }
}

/// Boundary, coordinates, type, and optional SRID parsed from EWKT.
#[derive(Clone, Debug)]
pub struct EwktBoundary<VR: VertexRef> {
    pub boundary: Boundary<VR>,
    pub vertices: Vertices<VR, RealWorldCoordinate>,
    pub ewkt_type: EwktType,
    pub srid: Option<u32>,
}

impl<VR: VertexRef> Boundary<VR> {
    /// Converts this boundary to finite XYZ ISO WKT.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid boundaries, references, rings, or coordinates.
    pub fn to_wkt(&self, vertices: &Vertices<VR, RealWorldCoordinate>) -> error::Result<String> {
        write(self, vertices, None, None)
    }
    /// Parses explicit-`Z` ISO WKT.
    ///
    /// # Errors
    ///
    /// Returns an error for unsupported or malformed WKT.
    pub fn from_wkt(text: &str) -> error::Result<(Self, Vertices<VR, RealWorldCoordinate>)> {
        parse(text, true, BoundaryType::MultiOrCompositeSurface).map(|v| (v.boundary, v.vertices))
    }
    /// Converts this boundary to EWKT, optionally including a top-level SRID.
    ///
    /// # Errors
    ///
    /// Returns an error for incompatible or invalid boundaries.
    pub fn to_ewkt(
        &self,
        vertices: &Vertices<VR, RealWorldCoordinate>,
        kind: EwktType,
        srid: Option<u32>,
    ) -> error::Result<String> {
        write(self, vertices, Some(kind), srid)
    }
    /// Parses EWKT, wrapping surface-like input as `target_boundary_type`.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed EWKT or an incompatible target boundary.
    pub fn from_ewkt(
        text: &str,
        target_boundary_type: BoundaryType,
    ) -> error::Result<EwktBoundary<VR>> {
        parse(text, false, target_boundary_type)
    }
}

/// Serializes a boundary using the selected ISO WKT or EWKT dialect.
fn write<VR: VertexRef>(
    b: &Boundary<VR>,
    v: &Vertices<VR, RealWorldCoordinate>,
    selected: Option<EwktType>,
    srid: Option<u32>,
) -> error::Result<String> {
    if !b.is_consistent() {
        return Err(invalid("inconsistent boundary offsets"));
    }
    let boundary_type = b.check_type();
    let kind = selected.unwrap_or(match boundary_type {
        BoundaryType::MultiPoint => EwktType::MultiPoint,
        BoundaryType::MultiLineString => EwktType::MultiLineString,
        BoundaryType::MultiOrCompositeSurface
        | BoundaryType::Solid
        | BoundaryType::MultiOrCompositeSolid => EwktType::MultiPolygon,
        BoundaryType::None => return Err(invalid("empty boundary")),
    });
    let compatible = match kind {
        EwktType::MultiPoint => boundary_type == BoundaryType::MultiPoint,
        EwktType::MultiLineString => boundary_type == BoundaryType::MultiLineString,
        _ => matches!(
            boundary_type,
            BoundaryType::MultiOrCompositeSurface
                | BoundaryType::Solid
                | BoundaryType::MultiOrCompositeSolid
        ),
    };
    if !compatible {
        return Err(error::Error::IncompatibleBoundary(
            boundary_type.to_string(),
            BoundaryType::MultiOrCompositeSurface.to_string(),
        ));
    }
    let mut out = srid.map_or_else(String::new, |s| format!("SRID={s};"));
    let name = match kind {
        EwktType::MultiPoint => "MULTIPOINT",
        EwktType::MultiLineString => "MULTILINESTRING",
        EwktType::MultiPolygon => "MULTIPOLYGON",
        EwktType::PolyhedralSurface => "POLYHEDRALSURFACE",
        EwktType::Tin => "TIN",
    };
    out.push_str(name);
    if selected.is_none() {
        out.push_str(" Z");
    }
    out.push(' ');
    match kind {
        EwktType::MultiPoint => {
            out.push('(');
            for (i, index) in b.vertices.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('(');
                coord(&mut out, v, *index)?;
                out.push(')');
            }
            out.push(')');
        }
        EwktType::MultiLineString => {
            out.push('(');
            for i in 0..b.rings.len() {
                if i > 0 {
                    out.push(',');
                }
                sequence(
                    &mut out,
                    v,
                    &b.vertices[part(&b.rings, b.vertices.len(), i)?],
                    false,
                )?;
            }
            out.push(')');
        }
        _ => {
            let surfaces = flattened_surfaces(b)?;
            out.push('(');
            for (i, s) in surfaces.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let rings = part(&b.surfaces, b.rings.len(), s)?;
                if rings.is_empty() {
                    return Err(invalid("polygon without rings"));
                }
                if kind == EwktType::Tin && rings.len() != 1 {
                    return Err(invalid("TIN triangle has multiple rings"));
                }
                out.push('(');
                for (j, r) in rings.enumerate() {
                    if j > 0 {
                        out.push(',');
                    }
                    let ring = &b.vertices[part(&b.rings, b.vertices.len(), r)?];
                    if kind == EwktType::Tin && closed_len(ring)? != 4 {
                        return Err(invalid("TIN triangle has invalid ring"));
                    }
                    sequence(&mut out, v, ring, true)?;
                }
                out.push(')');
            }
            out.push(')');
        }
    }
    Ok(out)
}
/// Returns surface indices in their flattened `CityJSON` traversal order.
fn flattened_surfaces<VR: VertexRef>(b: &Boundary<VR>) -> error::Result<Vec<usize>> {
    let mut out = Vec::new();
    match b.check_type() {
        BoundaryType::MultiOrCompositeSurface => out.extend(0..b.surfaces.len()),
        BoundaryType::Solid => {
            for i in 0..b.shells.len() {
                out.extend(part(&b.shells, b.surfaces.len(), i)?);
            }
        }
        BoundaryType::MultiOrCompositeSolid => {
            for i in 0..b.solids.len() {
                for shell in part(&b.solids, b.shells.len(), i)? {
                    out.extend(part(&b.shells, b.surfaces.len(), shell)?);
                }
            }
        }
        _ => {}
    }
    if out.is_empty() {
        Err(invalid("no reachable polygons"))
    } else {
        Ok(out)
    }
}
/// Resolves one implicit-end offset entry into a child range.
fn part<VR: VertexRef>(
    offsets: &[VertexIndex<VR>],
    len: usize,
    i: usize,
) -> error::Result<std::ops::Range<usize>> {
    Ok(offsets[i].try_to_usize()?
        ..offsets
            .get(i + 1)
            .map(VertexIndex::try_to_usize)
            .transpose()?
            .unwrap_or(len))
}
/// Validates a `CityJSON` ring and returns its serialized closed length.
fn closed_len<VR: VertexRef>(ring: &[VertexIndex<VR>]) -> error::Result<usize> {
    if ring.len() < 3 {
        Err(error::Error::InvalidRing {
            reason: "ring needs three vertices".to_owned(),
            vertex_count: ring.len(),
        })
    } else {
        Ok(ring.len() + usize::from(ring.first() != ring.last() || ring.len() == 3))
    }
}
/// Writes one coordinate sequence, optionally appending its closing coordinate.
fn sequence<VR: VertexRef>(
    out: &mut String,
    vertices: &Vertices<VR, RealWorldCoordinate>,
    ring: &[VertexIndex<VR>],
    close: bool,
) -> error::Result<()> {
    if ring.is_empty() {
        return Err(invalid("empty coordinate sequence"));
    }
    let append = close && (ring.first() != ring.last() || ring.len() == 3);
    if close {
        let _ = closed_len(ring)?;
    }
    out.push('(');
    for (i, index) in ring.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        coord(out, vertices, *index)?;
    }
    if append {
        out.push(',');
        coord(out, vertices, ring[0])?;
    }
    out.push(')');
    Ok(())
}
/// Resolves and appends one finite XYZ coordinate.
fn coord<VR: VertexRef>(
    out: &mut String,
    vertices: &Vertices<VR, RealWorldCoordinate>,
    index: VertexIndex<VR>,
) -> error::Result<()> {
    let c = vertices
        .get(index)
        .ok_or_else(|| error::Error::InvalidReference {
            element_type: "vertex".to_owned(),
            index: index.to_usize(),
            max_index: vertices.len().saturating_sub(1),
        })?;
    for (i, n) in [c.x(), c.y(), c.z()].into_iter().enumerate() {
        if !n.is_finite() {
            return Err(invalid("non-finite coordinate"));
        }
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&n.to_string());
    }
    Ok(())
}

/// Parses ISO WKT or EWKT and rebuilds a boundary plus vertex pool.
fn parse<VR: VertexRef>(
    text: &str,
    iso: bool,
    target: BoundaryType,
) -> error::Result<EwktBoundary<VR>> {
    let (srid, text) = if let Some(rest) = text.strip_prefix("SRID=") {
        let (s, rest) = rest
            .split_once(';')
            .ok_or_else(|| invalid("invalid SRID"))?;
        (Some(s.parse().map_err(|_| invalid("invalid SRID"))?), rest)
    } else {
        (None, text)
    };
    if iso && srid.is_some() {
        return Err(invalid("ISO WKT does not allow SRID"));
    }
    let upper = text.trim().to_ascii_uppercase();
    let (kind, body) = [
        ("MULTIPOINT", EwktType::MultiPoint),
        ("MULTILINESTRING", EwktType::MultiLineString),
        ("MULTIPOLYGON", EwktType::MultiPolygon),
        ("POLYHEDRALSURFACE", EwktType::PolyhedralSurface),
        ("TIN", EwktType::Tin),
    ]
    .into_iter()
    .find_map(|(n, k)| upper.strip_prefix(n).map(|b| (k, b)))
    .ok_or_else(|| invalid("unsupported WKT geometry"))?;
    let body = body.trim();
    let body = if let Some(body) = body.strip_prefix('Z') {
        body.trim()
    } else if iso {
        return Err(invalid("ISO WKT requires Z"));
    } else {
        body
    };
    if body.contains(" EMPTY") || body == "EMPTY" {
        return Err(invalid("EMPTY is not supported"));
    }
    let mut p = TextParser::new(body);
    let (mut b, vertices) = match kind {
        EwktType::MultiPoint => p.points()?,
        EwktType::MultiLineString => p.lines()?,
        _ => p.polygons(kind == EwktType::Tin)?,
    };
    p.end()?;
    let target = if iso {
        match kind {
            EwktType::MultiPoint => BoundaryType::MultiPoint,
            EwktType::MultiLineString => BoundaryType::MultiLineString,
            _ => BoundaryType::MultiOrCompositeSurface,
        }
    } else {
        target
    };
    match kind {
        EwktType::MultiPoint if target != BoundaryType::MultiPoint => {
            return Err(error::Error::IncompatibleBoundary(
                BoundaryType::MultiPoint.to_string(),
                target.to_string(),
            ));
        }
        EwktType::MultiLineString if target != BoundaryType::MultiLineString => {
            return Err(error::Error::IncompatibleBoundary(
                BoundaryType::MultiLineString.to_string(),
                target.to_string(),
            ));
        }
        EwktType::MultiPolygon | EwktType::PolyhedralSurface | EwktType::Tin => match target {
            BoundaryType::MultiOrCompositeSurface => {}
            BoundaryType::Solid => b.shells.push(VertexIndex::try_from(0)?),
            BoundaryType::MultiOrCompositeSolid => {
                b.shells.push(VertexIndex::try_from(0)?);
                b.solids.push(VertexIndex::try_from(0)?);
            }
            _ => {
                return Err(error::Error::IncompatibleBoundary(
                    BoundaryType::MultiOrCompositeSurface.to_string(),
                    target.to_string(),
                ));
            }
        },
        _ => {}
    }
    Ok(EwktBoundary {
        boundary: b,
        vertices,
        ewkt_type: kind,
        srid,
    })
}
struct TextParser<'a> {
    s: &'a [u8],
    p: usize,
}
impl<'a> TextParser<'a> {
    /// Creates a parser over the geometry body after its header.
    fn new(s: &'a str) -> Self {
        Self {
            s: s.as_bytes(),
            p: 0,
        }
    }
    /// Consumes ASCII whitespace.
    fn ws(&mut self) {
        while self.s.get(self.p).is_some_and(u8::is_ascii_whitespace) {
            self.p += 1;
        }
    }
    /// Consumes a punctuation byte when it appears after optional whitespace.
    fn take(&mut self, c: u8) -> bool {
        self.ws();
        if self.s.get(self.p) == Some(&c) {
            self.p += 1;
            true
        } else {
            false
        }
    }
    /// Requires a punctuation byte at the current parser position.
    fn need(&mut self, c: u8) -> error::Result<()> {
        if self.take(c) {
            Ok(())
        } else {
            Err(invalid("malformed WKT"))
        }
    }
    /// Parses one finite floating-point coordinate component.
    fn number(&mut self) -> error::Result<f64> {
        self.ws();
        let start = self.p;
        while self
            .s
            .get(self.p)
            .is_some_and(|b| !b.is_ascii_whitespace() && !matches!(*b, b',' | b')' | b'('))
        {
            self.p += 1;
        }
        let n: f64 = std::str::from_utf8(&self.s[start..self.p])
            .ok()
            .and_then(|x| x.parse().ok())
            .ok_or_else(|| invalid("invalid coordinate"))?;
        if n.is_finite() {
            Ok(n)
        } else {
            Err(invalid("non-finite coordinate"))
        }
    }
    /// Parses one XYZ coordinate tuple.
    fn c(&mut self) -> error::Result<RealWorldCoordinate> {
        Ok(RealWorldCoordinate::new(
            self.number()?,
            self.number()?,
            self.number()?,
        ))
    }
    /// Parses either supported `MULTIPOINT` coordinate spelling.
    fn points<VR: VertexRef>(
        &mut self,
    ) -> error::Result<(Boundary<VR>, Vertices<VR, RealWorldCoordinate>)> {
        self.need(b'(')?;
        let mut b = Boundary::new();
        let mut v = Vertices::new();
        loop {
            let wrap = self.take(b'(');
            b.vertices.push(v.push(self.c()?)?);
            if wrap {
                self.need(b')')?;
            }
            if !self.take(b',') {
                break;
            }
        }
        self.need(b')')?;
        Ok((b, v))
    }
    /// Parses all line-string coordinate sequences.
    fn lines<VR: VertexRef>(
        &mut self,
    ) -> error::Result<(Boundary<VR>, Vertices<VR, RealWorldCoordinate>)> {
        self.need(b'(')?;
        let mut b = Boundary::new();
        let mut v = Vertices::new();
        loop {
            self.need(b'(')?;
            b.rings.push(VertexIndex::try_from(b.vertices.len())?);
            loop {
                b.vertices.push(v.push(self.c()?)?);
                if !self.take(b',') {
                    break;
                }
            }
            self.need(b')')?;
            if !self.take(b',') {
                break;
            }
        }
        self.need(b')')?;
        Ok((b, v))
    }
    /// Parses polygon rings and validates optional TIN triangle topology.
    fn polygons<VR: VertexRef>(
        &mut self,
        tin: bool,
    ) -> error::Result<(Boundary<VR>, Vertices<VR, RealWorldCoordinate>)> {
        self.need(b'(')?;
        let mut b = Boundary::new();
        let mut v = Vertices::new();
        loop {
            self.need(b'(')?;
            b.surfaces.push(VertexIndex::try_from(b.rings.len())?);
            let mut nr = 0;
            loop {
                self.need(b'(')?;
                b.rings.push(VertexIndex::try_from(b.vertices.len())?);
                let mut x = Vec::new();
                loop {
                    x.push(self.c()?);
                    if !self.take(b',') {
                        break;
                    }
                }
                self.need(b')')?;
                if x.len() < 4 || x.first() != x.last() {
                    return Err(error::Error::InvalidRing {
                        reason: "WKT polygon ring must be closed".to_owned(),
                        vertex_count: x.len(),
                    });
                }
                if tin && x.len() != 4 {
                    return Err(invalid("invalid TIN triangle"));
                }
                for c in &x[..x.len() - 1] {
                    b.vertices.push(v.push(*c)?);
                }
                nr += 1;
                if !self.take(b',') {
                    break;
                }
            }
            self.need(b')')?;
            if tin && nr != 1 {
                return Err(invalid("invalid TIN triangle"));
            }
            if !self.take(b',') {
                break;
            }
        }
        self.need(b')')?;
        Ok((b, v))
    }
    /// Rejects non-whitespace trailing input.
    fn end(&mut self) -> error::Result<()> {
        self.ws();
        if self.p == self.s.len() {
            Ok(())
        } else {
            Err(invalid("trailing input"))
        }
    }
}
/// Builds a contextual invalid-WKT geometry error.
fn invalid(message: impl Into<String>) -> error::Error {
    error::Error::InvalidGeometry(format!("invalid WKT: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Purpose: Canonical ISO WKT round-trips.
    /// Inputs: A two-point MULTIPOINT Z.
    /// Assertions: The writer retains canonical text.
    #[test]
    fn point_round_trip() {
        let text = "MULTIPOINT Z ((0 0 0),(1 2 3))";
        let (boundary, vertices) = Boundary::<u32>::from_wkt(text).unwrap();
        assert_eq!(boundary.to_wkt(&vertices).unwrap(), text);
    }

    /// Purpose: Alternate multi-point syntax is accepted.
    /// Inputs: Unwrapped lower-case coordinates.
    /// Assertions: The result normalizes canonically.
    #[test]
    fn alternate_points() {
        let (boundary, vertices) = Boundary::<u32>::from_wkt("multipoint z (0 0 0,1 2 3)").unwrap();
        assert_eq!(
            boundary.to_wkt(&vertices).unwrap(),
            "MULTIPOINT Z ((0 0 0),(1 2 3))"
        );
    }

    /// Purpose: EWKT retains SRID/type.
    /// Inputs: `PolyhedralSurface` EWKT with SRID.
    /// Assertions: Parsed fields equal input metadata.
    #[test]
    fn ewkt_metadata() {
        let parsed = Boundary::<u32>::from_ewkt(
            "SRID=7415;POLYHEDRALSURFACE(((0 0 0,1 0 0,0 1 0,0 0 0)))",
            BoundaryType::MultiOrCompositeSurface,
        )
        .unwrap();
        assert_eq!(parsed.srid, Some(7415));
        assert_eq!(parsed.ewkt_type, EwktType::PolyhedralSurface);
    }
}
