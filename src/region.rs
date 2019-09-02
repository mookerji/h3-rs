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

//! Conversions H3 indexes to and from polygonal regions

use crate::aliases::*;
use crate::index::*;
use crate::resolution::*;

use geo_types::{LineString, Polygon};

// Coercion of H3-internal GeoJSON types to geo-types GeoJSON types.

// TODO(mookerji): Map a path out of this boilerplate.

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

pub trait ToH3Region {
    /// Returns H3Index's covering the given region.
    fn polyfill_h3_index(&self, res: GridResolution) -> Vec<H3Index>;

    /// Maximum number of hexagons in the given region.
    fn get_h3_polyfill_size(&self, res: GridResolution) -> usize;
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
mod tests {
    use super::*;
    use geo_types::{line_string, polygon, Point};

    fn assert_approx_point(expected: Point<f64>, actual: Point<f64>, eps: f64) {
        assert_relative_eq!(actual.lat(), expected.lat(), epsilon = eps);
        assert_relative_eq!(actual.lng(), expected.lng(), epsilon = eps);
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

}
