//! Reads a BeerXML file leniently and prints its recipes and every repair.
//!
//! ```text
//! cargo run --example read -- ../samples/original/recipes.xml
//! ```
//!
//! This file is also the example on the docs page (`docs/rust.md`), so the
//! example there always compiles.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: read <file.xml>")?;
    let bytes = std::fs::read(path)?;

    let read = beerxml::read(&bytes)?;
    println!("BeerXML {}", read.version);
    for recipe in &read.document.recipes {
        let litres = recipe.batch_size.map(beerxml::units::Litres::get);
        println!("{}: {litres:?} l, {} hops", recipe.name, recipe.hops.len());
    }
    for warning in &read.warnings {
        println!("repaired {}: {:?}", warning.path, warning.kind);
    }
    Ok(())
}
