//! Quantities in canonical BeerXML units.
//!
//! Each unit is its own type wrapping an `f64`, so the compiler stops values
//! of different units from being mixed: there is no arithmetic between them.
//!
//! ```compile_fail
//! use beerxml::units::{Kilograms, Litres};
//! let _ = Kilograms::new(1.0) + Litres::new(1.0);
//! ```
//!
//! Every unit has conversions to common display units, and implements
//! [`Display`](std::fmt::Display) with its symbol, honouring precision:
//!
//! ```
//! use beerxml::units::Kilograms;
//! let amount = Kilograms::new(0.0283495);
//! assert_eq!(format!("{amount:.3}"), "0.028 kg");
//! assert!((amount.ounces() - 1.0).abs() < 1e-6);
//! ```

use std::fmt;

macro_rules! unit {
    ($(#[$doc:meta])* $name:ident, $symbol:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
        pub struct $name(f64);

        impl $name {
            #[doc = concat!("A quantity of `value` ", $symbol, ".")]
            #[must_use]
            pub const fn new(value: f64) -> Self {
                Self(value)
            }

            #[doc = concat!("The value in ", $symbol, ".")]
            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)?;
                f.write_str(concat!(" ", $symbol))
            }
        }
    };
}

const GRAMS_PER_OUNCE: f64 = 28.349_523_125;
const GRAMS_PER_POUND: f64 = 453.592_37;
const LITRES_PER_US_GALLON: f64 = 3.785_411_784;

unit!(
    /// A weight in kilograms, the BeerXML unit for weights.
    Kilograms,
    "kg"
);

impl Kilograms {
    /// The weight in grams.
    #[must_use]
    pub fn grams(self) -> f64 {
        self.0 * 1000.0
    }

    /// The weight in avoirdupois ounces.
    #[must_use]
    pub fn ounces(self) -> f64 {
        self.grams() / GRAMS_PER_OUNCE
    }

    /// The weight in avoirdupois pounds.
    #[must_use]
    pub fn pounds(self) -> f64 {
        self.grams() / GRAMS_PER_POUND
    }
}

unit!(
    /// A volume in litres, the BeerXML unit for volumes.
    Litres,
    "L"
);

impl Litres {
    /// The volume in millilitres.
    #[must_use]
    pub fn millilitres(self) -> f64 {
        self.0 * 1000.0
    }

    /// The volume in US gallons.
    #[must_use]
    pub fn us_gallons(self) -> f64 {
        self.0 / LITRES_PER_US_GALLON
    }
}

unit!(
    /// A duration in minutes, the BeerXML unit for times.
    Minutes,
    "min"
);

impl Minutes {
    /// The duration in hours.
    #[must_use]
    pub fn hours(self) -> f64 {
        self.0 / 60.0
    }

    /// The duration in days.
    #[must_use]
    pub fn days(self) -> f64 {
        self.0 / (60.0 * 24.0)
    }
}

unit!(
    /// A percentage, e.g. `75.0` for 75 %.
    Percent,
    "%"
);

impl Percent {
    /// The percentage as a fraction, e.g. `0.75` for 75 %.
    #[must_use]
    pub fn fraction(self) -> f64 {
        self.0 / 100.0
    }
}

unit!(
    /// A specific gravity relative to water, e.g. `1.050`.
    SpecificGravity,
    "SG"
);

impl SpecificGravity {
    /// Gravity points, e.g. `50.0` for a specific gravity of `1.050`.
    #[must_use]
    pub fn points(self) -> f64 {
        (self.0 - 1.0) * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn weight_conversions() {
        let kg = Kilograms::new(1.0);
        assert!(close(kg.grams(), 1000.0));
        assert!(close(kg.pounds(), 2.204_622_621_8));
        assert!(close(kg.ounces(), 35.273_961_949_6));
    }

    #[test]
    fn volume_conversions() {
        assert!(close(Litres::new(18.927_058_92).us_gallons(), 5.0));
        assert!(close(Litres::new(0.5).millilitres(), 500.0));
    }

    #[test]
    fn time_percent_and_gravity_conversions() {
        assert!(close(Minutes::new(90.0).hours(), 1.5));
        assert!(close(Minutes::new(4320.0).days(), 3.0));
        assert!(close(Percent::new(75.0).fraction(), 0.75));
        assert!(close(SpecificGravity::new(1.043).points(), 43.0));
    }

    #[test]
    fn display_shows_the_symbol_and_honours_precision() {
        assert_eq!(Litres::new(30.0).to_string(), "30 L");
        assert_eq!(format!("{:.1}", Percent::new(4.75)), "4.8 %");
        assert_eq!(format!("{:.3}", SpecificGravity::new(1.05)), "1.050 SG");
    }
}
