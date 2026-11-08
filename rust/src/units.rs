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
//! Units have conversions to common display units where they exist, and implement
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

unit!(
    /// A temperature in degrees Celsius, the BeerXML unit for temperatures.
    Celsius,
    "°C"
);

impl Celsius {
    /// The temperature in degrees Fahrenheit.
    #[must_use]
    pub fn fahrenheit(self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}

unit!(
    /// A colour in SRM (Standard Reference Method), the BeerXML unit for
    /// colours.
    Srm,
    "SRM"
);

impl Srm {
    /// The colour in EBC, by the common approximation `EBC = SRM × 1.97`.
    #[must_use]
    pub fn ebc(self) -> f64 {
        self.0 * 1.97
    }

    /// The colour in degrees Lovibond, by the common approximation
    /// `SRM = 1.3546 × °L − 0.76`.
    #[must_use]
    pub fn lovibond(self) -> f64 {
        (self.0 + 0.76) / 1.3546
    }
}

unit!(
    /// A concentration in parts per million (mg/L), the BeerXML unit for
    /// water minerals. In water, ppm and mg/L are the same.
    Ppm,
    "ppm"
);

unit!(
    /// Diastatic power in degrees Lintner.
    Lintner,
    "°Lintner"
);

impl Lintner {
    /// The diastatic power in degrees Windisch-Kolbach, by the common
    /// approximation `°WK = 3.5 × °L − 16`.
    #[must_use]
    pub fn windisch_kolbach(self) -> f64 {
        self.0 * 3.5 - 16.0
    }
}

unit!(
    /// A duration in days, the BeerXML unit for fermentation and ageing
    /// times (e.g. [`Recipe::primary_age`](crate::Recipe::primary_age)).
    Days,
    "d"
);

impl Days {
    /// The duration in weeks.
    #[must_use]
    pub fn weeks(self) -> f64 {
        self.0 / 7.0
    }
}

unit!(
    /// Bitterness in International Bitterness Units.
    Ibu,
    "IBU"
);

unit!(
    /// Carbonation in volumes of CO₂ per volume of beer.
    Co2Volumes,
    "vol"
);

unit!(
    /// A specific heat in calories per gram per degree Celsius, the BeerXML
    /// unit for the specific heat of tun materials.
    SpecificHeat,
    "cal/(g·°C)"
);

impl SpecificHeat {
    /// The specific heat in joules per kilogram per kelvin
    /// (1 cal/(g·°C) = 4184 J/(kg·K), using the thermochemical calorie).
    #[must_use]
    pub fn joules_per_kg_kelvin(self) -> f64 {
        self.0 * 4184.0
    }
}

/// An amount that is a weight or a volume, as chosen by the record's
/// `AMOUNT_IS_WEIGHT` flag (used by Misc and Yeast records).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Amount {
    /// `AMOUNT_IS_WEIGHT` is `TRUE`: the amount is in kilograms.
    Weight(Kilograms),
    /// `AMOUNT_IS_WEIGHT` is `FALSE` or absent (the spec's default): the
    /// amount is in litres.
    Volume(Litres),
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Weight(kg) => fmt::Display::fmt(kg, f),
            Self::Volume(l) => fmt::Display::fmt(l, f),
        }
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
        assert!(close(Days::new(14.0).weeks(), 2.0));
        assert!(close(Percent::new(75.0).fraction(), 0.75));
        assert!(close(SpecificGravity::new(1.043).points(), 43.0));
    }

    #[test]
    fn temperature_colour_mineral_and_diastatic_conversions() {
        assert!(close(Celsius::new(100.0).fahrenheit(), 212.0));
        assert!(close(Celsius::new(-40.0).fahrenheit(), -40.0));
        assert!(close(Celsius::new(20.0).fahrenheit(), 68.0));
        assert!(close(Srm::new(10.0).ebc(), 19.7));
        assert!(close(Srm::new(1.3546 * 4.0 - 0.76).lovibond(), 4.0));
        assert_eq!(Ppm::new(295.0).to_string(), "295 ppm");
        assert!(close(Lintner::new(100.0).windisch_kolbach(), 334.0));
        assert!(close(
            SpecificHeat::new(0.12).joules_per_kg_kelvin(),
            502.08
        ));
    }

    #[test]
    fn amount_displays_its_unit() {
        assert_eq!(Amount::Weight(Kilograms::new(0.5)).to_string(), "0.5 kg");
        assert_eq!(
            format!("{:.2}", Amount::Volume(Litres::new(0.035))),
            "0.04 L"
        );
        assert_eq!(format!("{:.1}", Celsius::new(18.26)), "18.3 °C");
    }

    #[test]
    fn display_shows_the_symbol_and_honours_precision() {
        assert_eq!(Litres::new(30.0).to_string(), "30 L");
        assert_eq!(format!("{:.1}", Percent::new(4.75)), "4.8 %");
        assert_eq!(format!("{:.3}", SpecificGravity::new(1.05)), "1.050 SG");
        assert_eq!(Ibu::new(32.4).to_string(), "32.4 IBU");
        assert_eq!(Co2Volumes::new(2.4).to_string(), "2.4 vol");
        assert_eq!(Days::new(7.0).to_string(), "7 d");
    }
}
