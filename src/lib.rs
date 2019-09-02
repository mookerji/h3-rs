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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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

trait ToH3Region {
    /// Returns H3Index's covering the given region.
    fn polyfill_h3_index(&self, res: GridResolution) -> Vec<H3Index>;

    /// Maximum number of hexagons in the given region.
    fn get_h3_polyfill_size(&self, res: GridResolution) -> usize;
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

pub const MAX_GRID_RESOLUTION: i32 = GridResolution::Z15 as i32;

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
            // TODO(mookerji): figure out how to deal with .clone() / borrowed
            // content here.
            Vec::from_raw_parts(ptr, k_ring_size, k_ring_size)
                .iter()
                .filter_map(|i| {
                    if *i != H3Index(0) {
                        Some(i.clone())
                    } else {
                        None
                    }
                })
                .collect()
        }
    }

    /// Get H3 indices (or 'k-ring') within distance k of the given
    /// index, reporting distance from the origin.
    pub fn get_k_ring_distances(&self, k: i32) -> Vec<Vec<H3Index>> {
        // Get the maximum number of indices that result from the kRing
        // algorithm with the given k.
        let k_ring_size = unsafe { h3_sys::maxKringSize(k) } as usize;
        // TODO(mookerji): Verify that this coercion below is safe with H3Index.
        let mut h3_buf = Vec::<H3Index>::with_capacity(k_ring_size);
        let h3_ptr = h3_buf.as_mut_ptr();
        let mut distance_buf = Vec::<i32>::with_capacity(k_ring_size);
        let distance_ptr = distance_buf.as_mut_ptr();
        let (indices, distances) = unsafe {
            std::mem::forget(h3_buf);
            std::mem::forget(distance_buf);
            h3_sys::kRingDistances(
                self.0,
                k,
                h3_ptr as *mut h3_sys::H3Index,
                distance_ptr as *mut i32,
            );
            (
                Vec::from_raw_parts(h3_ptr, k_ring_size, k_ring_size),
                Vec::from_raw_parts(distance_ptr, k_ring_size, k_ring_size),
            )
        };
        let distance_size = *distances.iter().max().unwrap() as usize + 1;
        let mut result = vec![Vec::new(); distance_size];
        for i in 0..k_ring_size {
            if indices[i] == H3Index(0) {
                continue;
            }
            result[distances[i] as usize].push(indices[i].clone());
        }
        result
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

impl From<LineString<f64>> for GeoFence {
    fn from(c: LineString<f64>) -> GeoFence {
        let num_verts = c.num_coords() as i32;
        // YUCK
        let mut v: Vec<h3_sys::GeoCoord> = c
            .into_points()
            .iter()
            .map(|&c| -> h3_sys::GeoCoord {
                let f: GeoCoord = c.into();
                f.0
            })
            .collect();
        let ptr = v.as_mut_ptr();
        unsafe {
            std::mem::forget(v);
            GeoFence(h3_sys::Geofence {
                numVerts: num_verts,
                verts: ptr,
            })
        }
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
                .map(|&h| -> LineString<f64> { GeoFence(h).into() })
                .collect(),
        )
    }
}
impl From<Polygon<f64>> for GeoPolygon {
    fn from(p: Polygon<f64>) -> GeoPolygon {
        let (exterior, interiors) = p.into_inner();
        let geofence: GeoFence = exterior.into();
        let num_holes = interiors.len() as i32;
        let mut holes: Vec<GeoFence> = interiors
            .into_iter()
            .map(|g| -> GeoFence { g.into() })
            .collect();
        GeoPolygon(h3_sys::GeoPolygon {
            geofence: geofence.0,
            numHoles: num_holes,
            holes: holes.as_mut_ptr() as *mut h3_sys::Geofence,
        })
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

impl ToH3Region for Polygon<f64> {
    fn polyfill_h3_index(&self, res: GridResolution) -> Vec<H3Index> {
        let polygon: GeoPolygon = self.clone().into();
        let max_indices = self.get_h3_polyfill_size(res);
        let mut buf = Vec::<H3Index>::with_capacity(max_indices);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            h3_sys::polyfill(&polygon.0, res as i32, ptr as *mut h3_sys::H3Index);
            Vec::from_raw_parts(ptr, max_indices, max_indices)
        }
    }

    fn get_h3_polyfill_size(&self, res: GridResolution) -> usize {
        let polygon: GeoPolygon = self.clone().into();
        unsafe { h3_sys::maxPolyfillSize(&polygon.0, res as i32) as usize }
    }
}

#[cfg(test)]
#[macro_use]
extern crate approx;

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{line_string, polygon};

    #[test]
    fn test_grid_resolution() {
        assert_relative_eq!(GridResolution::Z1.edge_length(), 418676.0055);
        assert_eq!(GridResolution::Z1.num_hexagons(), 842);
        assert_relative_eq!(GridResolution::Z1.hex_area(), 607221000000.0);
    }

    /// Sanity check around round tripping points between h3-rs FFI and Rust
    /// geotypes.
    #[test]
    fn test_round_trip() {
        let orig = Point::new(-122.0553238, 37.3615593);
        let c: GeoCoord = orig.into();
        let new: Point<f64> = c.into();
        assert_relative_eq!(orig.lat(), new.lat());
        assert_relative_eq!(orig.lng(), new.lng());
    }

    // Many of the tests below are ported verbatim from the Python and
    // javascript unit tests found here:
    // - https://github.com/uber/h3-py/blob/master/tests/test_h3.py
    // - https://github.com/uber/h3-js/blob/master/test/h3core.spec.js

    #[test]
    fn test_h3_is_valid() {
        // H3 Address is considered an address
        assert_eq!(
            H3Index::new(0x85283473fffffff),
            Ok(H3Index(0x85283473fffffff))
        );
        assert!(H3Index(0x85283473fffffff).is_valid());
        // H3 Address from Java test also valid
        assert!(H3Index(0x850dab63fffffff).is_valid());
        // H3 0.x Addresses are not considered valid
        assert!(!H3Index(0x5004295803a88).is_valid());
        assert_eq!(
            H3Index::new(0x5004295803a88),
            Err(Error::InvalidIndexArgument(0x5004295803a88))
        );
        // H3 Address is considered an address
        for i in 0..MAX_GRID_RESOLUTION + 1 {
            let res = GridResolution::from_i32(i).expect("GridResolution failed!");
            assert!(Point::new(-122., 37.).to_h3_index(res).is_ok());
        }
        // Added!
        assert!(Point::new(std::f64::NAN, 0.)
            .to_h3_index(GridResolution::Z0)
            .is_err());
        assert!(Point::new(0., std::f64::NAN)
            .to_h3_index(GridResolution::Z0)
            .is_err());
    }

    #[test]
    fn test_geo_to_h3() {
        // geo_to_h3: Got the expected H3 address back
        assert_eq!(
            Point::new(-122.0553238, 37.3615593).to_h3_index(GridResolution::Z5),
            Ok(H3Index(0x85283473fffffff))
        );
    }

    #[test]
    fn test_h3_get_resolution() {
        for i in 0..MAX_GRID_RESOLUTION + 1 {
            let res = GridResolution::from_i32(i).expect("GridResolution failed!");
            let index = Point::new(-122.0553238, 37.3615593)
                .to_h3_index(res)
                .unwrap();
            // Got the expected H3 resolution back!
            assert_eq!(
                index.get_resolution().expect("Point.to_h3_index failed"),
                res
            );
        }
    }

    #[test]
    fn test_silly_geo_to_h3() {
        // world-wrapping lat, lng corrected
        use std::f64::consts::PI;
        let full_rot = 2. * PI.to_degrees();
        let res = GridResolution::Z5;
        let index = Ok(H3Index(0x85283473fffffff));
        let lng = -122.0553238;
        let lat = 37.3615593;
        assert_eq!(Point::new(lng + full_rot, lat).to_h3_index(res), index);
        assert_eq!(Point::new(lng, lat + full_rot).to_h3_index(res), index);
        assert_eq!(
            Point::new(lng + full_rot, lat + full_rot).to_h3_index(res),
            index
        );
    }

    fn assert_approx_point(expected: Point<f64>, actual: Point<f64>, eps: f64) {
        assert_relative_eq!(actual.lat(), expected.lat(), epsilon = eps);
        assert_relative_eq!(actual.lng(), expected.lng(), epsilon = eps);
    }

    #[test]
    fn test_h3_to_geo() {
        let index = H3Index::new(0x85283473fffffff).unwrap();
        assert_eq!(
            index.get_resolution().expect("GridResolution failed!"),
            GridResolution::Z5
        );
        assert_approx_point(
            index.into(),
            Point::new(-121.97637597255124, 37.34579337536848),
            1.0e-9,
        );
    }

    #[test]
    fn test_h3_to_geo_boundary() {
        let expected = line_string![
            (x: -121.91508032705622, y: 37.271355866731895),
            (x: -121.86222328902491, y: 37.353926450852256),
            (x: -121.9235499963016, y: 37.42834118609435),
            (x: -122.0377349642703, y: 37.42012867767778),
            (x: -122.09042892904395, y: 37.33755608435298),
            (x: -122.02910130919, y: 37.26319797461824)
        ];
        let actual: LineString<f64> = H3Index(0x85283473fffffff).into();
        assert_eq!(actual.num_coords(), expected.num_coords());
        let actual_vec = actual.into_points();
        let expected_vec = expected.into_points();
        for i in 0..actual_vec.len() {
            assert_approx_point(actual_vec[i], expected_vec[i], 1.0e-9);
        }
    }

    #[test]
    fn test_k_ring() {
        let k_ring = H3Index(0x8928308280fffff).get_k_ring_indices(1);
        // Check the expected number of hexagons for a single ring
        assert_eq!(k_ring.len(), 1 + 6);
        let expected_hexagons = vec![
            H3Index(0x8928308280fffff),
            H3Index(0x8928308280bffff),
            H3Index(0x89283082807ffff),
            H3Index(0x89283082877ffff),
            H3Index(0x89283082803ffff),
            H3Index(0x89283082873ffff),
            H3Index(0x8928308283bffff),
        ];
        // Expected hexagons are present in k-ring
        for hex in expected_hexagons {
            assert!(k_ring.contains(&hex));
        }
    }

    #[test]
    fn test_k_ring2() {
        let k_ring = H3Index(0x8928308280fffff).get_k_ring_indices(2);
        // Check the expected number of hexagons for two rings
        assert_eq!(k_ring.len(), 1 + 6 + 12);
        let expected_hexagons = vec![
            H3Index(0x89283082813ffff),
            H3Index(0x89283082817ffff),
            H3Index(0x8928308281bffff),
            H3Index(0x89283082863ffff),
            H3Index(0x89283082823ffff),
            H3Index(0x89283082873ffff),
            H3Index(0x89283082877ffff),
            H3Index(0x8928308287bffff),
            H3Index(0x89283082833ffff),
            H3Index(0x8928308282bffff),
            H3Index(0x8928308283bffff),
            H3Index(0x89283082857ffff),
            H3Index(0x892830828abffff),
            H3Index(0x89283082847ffff),
            H3Index(0x89283082867ffff),
            H3Index(0x89283082803ffff),
            H3Index(0x89283082807ffff),
            H3Index(0x8928308280bffff),
            H3Index(0x8928308280fffff),
        ];
        // Expected hexagons are present in k-ring
        for hex in expected_hexagons {
            assert!(k_ring.contains(&hex));
        }
    }

    #[test]
    fn test_k_ring_pentagon() {
        let k_ring = H3Index(0x821c07fffffffff).get_k_ring_indices(1);
        // Check the expected number for a single ring around a pentagon
        assert_eq!(k_ring.len(), 1 + 5);
        let expected_hexagons = vec![
            H3Index(0x821c2ffffffffff),
            H3Index(0x821c27fffffffff),
            H3Index(0x821c07fffffffff),
            H3Index(0x821c17fffffffff),
            H3Index(0x821c1ffffffffff),
            H3Index(0x821c37fffffffff),
        ];
        // Expected hexagons are present in k-ring
        for hex in expected_hexagons {
            assert!(k_ring.contains(&hex));
        }
    }

    #[test]
    fn test_k_ring_distances() {
        let k_ring = H3Index(0x8928308280fffff).get_k_ring_distances(1);
        assert_eq!(k_ring.len(), 2);
        assert_eq!(k_ring[0].len(), 1);
        assert_eq!(k_ring[1].len(), 6);
        assert_eq!(k_ring[0], vec![H3Index(0x8928308280fffff)]);
        let expected_hexagons = vec![
            H3Index(0x8928308280bffff),
            H3Index(0x89283082807ffff),
            H3Index(0x89283082877ffff),
            H3Index(0x89283082803ffff),
            H3Index(0x89283082873ffff),
            H3Index(0x8928308283bffff),
        ];
        for hex in expected_hexagons {
            assert!(k_ring[1].contains(&hex));
        }

        let k_ring2 = H3Index(0x870800003ffffff).get_k_ring_distances(2);
        assert_eq!(k_ring2.len(), 3);
        assert_eq!(k_ring2[0].len(), 1);
        assert_eq!(k_ring2[1].len(), 6);
        assert_eq!(k_ring2[2].len(), 11);
    }

    #[test]
    fn test_polyfill() {
        let poly = polygon![
            exterior: [
                (x: -122.4089866999972145, y: 37.813318999983238),
                (x: -122.3805436999997056, y: 37.7866302000007224),
                (x: -122.3544736999993603, y: 37.7198061999978478),
                (x: -122.5123436999983966, y: 37.7076131999975672),
                (x: -122.5247187000021967, y: 37.7835871999971715),
                (x: -122.4798767000009008, y: 37.8151571999998453),
            ],
            interiors: [[]],
        ];
        let res = GridResolution::Z9;
        let indices = poly.polyfill_h3_index(res);
        let max_indices = poly.get_h3_polyfill_size(res);
        assert_eq!(indices.len(), max_indices);
    }

    #[test]
    fn test_polyfill_with_hole() {
        let poly = polygon!(
            exterior: [
                (x: -122.4089866999972145, y: 37.813318999983238),
                (x: -122.3805436999997056, y: 37.7866302000007224),
                (x: -122.3544736999993603, y: 37.7198061999978478),
                (x: -122.5123436999983966, y: 37.7076131999975672),
                (x: -122.5247187000021967, y: 37.7835871999971715),
                (x: -122.4798767000009008, y: 37.8151571999998453),
            ],
            interiors: [
                [
                    (x: -122.4471197, y: 37.7869802),
                    (x: -122.4590777, y: 37.7664102),
                    (x: -122.4137097, y: 37.7710682)
                ],
            ],
        );
        let res = GridResolution::Z9;
        let indices = poly.polyfill_h3_index(res);
        let max_indices = poly.get_h3_polyfill_size(res);
        assert_eq!(indices.len(), max_indices);
    }

    #[test]
    fn test_polyfill_with_two_holes() {
        let poly = polygon!(
            exterior: [
                (x: -122.4089866999972145, y: 37.813318999983238),
                (x: -122.3805436999997056, y: 37.7866302000007224),
                (x: -122.3544736999993603, y: 37.7198061999978478),
                (x: -122.5123436999983966, y: 37.7076131999975672),
                (x: -122.5247187000021967, y: 37.7835871999971715),
                (x: -122.4798767000009008, y: 37.8151571999998453),
            ],
            interiors: [
                [
                    (x: -122.4471197, y: 37.7869802),
                    (x: -122.4590777, y: 37.7664102),
                    (x: -122.4137097, y: 37.7710682)
                ],
                [
                    (x: -122.490025, y: 37.747976),
                    (x: -122.503758, y: 37.731550),
                    (x: -122.452603, y: 37.725440)
                ],
            ],
        );
        // TODO: if holes are identical, test crashes?
        let res = GridResolution::Z9;
        let indices = poly.polyfill_h3_index(res);
        let max_indices = poly.get_h3_polyfill_size(res);
        assert_eq!(indices.len(), max_indices);
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
