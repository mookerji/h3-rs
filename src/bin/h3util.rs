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

/// CLI Errors
#[derive(Debug)]
pub enum Error {
    LibraryError(H3Error),
    InvalidSubCommand,
    ClapError(clap::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::LibraryError(arg) => arg.fmt(f),
            Error::InvalidSubCommand => write!(f, "Invalid subcommand!"),
            Error::ClapError(err) => err.fmt(f),
        }
    }
}

impl From<Error> for clap::Error {
    fn from(err: Error) -> Self {
        clap::Error {
            message: format!("{}", err),
            kind: clap::ErrorKind::InvalidValue,
            info: None,
        }
    }
}

impl From<clap::Error> for Error {
    fn from(err: clap::Error) -> Self {
        Error::ClapError(err)
    }
}

/// Output format
#[derive(Clone, Debug, PartialEq)]
enum OutputFormat {
    GeoJSON,
    Text,
}

/// CLI Commands
#[derive(Clone, Debug, PartialEq)]
enum Command {
    IndexToBoundary(Vec<H3Index>),
    IndexToPoint(H3Index),
    PointToIndex(Point<f64>, GridResolution),
    BoundaryToIndex(),
    IndexToComponents(H3Index),
    IndexToHexRange(H3Index, u32),
    IndexToKRing(H3Index, u32),
}

// TODO:
// Should these e in a library function?
// read from stdout

//.unwrap_or_else(|err| format!("{}", err))
// let idx_val = value_t!(matched, "INDEX", u64).expect("Invalid argument!");

// match value_t!(matched, "INDEX", H3Index) {
//     Ok(index) => Ok(Command::IndexToPoint(index)),
//     Err(err) => Err(Error::LibraryError(err)),
// }

impl Command {
    fn from_args<'a>(matches: ArgMatches<'a>) -> Result<Command> {
        match matches.subcommand() {
            ("index-to-boundary", Some(matched)) => {
                let mut indices = Vec::new();
                for entry in matched.value_of("INDEX").unwrap().split(" ") {
                    let val = (*entry).parse::<u64>().expect("Invalid argument!");
                    match H3Index::new(val) {
                        Ok(index) => indices.push(index),
                        Err(err) => return Err(Error::LibraryError(err)),
                    }
                }
                Ok(Command::IndexToBoundary(indices))
            }
            ("index-to-centroid", Some(matched)) => {
                let index = value_t!(matched, "INDEX", H3Index)?;
                Ok(Command::IndexToPoint(index))
            }
            ("point-to-index", Some(matched)) => {
                let lng = value_t!(matched, "lng", f64).expect("Invalid longitude argument!");
                let lat = value_t!(matched, "lat", f64).expect("Invalid latitude argument!");
                let res = value_t!(matched, "res", GridResolution)?;
                Ok(Command::PointToIndex(Point::new(lng, lat), res))
            }
            ("boundary-to-index", Some(matched)) => Ok(Command::BoundaryToIndex()),
            ("index-to-components", Some(matched)) => {
                let idx_val = value_t!(matched, "INDEX", u64).expect("Invalid argument!");
                match H3Index::new(idx_val) {
                    Ok(index) => Ok(Command::IndexToComponents(index)),
                    Err(err) => Err(Error::LibraryError(err)),
                }
            }
            ("index-to-hex-range", Some(matched)) => {
                let index = value_t!(matched, "INDEX", H3Index)?;
                let k_distance = value_t!(matched, "distance", u32).expect("Invalid k-distance!");
                Ok(Command::IndexToHexRange(index, k_distance))
            }
            ("index-to-k-ring", Some(matched)) => {
                let k_distance = value_t!(matched, "distance", u32).expect("Invalid k-distance!");
                let idx_val = value_t!(matched, "INDEX", u64).expect("Invalid argument!");
                match H3Index::new(idx_val) {
                    Ok(index) => Ok(Command::IndexToKRing(index, k_distance)),
                    Err(err) => Err(Error::LibraryError(err)),
                }
            }
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
            foreign_members: None,
        });
    }
    println!(
        "{}",
        FeatureCollection {
            bbox: None,
            features: boundaries,
            foreign_members: None,
        }
        .to_string()
    );
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
        Err(err) => eprintln!("{}", err),
    }
    Ok(())
}

/// CLI handler for boundary-to-indexn
fn boundary_to_index() -> Result<()> {
    Ok(())
}

/// CLI handler for boundary-to-index
fn index_to_components(index: H3Index) -> Result<()> {
    Ok(())
}

/// CLI handler for boundary-to-index
fn index_to_hex_range(index: H3Index, distance: u32) -> Result<()> {
    Ok(())
}

/// CLI handler for boundary-to-index
fn index_to_k_ring(index: H3Index, distance: u32) -> Result<()> {
    Ok(())
}

fn try_main(matches: ArgMatches) -> Result<()> {
    match Command::from_args(matches) {
        Ok(cmd) => match cmd {
            Command::IndexToBoundary(indices) => index_to_boundary(indices),
            Command::IndexToPoint(index) => index_to_point(index),
            Command::PointToIndex(point, res) => point_to_index(point, res),
            Command::BoundaryToIndex() => boundary_to_index(),
            Command::IndexToComponents(index) => index_to_components(index),
            Command::IndexToHexRange(index, distance) => index_to_hex_range(index, distance),
            Command::IndexToKRing(index, distance) => index_to_k_ring(index, distance),
        },
        Err(err) => Err(err),
    }
}

fn main() {
    let yaml = load_yaml!("./cli-defs.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    if let Err(err) = try_main(matches) {
        eprintln!("{}", err);
        std::process::exit(1);
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
