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

//! Hierarchical grid functions
//!
//! These functions permit moving between resolutions in the H3 grid system. The
//! functions produce parent (coarser) or children (finer) cells.

use crate::errors::*;
use crate::region::*;
use crate::resolution::*;
use crate::types::*;

use geo_types::Polygon;

impl H3Index {
    /// Returns the parent (or grandparent, etc) hexagon of the given hexagon
    pub fn parent(&self, res: GridResolution) -> H3Index {
        unsafe { H3Index(h3_sys::h3ToParent(self.0, res as i32)) }
    }

    /// Returns the maximum number of children (or grandchildren, etc) that
    /// could be for a given H3Index
    pub fn max_children(&self, child_res: GridResolution) -> usize {
        unsafe { h3_sys::maxH3ToChildrenSize(self.0, child_res as i32) as usize }
    }

    /// Returns the children for a given H3Index
    pub fn children(&self, child_res: GridResolution) -> Vec<H3Index> {
        let num_children = self.max_children(child_res);
        let mut buf = Vec::<H3Index>::with_capacity(num_children);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            h3_sys::h3ToChildren(self.0, child_res as i32, ptr as *mut h3_sys::H3Index);
            Vec::from_raw_parts(ptr, num_children, num_children)
        }
    }
}

/// Returns the size of the array needed by uncompact.
fn uncompact_size(set: &Vec<H3Index>, res: GridResolution) -> usize {
    unsafe {
        h3_sys::maxUncompactSize(
            set.as_ptr() as *const h3_sys::H3Index,
            set.len() as i32,
            res as i32,
        ) as usize
    }
}

pub trait ToCompactH3Region {
    /// Compacts the set indexes as best as possible.
    fn compact(&self) -> Result<Vec<H3Index>>;
}

/// Uncompacts the set of indexes to the resolution
fn uncompact(set: &Vec<H3Index>, res: GridResolution) -> Result<Vec<H3Index>> {
    let max_size = uncompact_size(&set, res);
    let mut buf = Vec::<H3Index>::with_capacity(max_size);
    let ptr = buf.as_mut_ptr();
    unsafe {
        let err = h3_sys::uncompact(
            set.as_ptr() as *const h3_sys::H3Index,
            set.len() as i32,
            ptr as *mut h3_sys::H3Index,
            max_size as i32,
            res as i32,
        );
        if err == 0 {
            Ok(Vec::from_raw_parts(ptr, max_size, max_size))
        } else {
            Err(Error::UnableToCompact(set.clone()))
        }
    }
}

impl ToCompactH3Region for Vec<H3Index> {
    fn compact(&self) -> Result<Vec<H3Index>> {
        let mut buf = Vec::<H3Index>::with_capacity(self.len());
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            let err = h3_sys::compact(
                self.as_ptr() as *const h3_sys::H3Index,
                ptr as *mut h3_sys::H3Index,
                self.len() as i32,
            );
            if err == 0 {
                Ok(Vec::from_raw_parts(ptr, self.len(), self.len()))
            } else {
                Err(Error::UnableToCompact(self.clone()))
            }
        }
    }
}

impl ToCompactH3Region for Polygon<f64> {
    fn compact(&self) -> Result<Vec<H3Index>> {
        let res = GridResolution::Z9;
        self.polyfill(res).compact()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::polygon;

    #[test]
    fn test_index_children() {
        let index = H3Index(0x87283472bffffff);
        let z7_children = index.children(GridResolution::Z7);
        assert_eq!(z7_children.len(), 1);
        let z8_children = index.children(GridResolution::Z8);
        assert_eq!(z8_children.len(), 7);
    }

    #[test]
    fn test_compact_and_uncompact() {
        let poly = polygon!(
            exterior: [
                (x: -122.4089866999972145, y: 37.813318999983238),
                (x: -122.3805436999997056, y: 37.7866302000007224),
                (x: -122.3544736999993603, y: 37.7198061999978478),
                (x:  -122.5123436999983966, y: 37.7076131999975672),
                (x:  -122.5247187000021967, y: 37.7835871999971715),
                (x: -122.4798767000009008, y: 37.8151571999998453),
            ],
            interiors: [],
        );
        let res = GridResolution::Z9;
        let compact_hexes = poly.polyfill(res).compact().unwrap();
        assert_eq!(compact_hexes.len(), 209);
        let uncompact_hexes = uncompact(&compact_hexes, res).unwrap();
        assert_eq!(uncompact_hexes.len(), 1253);
    }
}
