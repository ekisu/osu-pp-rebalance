extern crate serde;

use crate::config::{OSU_API_KEY, PERFORMANCE_CALCULATOR_PATH, DOTNET_COMMAND};
use std::process::{Command, Stdio};
use std::path::Path;
use std::vec;
//use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    song_name: String,
    live_pp: String,
    local_pp: String,
    pp_change: String,
    position_change: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResults {
    user: String,
    total_live_pp: String,
    total_local_pp: String,
    scores: Vec<Score>
}

static SEPARATORS : &'static str = r#"��║│║"#; // ?

fn is_separator(c: char) -> bool {
    SEPARATORS.contains(c)
}

fn parse_results(raw_results: String) -> Option<PerformanceResults> {
    let mut results = PerformanceResults {
        user: String::new(),
        total_live_pp: String::new(),
        total_local_pp: String::new(),
        scores: Vec::new()
    };

    // skip download messages and shit
    let mut iter = raw_results.split("\n").skip_while(|l| !l.starts_with("User: "));

    results.user = iter.next()?.clone().to_string();
    results.total_live_pp = iter.next()?.clone().to_string();
    results.total_local_pp = iter.next()?.clone().to_string();

    // white line, table graphic line, header, graphic line
    iter.next()?;
    iter.next()?;
    iter.next()?;
    iter.next()?;

    results.scores = iter.step_by(2).filter(|l| !l.is_empty()).map(|l| {
        let fields : Vec<&str> = l.split(|c| is_separator(c)).filter(|s| !s.is_empty()).collect();

        if fields.len() < 5 {
            None 
        } else {
            Some(Score {
                song_name: fields[0].trim().to_string(),
                live_pp: fields[1].trim().to_string(),
                local_pp: fields[2].trim().to_string(),
                pp_change: fields[3].trim().to_string(),
                position_change: fields[4].trim().to_string()
            })
        }
    }).filter_map(|x| x).collect();

    Some(results)
}

pub fn calculate_performance(user: String) -> Result<PerformanceResults, String> {
    let output = Command::new(DOTNET_COMMAND)
                           .arg(PERFORMANCE_CALCULATOR_PATH)
                           .arg("profile")
                           .arg(user)
                           .arg(OSU_API_KEY)
                           //.current_dir(Path::new(PERFORMANCE_CALCULATOR_PATH).parent().unwrap())
                           .output();
    
    match output {
        Ok(res) => {
            if res.status.success() {
                let raw = String::from_utf8_lossy(&res.stdout).to_string();

                if let Some(performance) = parse_results(raw) {
                    Ok(performance)
                } else {
                    Err(String::from("couldn't parse results"))
                }
            } else {
                Err(String::from("failed to calculate performance"))
            }
        },
        Err(_) => Err(String::from("failed to calculate performance"))
    }
}
