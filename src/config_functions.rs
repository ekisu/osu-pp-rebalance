use std::env;
use std::path::PathBuf;
use std::str::FromStr;

fn from_env<T>(key: &'static str, default: Option<T>) -> T 
where
    T: FromStr {
    match env::var(key) {
        Ok(str_value) => if let Ok(val) = str_value.parse::<T>() {
            val
        } else {
            panic!("from_env: key {} exists, but value couldn't be parsed!", key);
        },
        Err(_) => if let Some(val) = default {
            val
        } else {
            panic!("from_env: key {} wasn't set and no default was supplied!", key);
        }
    }
}

pub fn api_key() -> String {
    from_env("OSU_PP_CALC_API_KEY", None)
}

pub fn dotnet_command() -> String {
    from_env("OSU_PP_DOTNET_COMMAND", Some("dotnet".to_string()))
}

pub fn num_threads() -> usize {
    from_env("OSU_PP_CALC_NUM_THREADS", Some(2))
}

pub fn load_save_results() -> bool {
    from_env("OSU_PP_LOAD_SAVE_RESULTS", Some(false))
}

pub fn results_file() -> String {
    from_env("OSU_PP_RESULTS_FILE", Some("results.data".to_string()))
}

pub fn beatmaps_cache() -> String {
    from_env("OSU_PP_BEATMAPS_CACHE", Some("cache".to_string()))
}

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
        },
        Err(e) => panic!(format!("Couldn't get current path! {}", e))
    }
}

pub fn performance_calculator_path() -> String {
    let mut dir = binary_dir();
    dir.push("PerformanceCalculator.dll");
    dir.to_str().unwrap().to_string()
}
