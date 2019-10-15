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
use h3_rs::Error as H3Error;
use h3_rs::{GridResolution, H3Index};
use num_traits::FromPrimitive;
use std::process;

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
                let indices = Vec::new();
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
                let arg = sub_match.value_of("INDEX").unwrap();
                match H3Index::new(*arg.into()) {
                    Ok(index) => Ok(Command::IndexToPoint(index)),
                    Err(err) => Err(Error::LibraryError(err)),
                }
            }
            ("point-to-index", Some(sub_match)) => {
                let lng = sub_match.value_of("lng").unwrap();
                let lat = sub_match.value_of("lat").unwrap();
                let res_val = sub_match.value_of("res").unwrap();
                let res =
                    GridResolution::from_i32(*res_val.into()).expect("GridResolution failed!");
                Ok(Command::PointToIndex(
                    Point::new(*lng.into(), *lat.into()),
                    res,
                ))
            }
            ("boundary-to-index", Some(sub_match)) => Ok(Command::BoundaryToIndex()),
            _ => Err(Error::InvalidSubCommand),
        }
    }
}

fn index_to_boundary(indices: Vec<H3Index>) -> Result<()> {
    for i in 0..indices.len() {
        let region = LineString::from(indices[i]);
        println!("{}", "foo");
    }
    Ok(())
}

fn index_to_point(index: H3Index) -> Result<()> {
    let point = Point::from(index);
    println!("{} {}", point.lng(), point.lat());
    Ok(())
}

fn point_to_index(point: Point<f64>, res: GridResolution) -> Result<()> {
    Ok(())
}

fn boundary_to_index() -> Result<()> {
    Ok(())
}

fn try_main(matches: ArgMatches) -> Result<()> {
    Ok(())
    // Command::IndexToBoundary(Vec<H3Index>),
    // Command::IndexToPoint(H3Index),
    // Command::PointToIndex(Point<f64>, GridResolution),
    // Ok(Command::BoundaryToIndex())
}

fn main() {
    let yaml = load_yaml!("./cli-defs.yaml");
    //if let Err(err) = App::from_yaml(yaml).get_matches_safe().and_then(try_main) {
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
    fn test_main() {
        assert!(false);
    }
}
