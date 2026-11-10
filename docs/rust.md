# Rust crate

The [`rust/`](https://github.com/josefjura/beerxml/tree/master/rust) directory
holds the Rust reference implementation of BeerXML. It reads BeerXML 1.0 and
1.1 files into typed records, and validates files strictly against the 1.0 or
1.1 schema, giving the same verdict on every sample as the Python validator in
[`lib/`](https://github.com/josefjura/beerxml/tree/master/lib).

The package is called `beerxml-rs` (the name `beerxml` is taken on crates.io by
an abandoned crate), but the library is imported as `beerxml`.

## Two modes

| | `beerxml::read` (lenient) | `beerxml::validate` (strict) |
| :--- | :--- | :--- |
| **For** | Files you receive: imports, uploads | Files you write: exporters, tests |
| **Input** | Any real-world BeerXML 1.0 or 1.1 file | A file and the version to check it against |
| **Returns** | The typed `Document`, the detected `Version` and every repair as a `Warning` | Every `Violation` of the schema; none means valid |
| **Fails on** | Not XML, an unsupported encoding, or a root that isn't a BeerXML list | The same fatal problems only |

**Lenient reading** takes what real writers produce and repairs it: it reads
the number in `<IBU>34.2 IBU</IBU>`, matches `boil` to `Boil`, reads
ISO-8859-1 files, and skips `DISPLAY_*` and unknown elements. Each repair is
reported, so nothing is changed silently. Every rule, and the part of the
[1.0 spec](spec/v1.0/v1.0.md) or [RFC 01](spec/v1.1/RFC_01_goals.md) it comes
from, is listed in the crate's rustdoc under "Reading rules".

**Strict validation** checks a file against the
[1.0](spec/v1.0/beerxml-1.0.xsd) or [1.1](spec/v1.1/beerxml-1.1.xsd) XSD:
required fields, types and allowed values. For 1.1 it also enforces RFC 01's
"Clean XML" rules: the 1.1 namespace, `VERSION` 1.1 in every record, pure
numbers, ISO dates, no `DISPLAY_*` fields and UTF-8.

Reading is safe on untrusted uploads: DTDs are refused (so no entity
expansion), the crate does no I/O, and cost grows linearly with input size.

## Depend on it

The crate isn't on crates.io yet. Depend on it from git, pinned to a tag:

```toml
[dependencies]
beerxml = { package = "beerxml-rs", git = "https://github.com/josefjura/beerxml", tag = "rust-v0.1.0" }
```

Cargo finds the package in the `rust/` directory by itself. The `package`
key lets you write `beerxml::read` in your code.

## Read a file

```rust
--8<-- "rust/examples/read.rs"
```

This is [`rust/examples/read.rs`](https://github.com/josefjura/beerxml/blob/master/rust/examples/read.rs),
built by the crate's tests. Run it on a sample:

```bash
cd rust
cargo run --example read -- ../samples/original/recipes.xml
```

## Validate a file

```rust
use beerxml::Version;

let violations = beerxml::validate(&bytes, Version::V1_1)?;
for violation in &violations {
    println!("{violation}");
}
```

## API documentation

Build the full API documentation, including the reading rules, with:

```bash
cd rust
cargo doc --open
```
