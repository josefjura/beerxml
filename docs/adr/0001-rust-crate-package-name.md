# Rust crate package name: `beerxml-rs`, library `beerxml`

The Rust reference crate is published as the package `beerxml-rs`, with
`[lib] name = "beerxml"`, so code still writes `beerxml::read`. The name
`beerxml` on crates.io belongs to an abandoned 2017 crate, so we can't use it,
and `beerxml-rs` is the closest name that says what the package is. Until it is
published, consumers depend on it from git by a `rust-vX.Y.Z` tag (the `rust-`
prefix keeps crate tags apart from any future tags for the spec or the Python
library in this repository).
