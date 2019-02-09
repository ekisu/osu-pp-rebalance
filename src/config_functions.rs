use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

/// Tries reading a value from the `key` env variable, and casts it into
/// `T`.
/// 
/// # Panics
/// 
/// Will panic if the key doesn't exist, unless some `default` is provided;
/// and if the key exists, but can't be parsed into `T`.
fn from_env<T>(key: &'static str, default: Option<T>) -> T
where
    T: FromStr,
{
    match env::var(key) {
        Ok(str_value) => {
            if let Ok(val) = str_value.parse::<T>() {
                val
            } else {
                panic!(
                    "from_env: key {} exists, but value couldn't be parsed!",
                    key
                );
            }
        }
        Err(_) => {
            if let Some(val) = default {
                val
            } else {
                panic!(
                    "from_env: key {} wasn't set and no default was supplied!",
                    key
                );
            }
        }
    }
}

/// The osu! api key to be used on requests. Can be either on
/// `/run/secrets/osu_pp_calc_api_key`, if running under Docker, or on the
/// `OSU_PP_CALC_API_KEY` env variable.
/// 
/// # Panics
/// 
/// Will panic if neither `/run/secrets/osu_pp_calc_api_key` nor the
/// `OSU_PP_CALC_API_KEY` exists.
pub fn api_key() -> String {
    let docker_secret_file = Path::new("/run/secrets/osu_pp_calc_api_key");
    if docker_secret_file.exists() {
        fs::read_to_string(docker_secret_file)
            .unwrap()
            .trim()
            .to_string()
    } else {
        from_env("OSU_PP_CALC_API_KEY", None)
    }
}

/// The path to the dotnet executable. Is read from the `OSU_PP_CALC_DOTNET_COMMAND`
/// env variable, and defaults to "dotnet".
pub fn dotnet_command() -> String {
    from_env("OSU_PP_CALC_DOTNET_COMMAND", Some("dotnet".to_string()))
}

/// The number of workers to be used on the profile calculation queue. Is read
/// from the `OSU_PP_CALC_NUM_THREADS` env variable, and defaults to 2.
pub fn num_threads() -> usize {
    from_env("OSU_PP_CALC_NUM_THREADS", Some(2))
}

/// Whether to save the profile results cache into a file. Is read
/// from the `OSU_PP_CALC_LOAD_SAVE_RESULTS` env variable, and defaults to false.
pub fn load_save_results() -> bool {
    from_env("OSU_PP_CALC_LOAD_SAVE_RESULTS", Some(false))
}

/// The file where to save the profile results cache, if `load_save_results()` is true.
/// Is read from the `OSU_PP_CALC_RESULTS_FILE` env variable, and defaults to "results.data".
pub fn results_file() -> String {
    from_env("OSU_PP_CALC_RESULTS_FILE", Some("results.data".to_string()))
}

/// The file where to store the beatmaps cache.
/// Is read from the `OSU_PP_CALC_BEATMAPS_CACHE` env variable, and defaults to "cache".
pub fn beatmaps_cache() -> String {
    from_env("OSU_PP_CALC_BEATMAPS_CACHE", Some("cache".to_string()))
}

/// The minimal "age" for a profile calculation result to be, so that it's allowed to be
/// forcibly recalculated, in seconds.
/// Is read from the `OSU_PP_CALC_FORCE_INTERVAL_SECS` env variable, and defaults to 15 minutes.
pub fn minimal_force_interval() -> Duration {
    Duration::from_secs(from_env("OSU_PP_CALC_FORCE_INTERVAL_SECS", Some(60 * 15)))
}

/// The directory where the currently running executable resides.
/// 
/// # Panics
/// 
/// Will panic if `env::current_exe()` errors.
fn binary_dir() -> PathBuf {
    match env::current_exe() {
        Ok(mut exe_path) => {
            exe_path.pop();
            // "state-of-the-art" Rust code
            // see https://github.com/rust-lang/cargo/issues/5758
            if exe_path.ends_with("deps") {
                exe_path.pop();
            }
            exe_path
        }
        Err(e) => panic!(format!("Couldn't get current path! {}", e)),
    }
}

/// The path to the osu-tools' PerformanceCalculator.dll. 
/// It should be placed on the same dir as the application binary.
pub fn performance_calculator_path() -> String {
    let mut dir = binary_dir();
    dir.push("PerformanceCalculator.dll");
    dir.to_str().unwrap().to_string()
}
