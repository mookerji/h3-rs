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

pub use crate::errors::*;
pub use crate::hierarchy::*;
pub use crate::index::*;
pub use crate::inspection::*;
pub use crate::region::*;
pub use crate::resolution::*;
pub use crate::traversal::*;
pub use crate::types::*;

pub mod errors;
pub mod hierarchy;
pub mod index;
pub mod inspection;
mod raw;
pub mod region;
pub mod resolution;
pub mod traversal;
pub mod types;

pub use geo_types::{LineString, MultiPolygon, Point, Polygon};

#[cfg(test)]
#[macro_use]
extern crate approx;

// Many of the tests in this crate are ported verbatim from the Python and
// javascript unit tests found here:
// - https://github.com/uber/h3-py/blob/master/tests/test_h3.py
// - https://github.com/uber/h3-js/blob/master/test/h3core.spec.js
