use crate::config::{DOTNET_COMMAND, BEATMAPS_CACHE};
use crate::config_functions::performance_calculator_path;
use super::{Mod, Accuracy, UnsuccessfulCommandError};
use std::process::Command;
use std::path::PathBuf;
use std::error::Error;
use std::collections::{HashMap, BTreeSet};
use std::fs;
use std::fs::File;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    accuracy: Accuracy,
    mods: BTreeSet<Mod>,
    combo: Option<usize>,
    misses: Option<usize>
}

fn parse_simulation_results(raw_results: String) -> Result<SimulationResults, Box<Error>> {
    Ok(serde_json::from_str(raw_results.as_str())?)
}

fn get_beatmap_file(beatmap_id: i64) -> Result<String, Box<Error>> {
    let mut osu_path: PathBuf = PathBuf::new();
    osu_path.push(BEATMAPS_CACHE);
    if !osu_path.as_path().exists() {
        fs::create_dir_all(BEATMAPS_CACHE)?;
    }

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

    cmd.arg(performance_calculator_path())
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

        println!("simulate_play failed! output: {}", raw);

        Err(Box::new(UnsuccessfulCommandError))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_beatmaps() {
        println!("{}", performance_calculator_path());
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
                Err(e) => {
                    panic!(format!("simulate_play failed! {}", e));
                }
            }
        }
    }
}
