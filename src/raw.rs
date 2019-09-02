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

//! Aliased sys/raw types

use c_vec::CVec;
use geo_types::{Coordinate, LineString, MultiPolygon, Point, Polygon};

pub struct GeoCoord(pub h3_sys::GeoCoord);
pub struct GeoBoundary(pub h3_sys::GeoBoundary);
pub struct GeoFence(pub h3_sys::Geofence);
pub struct GeoPolygon(pub h3_sys::GeoPolygon);
pub struct GeoMultiPolygon(pub h3_sys::GeoMultiPolygon);

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
    #[allow(unused_unsafe)]
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
