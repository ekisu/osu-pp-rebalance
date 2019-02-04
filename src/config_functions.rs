use std::env;
use std::path::PathBuf;
use crate::config::OSU_API_KEY;

pub fn api_key() -> String {
    match env::var("OSU_API_KEY") {
        Ok(val) => val,
        Err(_) => OSU_API_KEY.to_string()
    }
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
