extern crate serde;
extern crate serde_json;
extern crate reqwest;

use crate::config::{OSU_API_KEY, PERFORMANCE_CALCULATOR_PATH, DOTNET_COMMAND, BEATMAPS_CACHE};
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::vec;
use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
//use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    #[serde(alias = "BeatmapID")]
    beatmap_id: i64,
    #[serde(alias = "BeatmapName")]
    beatmap_name: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Mod {
    HD, HR, DT, FL, NF, EZ, HT, SO
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
    mods: Vec<Mod>,
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

impl Mod {
    fn to_arg(&self) -> &'static str {
        use Mod::*;

        match *self {
            HD => "hd",
            HR => "hr",
            DT => "dt",
            FL => "fl",
            NF => "nf",
            EZ => "ez",
            HT => "ht",
            SO => "so"
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    accuracy: Accuracy,
    mods: Vec<Mod>,
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
