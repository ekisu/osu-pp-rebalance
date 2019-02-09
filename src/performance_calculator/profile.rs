use super::{Mod, UnsuccessfulCommandError};
use crate::config_functions::{api_key, dotnet_command, performance_calculator_path};
use std::collections::BTreeSet;
use std::error::Error;
use std::process::Command;

/// A single play, with both live (old) and local (new) PP results.
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
    position_change: i64,
}

/// The result of a PP calculation for a osu! profile. Contains the list of
/// scores (from the user actual top 100 plays), ordered by local (new) PP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResults {
    #[serde(alias = "Username")]
    user: String,
    #[serde(alias = "LivePP")]
    total_live_pp: f64,
    #[serde(alias = "BonusPP")]
    total_bonus_pp: f64,
    #[serde(alias = "LocalPP")]
    total_local_pp: f64,
    #[serde(alias = "DisplayPlays")]
    scores: Vec<Score>,
}

/// Parses the output from PerformanceCalculator (`raw_results`) into a ProfileResults struct
fn parse_profile_results(raw_results: String) -> Result<ProfileResults, Box<Error>> {
    Ok(serde_json::from_str(raw_results.as_str())?)
}

/// Calculates the new PP system scores for a osu! user profile. `user`, preferably, should
/// be a user id, but it can also be the user name.
pub fn calculate_profile(user: String) -> Result<ProfileResults, Box<Error>> {
    let output = Command::new(dotnet_command())
        .arg(performance_calculator_path())
        .arg("profile")
        .arg(user)
        .arg(api_key())
        .arg("--json")
        //.current_dir(Path::new(PERFORMANCE_CALCULATOR_PATH).parent().unwrap())
        .output()?;

    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(parse_profile_results(raw)?)
    } else {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        println!("calculate_profile failed! output: {}", raw);

        Err(Box::new(UnsuccessfulCommandError))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Calculate a few profiles, just to be sure everything is OK.
    #[test]
    fn test_calculate_profiles() {
        let players = vec!["rafis", "mathi", "yeahbennou", "freedomdiver"];

        for player in players {
            let result = calculate_profile(player.to_string());

            if let Err(e) = result {
                panic!("calculate_profile for {} failed! {}", player, e);
            }
        }
    }
}
