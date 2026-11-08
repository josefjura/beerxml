use crate::enums::{HopForm, HopType, HopUse};
use crate::units::{Kilograms, Litres, Minutes, Percent, SpecificGravity};

/// A BeerXML document: the records of its top-level list.
///
/// BeerXML files hold a single top-level list, so at most one of these is
/// non-empty. Record types that are not modelled yet are accepted (the file
/// still reads) but their records are not returned.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Document {
    /// Recipes from a `RECIPES` list, in document order.
    pub recipes: Vec<Recipe>,
    /// Hops from a top-level `HOPS` list (an ingredient library), in
    /// document order. Hops used by a recipe are on [`Recipe::hops`].
    pub hops: Vec<Hop>,
}

/// A `RECIPE` record.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Recipe {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `BATCH_SIZE`: target size of the finished batch.
    pub batch_size: Option<Litres>,
    /// `BOIL_SIZE`: starting pre-boil volume.
    pub boil_size: Option<Litres>,
    /// `BOIL_TIME`: boil length.
    pub boil_time: Option<Minutes>,
    /// `EFFICIENCY`: brewhouse efficiency.
    pub efficiency: Option<Percent>,
    /// `OG`: measured original gravity.
    pub og: Option<SpecificGravity>,
    /// `FG`: measured final gravity.
    pub fg: Option<SpecificGravity>,
    /// `HOPS`: the hop additions, in document order.
    pub hops: Vec<Hop>,
}

/// A `HOP` record: a hop variety, or a hop addition when inside a recipe.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Hop {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `ALPHA`: alpha acid content.
    pub alpha: Option<Percent>,
    /// `AMOUNT`: weight of the addition. Hop amounts are always weights.
    pub amount: Option<Kilograms>,
    /// `USE`: when the hop is added.
    pub usage: Option<HopUse>,
    /// `TIME`: how long the hop is used; its meaning depends on
    /// [`usage`](Self::usage), e.g. boil time for `Boil`, contact time for
    /// `Dry Hop`.
    pub time: Option<Minutes>,
    /// `NOTES`.
    pub notes: Option<String>,
    /// `TYPE`: bittering, aroma or both.
    pub kind: Option<HopType>,
    /// `FORM`: pellet, plug or leaf.
    pub form: Option<HopForm>,
    /// `BETA`: beta acid content.
    pub beta: Option<Percent>,
    /// `HSI`: hop stability index, the percentage of alpha acid lost after
    /// six months at room temperature.
    pub hsi: Option<Percent>,
    /// `ORIGIN`: where the hop was grown.
    pub origin: Option<String>,
    /// `SUBSTITUTES`: suggested substitutes.
    pub substitutes: Option<String>,
    /// `HUMULENE`: humulene content.
    pub humulene: Option<Percent>,
    /// `CARYOPHYLLENE`: caryophyllene content.
    pub caryophyllene: Option<Percent>,
    /// `COHUMULONE`: cohumulone content.
    pub cohumulone: Option<Percent>,
    /// `MYRCENE`: myrcene content.
    pub myrcene: Option<Percent>,
}
