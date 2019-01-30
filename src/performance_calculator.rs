use crate::config::{OSU_API_KEY, PERFORMANCE_CALCULATOR_PATH, DOTNET_COMMAND};
use std::process::{Command, Stdio};
use std::path::Path;

pub fn calculate_performance(user: String) -> Result<String, String> {
    println!("{}", PERFORMANCE_CALCULATOR_PATH);
    let output = Command::new(DOTNET_COMMAND)
                           .arg(PERFORMANCE_CALCULATOR_PATH)
                           .arg("profile")
                           .arg(user)
                           .arg(OSU_API_KEY)
                           //.current_dir(Path::new(PERFORMANCE_CALCULATOR_PATH).parent().unwrap())
                           .output();
    
    match output {
        Ok(res) => {
            println!("status: {}", res.status);
            if res.status.success() {
                Ok(String::from_utf8_lossy(&res.stdout).to_string())
            } else {
                println!("status: {}", String::from_utf8_lossy(&res.stderr));
                Err(String::from("failed to calculate performance"))
            }
        },
        Err(_) => Err(String::from("failed to calculate performance"))
    }
}
