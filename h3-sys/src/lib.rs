pub use crate::ffi::{
    compact,
    //    h3SetToMultiPolygon,
    degsToRads,
    //    edgeLength,
    edgeLengthKm,
    edgeLengthM,
    experimentalH3ToLocalIj,
    experimentalLocalIjToH3,
    geoToH3,
    getDestinationH3IndexFromUnidirectionalEdge,
    getH3IndexesFromUnidirectionalEdge,
    getH3UnidirectionalEdge,
    getH3UnidirectionalEdgeBoundary,
    getH3UnidirectionalEdgesFromHexagon,
    getOriginH3IndexFromUnidirectionalEdge,
    getRes0Indexes,
    h3Distance,
    h3GetBaseCell,
    h3GetFaces,
    h3GetResolution,
    h3IndexesAreNeighbors,
    h3IsPentagon,
    h3IsResClassIII,
    h3IsValid,
    h3Line,
    h3ToChildren,
    h3ToGeo,
    h3ToGeoBoundary,
    h3ToParent,
    h3ToString,
    h3UnidirectionalEdgeIsValid,
    //    hexArea,
    hexAreaKm2,
    hexAreaM2,
    hexRing,
    kRing,
    kRingDistances,
    numHexagons,
    polyfill,
    radsToDegs,
    stringToH3,
    uncompact,
    CoordIJ,
    GeoBoundary,
    GeoCoord,
    GeoMultiPolygon,
    GeoPolygon,
    Geofence,
    H3Index,
    LinkedGeoCoord,
    LinkedGeoLoop,
    LinkedGeoPolygon,
    MAX_CELL_BNDRY_VERTS,
};

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::unreadable_literal)]
mod ffi;

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_int;

    #[test]
    fn test_geo_to_h3() {
        unsafe {
            let input = GeoCoord {
                lat: degsToRads(37.3615593),
                lon: degsToRads(-122.0553238),
            };
            let res: c_int = 7;
            let output = geoToH3(&input, res);
            println!("{:X}", output);
            assert_eq!(output, 0x87283472bffffff);
            assert_eq!(format!("{:X}", output), "87283472BFFFFFF")
        }
    }
}
