//! A interface to osu-tools' PerformanceCalculator.dll
//!
//! This module contains a few data structures/enums common to both
//! profile calculation and simulation requests. Specialized functions
//! can be found into the `profile` and `simulate` modules.
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::fmt;

#[macro_use]
macro_rules! mods {
    ( $( $mod:expr ),* ) => {
        {
            let mut temp_mods : BTreeSet<Mod> = BTreeSet::new();

            $(
                temp_mods.insert($mod);
            )*
            temp_mods
        }
    }
}

/// A enum, representing all possible mods in osu!standard.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Mod {
    HD,
    HR,
    DT,
    NC,
    FL,
    NF,
    EZ,
    HT,
    SO,
    SD,
    PF,
    TD,
}

impl Mod {
    /// Obtain a string representation of the mod, suitable to pass as a mod
    /// parameter to PerformanceCalculator.
    fn to_arg(&self) -> &'static str {
        use Mod::*;

        match *self {
            HD => "hd",
            HR => "hr",
            DT => "dt",
            NC => "nc",
            FL => "fl",
            NF => "nf",
            EZ => "ez",
            HT => "ht",
            SO => "so",
            SD => "sd",
            PF => "pf",
            TD => "td",
        }
    }

    /// Obtain a human-readable representation for the mod.
    fn to_string(&self) -> &'static str {
        use Mod::*;

        // uhh
        match *self {
            HD => "HD",
            HR => "HR",
            DT => "DT",
            NC => "NC",
            FL => "FL",
            NF => "NF",
            EZ => "EZ",
            HT => "HT",
            SO => "SO",
            SD => "SD",
            PF => "PF",
            TD => "TD",
        }
    }
}

/// A data type that represents the Accuracy of a play in osu!standard.
///
/// Can either be a *Percentage*, or *Hits*, which contains the number of
/// non-300s (perfect) hits of a play: good (100s) and meh (50s).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Accuracy {
    Percentage(f64),
    Hits { good: usize, meh: usize },
}

/// An error that can be returned when the external command call fails.
#[derive(Debug)]
struct UnsuccessfulCommandError;
impl fmt::Display for UnsuccessfulCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsuccessful command")
    }
}

impl Error for UnsuccessfulCommandError {}

pub mod profile;
pub use profile::{calculate_profile, ProfileResults};

pub mod simulate;
pub use simulate::{simulate_play, SimulationParams, SimulationResults};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mod_order() {
        use std::collections::BTreeSet;

        let mut mod_list = BTreeSet::new();
        mod_list.insert(Mod::DT);
        mod_list.insert(Mod::HR);
        mod_list.insert(Mod::HD);

        let mod_vec: Vec<_> = mod_list.into_iter().collect();
        assert_eq!(mod_vec, [Mod::HD, Mod::HR, Mod::DT]);
    }
}
