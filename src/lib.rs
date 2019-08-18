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
//! This library contains bindings to the [h3][h3_uber] C library for hexagonal,
//! hierarchical geospatial indexing.
//!
//! This is **experimental, in-progress software**.
//!
//! [h3_uber]: https://uber.github.io/h3/#/

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

extern crate geo_types;

use h3_sys;
use num_traits::FromPrimitive;
use std::ffi::{CString, IntoStringError};

pub use geo_types::{LineString, MultiPolygon, Point, Polygon};

struct GeoCoord(h3_sys::GeoCoord);
struct GeoFence(h3_sys::Geofence);
struct GeoPolygon(h3_sys::GeoPolygon);
struct GeoMultiPolygon(h3_sys::GeoMultiPolygon);

/// A unique hierarchical index for an H3 cell
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct H3Index(pub h3_sys::H3Index);

impl std::fmt::Display for H3Index {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let expression = "point!";
        write!(f, "{ }", expression)
    }
}

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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let expression = match self {
            Error::InvalidIndexArgument(arg) => format!("Invalid index arg={}", arg),
            Error::IncompatibleIndices(left, right) => {
                format!("Incompatible H3 indices: {} and {}", left, right)
            }
            Error::UnableToIndex(point) => format!("Unable to index point (lat={}, lon={})", point.lat(), point.lng()),
        };
        write!(f, "{ }", expression)
    }
}

/// `h3-rs`-specific Result type
pub type Result<T> = std::result::Result<T, Error>;

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
pub enum Resolution {
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

impl Resolution {
    /// Average hexagon edge length in meters at the given resolution.
    pub fn edge_length(&self) -> f64 {
        unsafe { h3_sys::edgeLengthM(*self as i32) }
    }

    /// Average hexagon area in square meters at the given resolution.
    pub fn hex_area(&self) -> f64 {
        unsafe { h3_sys::hexAreaM2(*self as i32) }
    }

    /// Number of unique H3 indexes at the given resolution.
    pub fn num_hexagons(&self) -> i64 {
        unsafe { h3_sys::numHexagons(*self as i32) }
    }
}

// impl std::fmt::Display for H3Index {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let s = self.into().unwrap_or("INVALID");
//         write!(f, "{}", s)
//     }
// }

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
        unsafe { h3_sys::h3IsPentagon(self.0) == 1 }
    }

    /// Is the given H3Index valid?
    pub fn is_valid(&self) -> bool {
        unsafe { h3_sys::h3IsValid(self.0) == 1 }
    }

    /// Returns the resolution of the provided hexagon
    pub fn get_base_cell(&self) -> i32 {
        unsafe { h3_sys::h3GetBaseCell(self.0) }
    }

    /// Returns the resolution of the given H3Index
    pub fn get_resolution(&self) -> Option<Resolution> {
        unsafe { Resolution::from_i32(h3_sys::h3GetResolution(self.0)) }
    }

    /// Return centroid of the given H3Index.
    pub fn get_centroid(&self) -> Point<f64> {
        let mut c = h3_sys::GeoCoord::default();
        unsafe {
            h3_sys::h3ToGeo(self.0, &mut c);
        }
        Point::from(GeoCoord(c))
    }

    /// Returns the parent (or grandparent, etc) hexagon of the given hexagon
    pub fn get_parent(&self, res: Resolution) -> H3Index {
        unsafe { H3Index(h3_sys::h3ToParent(self.0, res as i32)) }
    }

    /// Returns the maximum number of children (or grandchildren, etc) that
    /// could be for a given H3Index
    pub fn get_max_children(&self, child_res: Resolution) -> i32 {
        unsafe { h3_sys::maxH3ToChildrenSize(self.0, child_res as i32) }
    }

    /// Returns the children for a given H3Index
    pub fn get_children(&self, child_res: Resolution) -> Vec<H3Index> {
        let num_children = self.get_max_children(child_res) as usize;
        let mut buffer = Vec::<H3Index>::with_capacity(num_children);
        let buffer_ptr = buffer.as_mut_ptr();
        unsafe {
            std::mem::forget(buffer);
            h3_sys::h3ToChildren(self.0, child_res as i32, buffer_ptr as *mut h3_sys::H3Index);
            Vec::from_raw_parts(buffer_ptr, num_children, num_children)
        }
    }
}

impl From<H3Index> for std::result::Result<String, IntoStringError> {
    fn from(h: H3Index) -> std::result::Result<String, IntoStringError> {
        const BUF_SIZE: usize = 17;
        let buf = Vec::<u8>::with_capacity(BUF_SIZE);
        let ptr = CString::new(buf).expect("CString::new failed!").into_raw();
        unsafe {
            h3_sys::h3ToString(h.0, ptr, BUF_SIZE);
            let c_str = CString::from_raw(ptr);
            c_str.into_string()
        }
    }
}

impl From<String> for H3Index {
    fn from(s: String) -> H3Index {
        let terminated = CString::new(s).unwrap();
        unsafe { H3Index(h3_sys::stringToH3(terminated.as_ptr())) }
    }
}

/// Indexes the location at the specified resolution, returning the index of the
/// cell containing the location.
pub fn point_to_index(point: Point<f64>, res: Resolution) -> Result<H3Index> {
    let c = GeoCoord::from(point).0;
    let index = unsafe { h3_sys::geoToH3(&c, res as i32) };
    if index == 0 {
        Err(Error::UnableToIndex(point))
    } else {
        H3Index::new(index)
    }
}

impl From<H3Index> for Point<f64> {
    /// Finds the centroid of the index.
    fn from(i: H3Index) -> Point<f64> {
        let mut c = h3_sys::GeoCoord::default();
        unsafe {
            h3_sys::h3ToGeo(i.0, &mut c);
        }
        Point::from(GeoCoord(c))
    }
}

impl From<Point<f64>> for GeoCoord {
    fn from(p: Point<f64>) -> GeoCoord {
        unsafe {
            GeoCoord(h3_sys::GeoCoord {
                lat: h3_sys::degsToRads(p.y()),
                lon: h3_sys::degsToRads(p.x()),
            })
        }
    }
}

impl From<GeoCoord> for Point<f64> {
    fn from(c: GeoCoord) -> Point<f64> {
        unsafe { Point::new(h3_sys::radsToDegs(c.0.lon), h3_sys::radsToDegs(c.0.lat)) }
    }
}

// impl From<GeoFence> for LineString<f64> {
//     fn from(c: GeoFence) -> LineString<f64> {
//         let num_points = c.0.numVerts;

//         for

//     }
// }

// impl From<GeoPolygon> for Polygon<f64> {
//     fn from(p: GeoPolygon) -> Polygon<f64> {
//         let exterior = LineString::from(p.0.geofence);
//         let holes = ();
//         Polygon::new(exterior, holes)
//     }
// }

// impl From<Polygon<f64>> for GeoPolygon {
//     fn from(c: Polygon<f64>) -> GeoPolygon {}
// }

// impl From<GeoMultiPolygon> for MultiPolygon {
//     fn from(c: GeoMultiPolygon ) -> MultiPolygon {

//     }
// }

// impl From<MultiPolygon> for GeoMultiPolygon {
//     fn from(c: MultiPolygon) -> GeoMultiPolygon {

//     }
// }

// Utility functions around hexagons

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution() {
        assert_eq!(Resolution::Z1.edge_length(), 418676.0055);
        assert_eq!(Resolution::Z1.num_hexagons(), 842);
        assert_eq!(Resolution::Z1.hex_area(), 607221000000.0);
    }

    #[test]
    fn test_h3_index() {}

    #[test]
    fn test_point_to_index() {
        let p = Point::new(-122.0553238, 37.3615593);
        assert_eq!(
            point_to_index(p, Resolution::Z7),
            H3Index(0x87283472bffffff)
        );
    }
}
