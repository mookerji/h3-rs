
> **UNMAINTAINED**: This project is **no longer under development**. Please feel
> free to browse, fork, reuse these FFI bindings for other projects.

## `h3-rs`: Rust Bindings to h3

Rust bindings to [H3][h3_uber], a C library for hexagonal, hierarchical
geospatial indexing. `h3-rs` interoperates between GeoJSON types defined the
[`geo-types` crate][geo_types]

This is **experimental, in-progress software**.

## Installation

`h3-rs` is ~available on [crates.io][crates_h3_rs]~ available for installation
from source. First, checkout this repo:

```bash
$ git clone git@github.com:mookerji/h3-rs.git
$ git checkout tags/v0.1.0
```

Assuming that your downstream project repo and `h3-rs` are in the same repo, add
to your `Cargo.toml`:

```
[dependencies]
h3-rs = { version = "0.1.0", path = "../h3-rs" }
```

`h3-rs` requires that you already have `h3` and its headers installed on your
system. Follow those instructions [from h3][h3_install].

## License

Copyright (c) 2016-2020 Uber Technologies, Inc.
Copyright (c) 2020 Bhaskar Mookerji

Distributed under the Apache License 2.0

[h3_uber]: https://uber.github.io/h3/#/
[h3_install]: https://github.com/uber/h3#installing
[crates_h3_rs]: https://github.com/mookerji/h3-rs
[geo_types]: https://docs.rs/geo-types/
