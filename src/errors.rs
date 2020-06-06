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
    /// Unable to parse string value to integer
    ParseIntError(std::num::ParseIntError),
    /// Invalid resolution argument
    InvalidResolutionArgument(i32),
    /// Unable to compute line between two H3 indices
    UnableToComputeH3Line(H3Index, H3Index),
    /// Unable to compute a traversal (hex range or hex ring) centered at H3
    /// index.
    UnableToComputeTraversal(H3Index, i32),
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
            Error::ParseIntError(error) => format!("Unable to parse integer. error={}", error),
            Error::InvalidResolutionArgument(arg) => {
                format!("Unable to parse integer to create resolution. arg={}", arg)
            }
            Error::UnableToComputeH3Line(left, right) => format!(
                "Unable to compute line between indices: left={} right={}",
                left, right
            ),
            Error::UnableToComputeTraversal(index, k) => {
                format!("Unable to compute traversal index={} k={}", index, k)
            }
        };
        write!(f, "{ }", expression)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}
