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

//! Hierarchical grid functions

use crate::index::*;
use crate::resolution::*;

impl H3Index {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_children() {
        let index = H3Index(0x87283472bffffff);
        let z7_children = index.get_children(GridResolution::Z7);
        assert_eq!(z7_children.len(), 1);
        let z8_children = index.get_children(GridResolution::Z8);
        assert_eq!(z8_children.len(), 7);
    }
}
