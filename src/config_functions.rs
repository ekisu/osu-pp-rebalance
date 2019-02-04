use std::env;
use crate::config::OSU_API_KEY;

pub fn api_key() -> String {
    match env::var("OSU_API_KEY") {
        Ok(val) => val,
        Err(_) => OSU_API_KEY.to_string()
    }
}
