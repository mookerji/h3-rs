// Copyright 2016-2020 Uber Technologies, Inc.
// Copyright 2020      Bhaskar Mookerji
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

//! H3 indexing
//!
//! This module defines the H3 index, as well as functions defining coordinate
//! conversions and boundaries.

use crate::errors::*;
use crate::raw::*;
use crate::resolution::*;
use crate::types::*;

/// A unique hierarchical index for an H3 cell
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct H3Index(pub h3_sys::H3Index);

pub trait ToH3Index {
    /// Indexes the location at the specified resolution, returning the index of
    /// the cell containing the location.
    fn to_h3_index(&self, res: GridResolution) -> Result<H3Index>;
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

    /// Return centroid of the given H3Index.
    pub fn centroid(&self) -> Point<f64> {
        let mut c = h3_sys::GeoCoord::default();
        unsafe {
            h3_sys::h3ToGeo(self.0, &mut c);
        }
        GeoCoord(c).into()
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

impl std::str::FromStr for H3Index {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let idx_val = s.parse::<u64>()?;
        H3Index::new(idx_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use num_traits::FromPrimitive;

    #[test]
    fn test_h3_is_valid() {
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
            index.resolution().expect("GridResolution failed!"),
            GridResolution::Z5
        );
        assert_approx_point(
            index.into(),
            Point::new(-121.97637597255124, 37.34579337536848),
            1.0e-9,
        );
    }
}
