extern crate serde;
extern crate serde_json;
extern crate reqwest;

use crate::config::{OSU_API_KEY, PERFORMANCE_CALCULATOR_PATH, DOTNET_COMMAND, BEATMAPS_CACHE};
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::vec;
use std::error::Error;
use std::collections::{HashMap, BTreeSet};
use std::fmt;
use std::fs::File;
//use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Mod {
    HD, HR, DT, NC, FL, NF, EZ, HT, SO, SD, PF, TD
}

impl Mod {
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
            TD => "td"
        }
    }

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
            TD => "TD"
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    #[serde(alias = "BeatmapID")]
    beatmap_id: i64,
    #[serde(alias = "BeatmapName")]
    beatmap_name: String,
    #[serde(alias = "Mods")]
    mods: BTreeSet<Mod>, // Use BTreeSet for ordering, makes displaying them as a string easier.
    #[serde(alias = "Accuracy")]
    accuracy: f64,
    #[serde(alias = "LivePP")]
    live_pp: f64,
    #[serde(alias = "LocalPP")]
    local_pp: f64,
    #[serde(alias = "PPDelta")]
    pp_change: f64,
    #[serde(alias = "PositionDelta")]
    position_change: i64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResults {
    #[serde(alias = "Username")]
    user: String,
    #[serde(alias = "LivePP")]
    total_live_pp: f64,
    #[serde(alias = "BonusPP")]
    total_bonus_pp: f64,
    #[serde(alias = "LocalPP")]
    total_local_pp: f64,
    #[serde(alias = "DisplayPlays")]
    scores: Vec<Score>
}

fn parse_results(raw_results: String) -> Result<PerformanceResults, Box<Error>> {
    Ok(serde_json::from_str(raw_results.as_str())?)
}

#[derive(Debug)]
struct UnsuccessfulCommandError;
impl fmt::Display for UnsuccessfulCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsuccessful command")
    }
}
impl Error for UnsuccessfulCommandError {}

pub fn calculate_performance(user: String) -> Result<PerformanceResults, Box<Error>> {
    let output = Command::new(DOTNET_COMMAND)
                           .arg(PERFORMANCE_CALCULATOR_PATH)
                           .arg("profile")
                           .arg(user)
                           .arg(OSU_API_KEY)
                           .arg("--json")
                           //.current_dir(Path::new(PERFORMANCE_CALCULATOR_PATH).parent().unwrap())
                           .output()?;
    
    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(parse_results(raw)?)
    } else {
        Err(Box::new(UnsuccessfulCommandError))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Accuracy {
    Percentage(f64),
    Hits { good: usize, meh: usize }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayInfo {
    #[serde(alias = "Accuracy")]
    accuracy: f64,
    #[serde(alias = "Combo")]
    combo: i64,
    #[serde(alias = "MaxCombo")]
    max_combo: i64,
    #[serde(alias = "Great")]
    great: i64,
    #[serde(alias = "Good")]
    good: i64,
    #[serde(alias = "Meh")]
    meh: i64,
    #[serde(alias = "Miss")]
    miss: i64,
}

// Simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResults {
    #[serde(alias = "BeatmapInfo")]
    beatmap_info: String,
    #[serde(alias = "Mods")]
    mods: BTreeSet<Mod>,
    #[serde(alias = "PlayInfo")]
    play_info: PlayInfo,
    #[serde(alias = "CategoryAttribs")]
    category_attribs: HashMap<String, f64>,
    #[serde(alias = "PP")]
    pp: f64

}

fn parse_simulation_results(raw_results: String) -> Result<SimulationResults, Box<Error>> {
    Ok(serde_json::from_str(raw_results.as_str())?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    accuracy: Accuracy,
    mods: BTreeSet<Mod>,
    combo: Option<usize>,
    misses: Option<usize>
}

fn get_beatmap_file(beatmap_id: i64) -> Result<String, Box<Error>> {
    let mut osu_path: PathBuf = PathBuf::new();
    osu_path.push(BEATMAPS_CACHE);
    osu_path.push(format!("{}.osu", beatmap_id));

    if !osu_path.as_path().exists() {
        // Download it then...
        let mut resp = reqwest::get(&format!("https://osu.ppy.sh/osu/{}", beatmap_id))?;
        let mut file = File::create(osu_path.as_path())?;
        resp.copy_to(&mut file)?;
    }

    Ok(osu_path.to_str().unwrap().to_string())
}

pub fn simulate_play(beatmap_id: i64, params: SimulationParams) -> Result<SimulationResults, Box<Error>> {
    let mut cmd = Command::new(DOTNET_COMMAND);

    cmd.arg(PERFORMANCE_CALCULATOR_PATH)
       .arg("simulate")
       .arg("osu");
    
    let beatmap = get_beatmap_file(beatmap_id)?;
    cmd.arg(beatmap);

    match params.accuracy {
        Accuracy::Percentage(pct) => cmd.arg("-a").arg(format!("{:.*}", 2, pct)),
        Accuracy::Hits { good, meh } => cmd.arg("-G").arg(good.to_string()).arg("-M").arg(meh.to_string())
    };

    for m in params.mods {
        cmd.arg("-m").arg(m.to_arg());
    }

    if let Some(combo) = params.combo {
        cmd.arg("-c").arg(combo.to_string());
    }

    if let Some(misses) = params.misses {
        cmd.arg("-X").arg(misses.to_string());
    }

    cmd.arg("--json");

    let output = cmd.output()?;
    
    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(parse_simulation_results(raw)?)
    } else {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        println!("{}", raw);

        Err(Box::new(UnsuccessfulCommandError))
    }
}

// Tests
// I think these tests should be on their own directory, but there's no way to test binaries
// like this... So this will be here
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mod_order() {
        let mut mod_list = BTreeSet::new();
        mod_list.insert(Mod::DT);
        mod_list.insert(Mod::HR);
        mod_list.insert(Mod::HD);

        let mod_vec : Vec<_> = mod_list.into_iter().collect();
        assert_eq!(mod_vec, [Mod::HD, Mod::HR, Mod::DT]);
    }

    // Calculate a few profiles, just to be sure everything is OK.
    #[test]
    fn test_calculate_profiles() {
        let players = vec!["rafis", "mathi", "yeahbennou", "freedomdiver"];

        for player in players {
            let result = calculate_performance(player.to_string());

            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_calculate_beatmaps() {
        use Mod::*;

        let data = vec![
            // Cookiezi's Freedom Dive
            (129891, Accuracy::Percentage(99.83f64), mods![HD, HR], None, 898f64),
            // Rafis' Necrofantasia
            (1097543, Accuracy::Hits { good: 21, meh: 0 }, mods![HD, DT], Some(1627), 792f64)
        ];

        for (beatmap_id, acc, mods, combo, pp) in data {
            let params = SimulationParams {
                accuracy: acc,
                mods: mods,
                combo: combo,
                misses: None
            };

            match simulate_play(beatmap_id, params) {
                Ok(result) => {
                    // who cares about decimal places
                    assert_eq!(result.pp.trunc(), pp);
                },
                Err(_) => {
                    panic!("simulate_play failed!");
                }
            }
        }
    }
}
