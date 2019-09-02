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

//! Shared error types

pub use crate::index::*;

pub use geo_types::Point;

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
    /// Unable to serialize
    UnableToSerialize(H3Index),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let expression = match self {
            Error::InvalidIndexArgument(arg) => format!("Invalid index arg={}", arg),
            Error::IncompatibleIndices(left, right) => {
                format!("Incompatible H3 indices: {} and {}", left, right)
            }
            Error::UnableToIndex(point) => format!(
                "Unable to index point (lat={}, lon={})",
                point.lat(),
                point.lng()
            ),
            Error::UnableToSerialize(index) => format!("Unable to serialize h3index={}", index),
        };
        write!(f, "{ }", expression)
    }
}
