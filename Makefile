.PHONY: build
build:
	@cargo build

#TODO(mookerji): move to standalone bash file.
# See also:
# unused option: --whitelist-function h3SetToMultiPolygon
# unused option: --whitelist-function hexArea
# unused option: --whitelist-function edgeLength
.PHONY: gen
gen:
	@bindgen \
		--with-derive-default \
		--no-layout-tests \
		--no-doc-comments \
		--whitelist-type 'H3Index' \
		--whitelist-type 'GeoCoord' \
		--whitelist-type 'GeoBoundary' \
		--whitelist-type 'Geofence' \
		--whitelist-type 'GeoPolygon' \
		--whitelist-type 'GeoMultiPolygon' \
		--whitelist-type 'LinkedGeoCoord' \
		--whitelist-type 'LinkedGeoLoop' \
		--whitelist-type 'LinkedGeoPolygon' \
		--whitelist-type 'CoordIJ' \
		--whitelist-var 'MAX_CELL_BNDRY_VERTS' \
		--whitelist-function 'geoToH3' \
		--whitelist-function 'h3ToGeo' \
		--whitelist-function 'h3ToGeoBoundary' \
		--whitelist-function 'kRing' \
		--whitelist-function 'kRingDistances' \
		--whitelist-function 'hexRing' \
		--whitelist-function 'polyfill' \
		--whitelist-function 'h3SetToMultiPolygon' \
		--whitelist-function 'degsToRads' \
		--whitelist-function 'radsToDegs' \
		--whitelist-function 'hexArea' \
		--whitelist-function 'hexAreaKm2' \
		--whitelist-function 'hexAreaM2' \
		--whitelist-function 'edgeLength' \
		--whitelist-function 'edgeLengthKm' \
		--whitelist-function 'edgeLengthM' \
		--whitelist-function 'numHexagons' \
		--whitelist-function 'getRes0Indexes' \
		--whitelist-function 'h3GetResolution' \
		--whitelist-function 'h3GetBaseCell' \
		--whitelist-function 'stringToH3' \
		--whitelist-function 'h3ToString' \
		--whitelist-function 'h3IsValid' \
		--whitelist-function 'h3ToParent' \
		--whitelist-function 'h3ToChildren' \
		--whitelist-function 'compact' \
		--whitelist-function 'uncompact' \
		--whitelist-function 'h3IsResClassIII' \
		--whitelist-function 'h3IsPentagon' \
		--whitelist-function 'h3GetFaces' \
		--whitelist-function 'h3IndexesAreNeighbors' \
		--whitelist-function 'getH3UnidirectionalEdge' \
		--whitelist-function 'h3UnidirectionalEdgeIsValid' \
		--whitelist-function 'getOriginH3IndexFromUnidirectionalEdge' \
		--whitelist-function 'getDestinationH3IndexFromUnidirectionalEdge' \
		--whitelist-function 'getH3IndexesFromUnidirectionalEdge' \
		--whitelist-function 'getH3UnidirectionalEdgesFromHexagon' \
		--whitelist-function 'getH3UnidirectionalEdgeBoundary' \
		--whitelist-function 'h3Distance' \
		--whitelist-function 'h3Line' \
		--whitelist-function 'experimentalH3ToLocalIj' \
		--whitelist-function 'experimentalLocalIjToH3' \
		h3-sys/interface.h \
		-- -std=c11 \
		> h3-sys/src/ffi.rs
