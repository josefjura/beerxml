/// A BeerXML document: the records of its top-level list.
///
/// Record types that are not modelled yet are accepted (the file still reads)
/// but their records are not returned.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Document {
    /// Recipes from a `RECIPES` list, in document order.
    pub recipes: Vec<Recipe>,
}

/// A `RECIPE` record.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Recipe {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `BATCH_SIZE`: target size of the finished batch, in litres.
    pub batch_size: Option<f64>,
    /// `BOIL_SIZE`: starting pre-boil volume, in litres.
    pub boil_size: Option<f64>,
    /// `BOIL_TIME`: boil length, in minutes.
    pub boil_time: Option<f64>,
    /// `EFFICIENCY`: brewhouse efficiency, in percent.
    pub efficiency: Option<f64>,
    /// `OG`: measured original gravity (specific gravity).
    pub og: Option<f64>,
    /// `FG`: measured final gravity (specific gravity).
    pub fg: Option<f64>,
}
