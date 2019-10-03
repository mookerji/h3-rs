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

//! CLI entrypoint to h3_rs

#[macro_use]
extern crate clap;
extern crate geo_types;
extern crate h3_rs;

use clap::App;
use h3_rs::*;
use geo_types::{Point};

#[derive(Clone, Debug, PartialEq)]
enum Command {
    IndexToBoundary(Vec<H3Index>),
    IndexToPoint(H3Index),
    PointToIndex(Point<f64>, GridResolution),
    BoundaryToIndex(),
}

fn try_main() -> Result<()> {
    let yaml = load_yaml!("./cli-defs.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    Ok(())
}

fn index_to_boundary() -> Result<()>{
    Ok(())
}

fn index_to_point() -> Result<()>{
    Ok(())
}

fn point_to_index() -> Result<()>{
    Ok(())
}

fn boundary_to_index() -> Result<()>{
    Ok(())
}

fn main () -> Result<()>{
    Ok(())
    //if let Err(err) =
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        assert!(false);
    }
}
