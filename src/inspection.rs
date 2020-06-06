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

//! H3 inspection
//!
//! Functions for retrieving metadata about an H3 index, such as its resolution
//! or base cell, and provide utilities for converting into and out of the
//! 64-bit representation of an H3 index.

use crate::errors::*;
use crate::resolution::*;
use crate::types::*;

use num_traits::FromPrimitive;
use std::ffi::CString;

impl H3Index {
    /// Is the given H3Index valid?
    pub fn is_valid(&self) -> bool {
        unsafe { h3_sys::h3IsValid(self.0) != 0 }
    }

    /// Is the given H3Index a pentagon?
    pub fn is_pentagon(&self) -> bool {
        unsafe { h3_sys::h3IsPentagon(self.0) != 0 }
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

#[cfg(test)]
mod tests {
    use super::*;

    use geo_types::Point;

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
}
