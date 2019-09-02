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

//! H3 grid resolution
//!
//! For more details, see the [overview][overview] of the H3 indexing system and
//! the accompanying [resolution table][res_table]. The following table will
//! help you choose a value for a specific grid resolution:
//!
//!
//! | H3 Resolution | Average Hexagon Area (km<sup>2</sup>) | Average Hexagon Edge Length (km)
//! | ------------- | ------------------------------------: | -------------------------------:
//! | Z0            | 4,250,546.8477000                     | 1,107.712591000
//! | Z1            |   607,220.9782429                     |   418.676005500
//! | Z2            |    86,745.8540347                     |   158.244655800
//! | Z3            |    12,392.2648621                     |    59.810857940
//! | Z4            |     1,770.3235517                     |    22.606379400
//! | Z5            |       252.9033645                     |     8.544408276
//! | Z6            |        36.1290521                     |     3.229482772
//! | Z7            |         5.1612932                     |     1.220629759
//! | Z8            |         0.7373276                     |     0.461354684
//! | Z9            |         0.1053325                     |     0.174375668
//! | Z10           |         0.0150475                     |     0.065907807
//! | Z11           |         0.0021496                     |     0.024910561
//! | Z12           |         0.0003071                     |     0.009415526
//! | Z13           |         0.0000439                     |     0.003559893
//! | Z14           |         0.0000063                     |     0.001348575
//! | Z15           |         0.0000009                     |     0.000509713
//!
//! [overview]: https://uber.github.io/h3/#/documentation/core-library/overview
//! [res_table]: https://uber.github.io/h3/#/documentation/core-library/resolution-table

/// H3 Grid Resolution
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

/// Maximum grid resolution
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_resolution() {
        assert_relative_eq!(GridResolution::Z1.edge_length(), 418676.0055);
        assert_eq!(GridResolution::Z1.num_hexagons(), 842);
        assert_relative_eq!(GridResolution::Z1.hex_area(), 607221000000.0);
    }
}
