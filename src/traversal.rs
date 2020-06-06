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

//! Grid traversal functions
//!
//! Grid traversal allows finding cells in the vicinity of an origin cell, and
//! determining how to traverse the grid from one cell to another.

use crate::errors::*;
use crate::types::*;

impl H3Index {
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

    /// Returns grid distance to another H3Index
    pub fn distance_to(&self, other: H3Index) -> Result<i32> {
        unsafe {
            let distance = h3_sys::h3Distance(self.0, other.0);
            if distance < 0 {
                Err(Error::IncompatibleIndices(self.clone(), other))
            } else {
                Ok(distance)
            }
        }
    }

    /// Return the line of indexes to another H3 index. Returns error if the
    /// line cannot be computed.
    pub fn line_to(&self, other: &H3Index) -> Result<Vec<H3Index>> {
        let line_size = self.line_size(other)?;
        let mut buf = Vec::<H3Index>::with_capacity(line_size);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            h3_sys::h3Line(self.0, other.0, ptr as *mut h3_sys::H3Index);
            Ok(Vec::from_raw_parts(ptr, line_size, line_size))
        }
    }

    /// Number of indexes in a line from the this index to the end
    /// index. Returns error if the line cannot be computed.
    fn line_size(&self, other: &H3Index) -> Result<usize> {
        let distance = unsafe { h3_sys::h3LineSize(self.0, other.0) };
        if distance < 0 {
            Err(Error::UnableToComputeH3Line(self.clone(), other.clone()))
        } else {
            Ok(distance as usize)
        }
    }

    /// Produces the hollow hexagonal ring centered at origin with sides of length k.
    pub fn hex_ring(&self, k: i32) -> Result<Vec<H3Index>> {
        let hex_ring_size = unsafe { h3_sys::maxKringSize(k) } as usize;
        let mut buf = Vec::<H3Index>::with_capacity(hex_ring_size);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            let ret = h3_sys::hexRing(self.0, k, ptr as *mut h3_sys::H3Index);
            if ret != 0 {
                Err(Error::UnableToComputeTraversal(self.clone(), k))
            } else {
                Ok(Vec::from_raw_parts(ptr, hex_ring_size, hex_ring_size))
            }
        }
    }

    /// Hexagons neighbors in all directions, assuming no pentagons.
    pub fn hex_range(&self, k: i32) -> Result<Vec<H3Index>> {
        let hex_range_size = unsafe { h3_sys::maxKringSize(k) } as usize;
        let mut buf = Vec::<H3Index>::with_capacity(hex_range_size);
        let ptr = buf.as_mut_ptr();
        unsafe {
            std::mem::forget(buf);
            let ret = h3_sys::hexRange(self.0, k, ptr as *mut h3_sys::H3Index);
            if ret != 0 {
                Err(Error::UnableToComputeTraversal(self.clone(), k))
            } else {
                Ok(Vec::from_raw_parts(ptr, hex_range_size, hex_range_size))
            }
        }
    }

    /// Prfoduces hexagon indexes within k distance of the origin index. Output
    /// behavior is undefined when one of the indexes returned by this function
    /// is a pentagon or is in the pentagon distortion area.
    pub fn hex_range_distances(self, k: i32) -> Result<Vec<Vec<H3Index>>> {
        let hex_range_size = unsafe { h3_sys::maxKringSize(k) } as usize;
        let mut h3_buf = Vec::<H3Index>::with_capacity(hex_range_size);
        let h3_ptr = h3_buf.as_mut_ptr();
        let mut distance_buf = Vec::<i32>::with_capacity(hex_range_size);
        let distance_ptr = distance_buf.as_mut_ptr();
        let (indices, distances) = unsafe {
            std::mem::forget(h3_buf);
            std::mem::forget(distance_buf);
            h3_sys::hexRangeDistances(
                self.0,
                k,
                h3_ptr as *mut h3_sys::H3Index,
                distance_ptr as *mut i32,
            );
            (
                Vec::from_raw_parts(h3_ptr, hex_range_size, hex_range_size),
                Vec::from_raw_parts(distance_ptr, hex_range_size, hex_range_size),
            )
        };
        let distance_size = *distances.iter().max().unwrap() as usize + 1;
        let mut result = vec![Vec::new(); distance_size];
        for i in 0..hex_range_size {
            if indices[i] == H3Index(0) {
                continue;
            }
            result[distances[i] as usize].push(indices[i].clone());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
