use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn target_dir() -> PathBuf {
    let profile = env::var("PROFILE").unwrap();
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target).join(profile)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("target")
            .join(profile)
    }
}

fn disable_build() -> bool {
    match env::var("DONT_BUILD_PERFORMANCE_CALCULATOR") {
        Ok(val) => val == "1",
        Err(_) => false,
    }
}

fn is_debug() -> bool {
    env::var("PROFILE").unwrap() == "debug"
}

fn main() {
    if disable_build() {
        return;
    }

    let target_dir = target_dir();

    println!("Building PerformanceCalculator.dll");

    Command::new("dotnet")
        .args(&[
            "publish",
            "osu-tools/PerformanceCalculator/PerformanceCalculator.csproj",
            "-c",
        ])
        .arg(if is_debug() { "Debug" } else { "Release" })
        .arg("-o")
        .arg(fs::canonicalize(target_dir.clone()).unwrap().to_str().unwrap())
        // We don't know the system we're publishing, so linking won't work
        .arg("/p:LinkDuringPublish=false")
        .status()
        .unwrap();

    println!("Removing unneeded build files from PerformanceCalculator.dll");
    
    Command::new("dotnet")
        .args(&[
            "run",
            "--project",
            "osu-tools/RemoveBuildFiles/RemoveBuildFiles.csproj",
        ])
        .arg(target_dir)
        .status()
        .unwrap();
}
