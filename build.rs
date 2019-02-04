use std::process::Command;
use std::env;
use std::path::PathBuf;

fn target_dir() -> PathBuf {
    let profile = env::var("PROFILE").unwrap();
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target).join(profile)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target").join(profile)
    }
}

fn main() {
    let target_dir = target_dir();

    Command::new("dotnet").args(&["build",
                        "osu-tools/PerformanceCalculator/PerformanceCalculator.csproj", "-o"])
                       .arg(target_dir.to_str().unwrap())
                       .status().unwrap();
}