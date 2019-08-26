// Copyright 2016-2019 Uber Technologies, Inc.
// Copyright 2019      Bhaskar Mookerji
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # `h3-rs`: h3 bindings for Rust
//!
//! Rust bindings to [H3][h3_uber], a C library for hexagonal, hierarchical
//! geospatial indexing. `h3-rs` interoperates with core geospatial data types
//! defined by [`geo-types`][geotypes_rust] crate.
//!
//! This is **experimental, in-progress software**.
//!
//! [h3_uber]: https://uber.github.io/h3/#/
//! [geotypes_rust]: https://crates.io/crates/geo-types

#[macro_use]
extern crate enum_primitive_derive;
extern crate c_vec;
extern crate geo_types;
extern crate num_traits;

use c_vec::CVec;
use geo_types::Coordinate;
use h3_sys;
use num_traits::FromPrimitive;
use std::ffi::CString;

pub use geo_types::{LineString, MultiPolygon, Point, Polygon};

// Alias sys types
struct GeoCoord(h3_sys::GeoCoord);
struct GeoBoundary(h3_sys::GeoBoundary);
struct GeoFence(h3_sys::Geofence);
struct GeoPolygon(h3_sys::GeoPolygon);
struct GeoMultiPolygon(h3_sys::GeoMultiPolygon);

/// A unique hierarchical index for an H3 cell
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct H3Index(pub h3_sys::H3Index);

/// `h3-rs`-specific errors
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The integer value is not valid as an H3Index.
    InvalidIndexArgument(u64),
    /// The binary operation involving the two indices failed. this can happen
    /// because the two indexes are not comparable (different resolutions), too
    /// far apart, or are separated by pentagonal distortion.
    IncompatibleIndices(H3Index, H3Index),
    /// The point could not be indexed.
    UnableToIndex(Point<f64>),
    /// Unable to serialize
    UnableToSerialize(H3Index),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let expression = match self {
            Error::InvalidIndexArgument(arg) => format!("Invalid index arg={}", arg),
            Error::IncompatibleIndices(left, right) => {
                format!("Incompatible H3 indices: {} and {}", left, right)
            }
            Error::UnableToIndex(point) => format!(
                "Unable to index point (lat={}, lon={})",
                point.lat(),
                point.lng()
            ),
            Error::UnableToSerialize(index) => format!("Unable to serialize h3index={}", index),
        };
        write!(f, "{ }", expression)
    }
}

/// `h3-rs`-specific Result type
pub type Result<T> = std::result::Result<T, Error>;

trait ToH3Index {
    /// Indexes the location at the specified resolution, returning the index of
    /// the cell containing the location.
    fn to_h3_index(&self, res: GridResolution) -> Result<H3Index>;
}

/// ## H3 Grid Resolution
///
/// For more details, see the [overview][overview] of the H3 indexing system and
/// the accompanying [resolution table][res_table]. The following table will
/// help you choose a value for a specific grid resolution:
///
///
/// | H3 Resolution | Average Hexagon Area (km<sup>2</sup>) | Average Hexagon Edge Length (km)
/// | ------------- | ------------------------------------: | -------------------------------:
/// | Z0            | 4,250,546.8477000                     | 1,107.712591000
/// | Z1            |   607,220.9782429                     |   418.676005500
/// | Z2            |    86,745.8540347                     |   158.244655800
/// | Z3            |    12,392.2648621                     |    59.810857940
/// | Z4            |     1,770.3235517                     |    22.606379400
/// | Z5            |       252.9033645                     |     8.544408276
/// | Z6            |        36.1290521                     |     3.229482772
/// | Z7            |         5.1612932                     |     1.220629759
/// | Z8            |         0.7373276                     |     0.461354684
/// | Z9            |         0.1053325                     |     0.174375668
/// | Z10           |         0.0150475                     |     0.065907807
/// | Z11           |         0.0021496                     |     0.024910561
/// | Z12           |         0.0003071                     |     0.009415526
/// | Z13           |         0.0000439                     |     0.003559893
/// | Z14           |         0.0000063                     |     0.001348575
/// | Z15           |         0.0000009                     |     0.000509713
///
/// [overview]: https://uber.github.io/h3/#/documentation/core-library/overview
/// [res_table]: https://uber.github.io/h3/#/documentation/core-library/resolution-table
#[allow(unused_variables)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Primitive)]
pub enum GridResolution {
    Z0 = 0,
    Z1 = 1,
    Z2 = 2,
    Z3 = 3,
    Z4 = 4,
    Z5 = 5,
    Z6 = 6,
    Z7 = 7,
    Z8 = 8,
    Z9 = 9,
    Z10 = 10,
    Z11 = 11,
    Z12 = 12,
    Z13 = 13,
    Z14 = 14,
    Z15 = 15,
}

impl GridResolution {
    /// Average hexagon edge length in meters at the given resolution.
    pub fn edge_length(self) -> f64 {
        unsafe { h3_sys::edgeLengthM(self as i32) }
    }

    /// Average hexagon area in square meters at the given resolution.
    pub fn hex_area(self) -> f64 {
        unsafe { h3_sys::hexAreaM2(self as i32) }
    }

    /// Number of unique H3 indexes at the given resolution.
    pub fn num_hexagons(self) -> i64 {
        unsafe { h3_sys::numHexagons(self as i32) }
    }
}

impl H3Index {
    /// Construct an H3Index
    pub fn new(index: u64) -> Result<Self> {
        if H3Index(index).is_valid() {
            Ok(Self(index))
        } else {
            Err(Error::InvalidIndexArgument(index))
        }
    }

    /// Returns grid distance to another H3Index
    pub fn distance_to(self, other: H3Index) -> Result<i32> {
        unsafe {
            let distance = h3_sys::h3Distance(self.0, other.0);
            if distance < 0 {
                Err(Error::IncompatibleIndices(self, other))
            } else {
                Ok(distance)
            }
        }
    }

    /// Is the given H3Index a pentagon?
    pub fn is_pentagon(&self) -> bool {
        unsafe { h3_sys::h3IsPentagon(self.0) != 0 }
    }

    /// Is the given H3Index valid?
    pub fn is_valid(&self) -> bool {
        unsafe { h3_sys::h3IsValid(self.0) != 0 }
    }

    /// Returns true if this index has a resolution with Class III orientation,
    /// false otherwise.
    pub fn is_res_class3(&self) -> bool {
        unsafe { h3_sys::h3IsResClassIII(self.0) != 0 }
    }

    /// Returns the base cell number of the index.
    pub fn get_base_cell(&self) -> i32 {
        unsafe { h3_sys::h3GetBaseCell(self.0) }
    }

    /// Returns the resolution of the given H3Index
    pub fn get_resolution(&self) -> Option<GridResolution> {
        unsafe { GridResolution::from_i32(h3_sys::h3GetResolution(self.0)) }
    }

    /// Return centroid of the given H3Index.
    pub fn get_centroid(&self) -> Point<f64> {
        let mut c = h3_sys::GeoCoord::default();
        unsafe {
            h3_sys::h3ToGeo(self.0, &mut c);
        }
        GeoCoord(c).into()
    }

    /// Returns the maximum number of icosahedron faces the given H3 index may
    /// intersect.
    fn get_max_face_count(&self) -> usize {
        unsafe { h3_sys::maxFaceCount(self.0) as usize }
    }

    /// Return vector of all icosahedron faces intersected by a given H3
    pub fn get_icosahedron_faces(&self) -> Vec<i32> {
        let num_faces = self.get_max_face_count();
        let mut buf = Vec::<i32>::with_capacity(num_faces);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            h3_sys::h3GetFaces(self.0, ptr as *mut i32);
            Vec::from_raw_parts(ptr, num_faces, num_faces)
        }
    }

    /// Get H3 indices (or 'k-ring') within distance k of the given
    /// index. k-ring 0 is defined as the origin index, k-ring 1 is defined as
    /// k-ring 0 and all neighboring indices, and so on.
    pub fn get_k_ring_indices(&self, k: i32) -> Vec<H3Index> {
        // Get the maximum number of indices that result from the kRing
        // algorithm with the given k.
        let k_ring_size = unsafe { h3_sys::maxKringSize(k) } as usize;
        // TODO(mookerji): Verify that this coercion below is safe with H3Index.
        let mut buf = Vec::<H3Index>::with_capacity(k_ring_size);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            h3_sys::kRing(self.0, k, ptr as *mut h3_sys::H3Index);
            Vec::from_raw_parts(ptr, k_ring_size, k_ring_size)
        }
    }

    /// Returns the parent (or grandparent, etc) hexagon of the given hexagon
    pub fn get_parent(&self, res: GridResolution) -> H3Index {
        unsafe { H3Index(h3_sys::h3ToParent(self.0, res as i32)) }
    }

    /// Returns the maximum number of children (or grandchildren, etc) that
    /// could be for a given H3Index
    pub fn get_max_children(&self, child_res: GridResolution) -> usize {
        unsafe { h3_sys::maxH3ToChildrenSize(self.0, child_res as i32) as usize }
    }

    /// Returns the children for a given H3Index
    pub fn get_children(&self, child_res: GridResolution) -> Vec<H3Index> {
        let num_children = self.get_max_children(child_res);
        let mut buf = Vec::<H3Index>::with_capacity(num_children);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            h3_sys::h3ToChildren(self.0, child_res as i32, ptr as *mut h3_sys::H3Index);
            Vec::from_raw_parts(ptr, num_children, num_children)
        }
    }
}

impl std::fmt::Display for H3Index {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "H3Index={ }", self.0)
    }
}

impl From<H3Index> for Result<String> {
    fn from(h: H3Index) -> Result<String> {
        // h3 magic number for string buffer width
        const BUF_SIZE: usize = 17;
        let buf = Vec::<u8>::with_capacity(BUF_SIZE);
        let ptr = CString::new(buf).expect("CString::new failed!").into_raw();
        unsafe {
            h3_sys::h3ToString(h.0, ptr, BUF_SIZE);
            match CString::from_raw(ptr).into_string() {
                Ok(s) => Ok(s),
                Err(_) => Err(Error::UnableToSerialize(h)),
            }
        }
    }
}

impl From<String> for H3Index {
    fn from(s: String) -> H3Index {
        let terminated = CString::new(s).unwrap();
        unsafe { H3Index(h3_sys::stringToH3(terminated.as_ptr())) }
    }
}

impl ToH3Index for Point<f64> {
    fn to_h3_index(&self, res: GridResolution) -> Result<H3Index> {
        let c = GeoCoord::from(*self).0;
        let index = unsafe { h3_sys::geoToH3(&c, res as i32) };
        if index == 0 {
            Err(Error::UnableToIndex(*self))
        } else {
            H3Index::new(index)
        }
    }
}

impl From<H3Index> for Point<f64> {
    /// Finds the centroid of the index.
    fn from(i: H3Index) -> Point<f64> {
        let mut c = h3_sys::GeoCoord::default();
        unsafe {
            h3_sys::h3ToGeo(i.0, &mut c);
        }
        GeoCoord(c).into()
    }
}

impl From<H3Index> for LineString<f64> {
    /// Finds the GeoJSON cell boundary in lat/lon coordinates for the H3Index
    /// cell.
    fn from(i: H3Index) -> LineString<f64> {
        let mut c = h3_sys::GeoBoundary::default();
        unsafe {
            h3_sys::h3ToGeoBoundary(i.0, &mut c);
        }
        GeoBoundary(c).into()
    }
}

impl From<Point<f64>> for GeoCoord {
    fn from(p: Point<f64>) -> GeoCoord {
        unsafe {
            GeoCoord(h3_sys::GeoCoord {
                lat: h3_sys::degsToRads(p.lat()),
                lon: h3_sys::degsToRads(p.lng()),
            })
        }
    }
}

impl From<GeoCoord> for Point<f64> {
    fn from(c: GeoCoord) -> Point<f64> {
        unsafe { Point::new(h3_sys::radsToDegs(c.0.lon), h3_sys::radsToDegs(c.0.lat)) }
    }
}

impl From<GeoCoord> for Coordinate<f64> {
    fn from(c: GeoCoord) -> Coordinate<f64> {
        unsafe {
            Coordinate {
                x: h3_sys::radsToDegs(c.0.lon),
                y: h3_sys::radsToDegs(c.0.lat),
            }
        }
    }
}

// Coercion of H3-internal GeoJSON types to geo-types GeoJSON types.

// TODO(mookerji): Map a path out of this boilerplate.

impl From<GeoFence> for LineString<f64> {
    fn from(c: GeoFence) -> LineString<f64> {
        let num_vertices = c.0.numVerts as usize;
        let h3coords: Vec<h3_sys::GeoCoord> = unsafe { CVec::new(c.0.verts, num_vertices).into() };
        let coords: Vec<Coordinate<f64>> = h3coords
            .iter()
            .take(num_vertices)
            .map(|c| GeoCoord(*c).into())
            .collect();
        coords.into()
    }
}

impl From<GeoBoundary> for LineString<f64> {
    fn from(c: GeoBoundary) -> LineString<f64> {
        let num_vertices = c.0.numVerts as usize;
        let verts: Vec<Coordinate<f64>> =
            c.0.verts
                .iter()
                .take(num_vertices)
                .map(|c| GeoCoord(*c).into())
                .collect();
        verts.into()
    }
}

impl From<GeoPolygon> for Polygon<f64> {
    fn from(p: GeoPolygon) -> Polygon<f64> {
        let num_holes = p.0.numHoles as usize;
        let holes: Vec<h3_sys::Geofence> = unsafe { CVec::new(p.0.holes, num_holes).into() };
        Polygon::new(
            GeoFence(p.0.geofence).into(),
            holes
                .iter()
                .map(|h| -> LineString<f64> { GeoFence(*h).into() })
                .collect(),
        )
    }
}

impl From<GeoMultiPolygon> for MultiPolygon<f64> {
    fn from(p: GeoMultiPolygon) -> MultiPolygon<f64> {
        let num_poly = p.0.numPolygons as usize;
        let poly: Vec<h3_sys::GeoPolygon> = unsafe { CVec::new(p.0.polygons, num_poly).into() };
        MultiPolygon(
            poly.iter()
                .map(|p| -> Polygon<f64> { GeoPolygon(*p).into() })
                .collect(),
        )
    }
}

#[cfg(test)]
#[macro_use]
extern crate approx;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_resolution() {
        assert_relative_eq!(GridResolution::Z1.edge_length(), 418676.0055);
        assert_eq!(GridResolution::Z1.num_hexagons(), 842);
        assert_relative_eq!(GridResolution::Z1.hex_area(), 607221000000.0);
    }

    // Sanity check around round tripping points.
    #[test]
    fn test_round_trip() {
        let orig = Point::new(-122.0553238, 37.3615593);
        let c: GeoCoord = orig.into();
        let new: Point<f64> = c.into();
        assert_relative_eq!(orig.lat(), new.lat());
        assert_relative_eq!(orig.lng(), new.lng());
    }

    #[test]
    fn test_point_to_index() {
        assert_eq!(
            Point::new(-122.0553238, 37.3615593).to_h3_index(GridResolution::Z7),
            Ok(H3Index(0x87283472bffffff))
        );
        assert!(Point::new(std::f64::NAN, 0.)
            .to_h3_index(GridResolution::Z0)
            .is_err());
    }

    // #[test]
    // fn test_index_to_point() {
    //     let p: Point<f64> = H3Index(0x87283472bffffff).into();
    //     assert_relative_eq!(p.lng(), -122.0553238, epsilon = 1.0e-5);
    //     assert_relative_eq!(p.lat(), 37.3615593, epsilon = 1.0e-5);
    // }

    #[test]
    fn test_index_to_boundary() {
        let index = H3Index(0x87283472bffffff);
        let line: LineString<f64> = index.into();
        assert_eq!(line.num_coords(), 6);
    }

    #[test]
    fn test_index_children() {
        let index = H3Index(0x87283472bffffff);
        let z7_children = index.get_children(GridResolution::Z7);
        assert_eq!(z7_children.len(), 1);
        let z8_children = index.get_children(GridResolution::Z8);
        assert_eq!(z8_children.len(), 7);
    }

}
