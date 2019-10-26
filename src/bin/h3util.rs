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

//! CLI entrypoint to h3-rs

#[macro_use]
extern crate clap;
extern crate geo_types;
extern crate h3_rs;

use clap::{App, ArgMatches};
use geo_types::{LineString, Point};
use geojson::{Feature, FeatureCollection, Geometry, Value};
use h3_rs::Error as H3Error;
use h3_rs::{GridResolution, H3Index, ToH3Index};
use num_traits::FromPrimitive;
use std::process;

/// CLI Errors
#[derive(Debug, PartialEq)]
pub enum Error {
    LibraryError(H3Error),
    InvalidSubCommand,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::LibraryError(arg) => arg.fmt(f),
            Error::InvalidSubCommand => write!(f, "Invalid subcommand!"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// CLI Commands
#[derive(Clone, Debug, PartialEq)]
enum Command {
    IndexToBoundary(Vec<H3Index>),
    IndexToPoint(H3Index),
    PointToIndex(Point<f64>, GridResolution),
    BoundaryToIndex(),
}

impl Command {
    fn from_args<'a>(matches: ArgMatches<'a>) -> Result<Command> {
        match matches.subcommand() {
            ("index-to-boundary", Some(sub_match)) => {
                let mut indices = Vec::new();
                for entry in sub_match.value_of("INDEX").unwrap().split(" ") {
                    let val = (*entry).parse::<u64>().expect("Invalid argument!");
                    match H3Index::new(val) {
                        Ok(index) => indices.push(index),
                        Err(err) => return Err(Error::LibraryError(err)),
                    }
                }
                Ok(Command::IndexToBoundary(indices))
            }
            ("index-to-centroid", Some(sub_match)) => {
                let val = sub_match
                    .value_of("INDEX")
                    .unwrap()
                    .parse::<u64>()
                    .expect("Invalid argument!");
                match H3Index::new(val) {
                    Ok(index) => Ok(Command::IndexToPoint(index)),
                    Err(err) => Err(Error::LibraryError(err)),
                }
            }
            ("point-to-index", Some(sub_match)) => {
                let lng = sub_match
                    .value_of("lng")
                    .unwrap()
                    .parse::<f64>()
                    .expect("Invalid argument!");;
                let lat = sub_match
                    .value_of("lat")
                    .unwrap()
                    .parse::<f64>()
                    .expect("Invalid argument!");;
                let res_val = sub_match
                    .value_of("res")
                    .unwrap()
                    .parse::<i32>()
                    .expect("Invalid argument!");
                let res = GridResolution::from_i32(res_val).expect("GridResolution failed!");
                Ok(Command::PointToIndex(Point::new(lng, lat), res))
            }
            ("boundary-to-index", Some(sub_match)) => Ok(Command::BoundaryToIndex()),
            _ => Err(Error::InvalidSubCommand),
        }
    }
}

/// CLI handler for index-to-boundary
fn index_to_boundary(indices: Vec<H3Index>) -> Result<()> {
    let mut boundaries = Vec::new();
    for i in 0..indices.len() {
        let region: LineString<f64> = indices[i].clone().into();
        let val = Value::from(&region);
        boundaries.push(Feature {
            bbox: None,
            geometry: Some(Geometry::new(val)),
            id: None,
            properties: None,
            foreign_members: None
        });
    }
    println!("{}", FeatureCollection {
        bbox: None,
        features: boundaries,
        foreign_members: None,
    }.to_string());
    Ok(())
}

/// CLI handler for index-to-point
fn index_to_point(index: H3Index) -> Result<()> {
    let point = Point::from(index);
    println!("{} {}", point.lng(), point.lat());
    Ok(())
}

/// CLI handler for point-to-index
fn point_to_index(point: Point<f64>, res: GridResolution) -> Result<()> {
    match point.to_h3_index(res) {
        Ok(index) => println!("{}", index),
        Err(err) => eprintln!("{}", err)
    }
    Ok(())
}
/// CLI handler for boundary-to-index
fn boundary_to_index() -> Result<()> {
    Ok(())
}

fn try_main(matches: ArgMatches) -> Result<()> {
    match Command::from_args(matches) {
        Ok(cmd) => match cmd {
            Command::IndexToBoundary(indices) => index_to_boundary(indices),
            Command::IndexToPoint(index) => index_to_point(index),
            Command::PointToIndex(point, res) => point_to_index(point, res),
            Command::BoundaryToIndex() => boundary_to_index(),
        },
        Err(err) => Err(err)
    }
}

fn main() {
    let yaml = load_yaml!("./cli-defs.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    if let Err(err) = try_main(matches) {
        eprintln!("{}", err);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
        assert!(false);
    }

    #[test]
    fn test_index_to_boundary() {
        assert!(false);
    }

    #[test]
    fn test_index_to_point() {
        assert!(false);
    }

    #[test]
    fn test_point_to_index() {
        assert!(false);
    }
}
