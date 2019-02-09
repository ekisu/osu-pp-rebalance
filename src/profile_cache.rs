use super::performance_calculator::ProfileResults;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct ProfileCache {
    data: Arc<Mutex<HashMap<String, (ProfileResults, SystemTime)>>>,
}

impl ProfileCache {
    fn load_results(
        results_file: String,
    ) -> Result<HashMap<String, (ProfileResults, SystemTime)>, Box<Error>> {
        let file = File::open(results_file)?;
        let reader = BufReader::new(file);

        let results = serde_json::from_reader(reader)?;

        Ok(results)
    }

    fn save_results(
        data: &HashMap<String, (ProfileResults, SystemTime)>,
        results_file: String,
    ) -> Result<(), Box<Error>> {
        let file = File::create(results_file)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, data)?;

        Ok(())
    }

    // what if the save location wasn't static
    pub fn setup_save_results_handler(&self, save_location: String) {
        let data_clone = self.data.clone();
        ctrlc::set_handler(move || {
            let guard = data_clone.lock().unwrap();
            let location_clone = save_location.clone(); // uhh

            println!("Saving results data...");

            match ProfileCache::save_results(&*guard, location_clone) {
                Ok(_) => println!("Saved results data successfully!"),
                Err(e) => println!("Error while saving: {}", e),
            };
        });
    }

    pub fn new(results_file: Option<String>) -> Self {
        let data_hm = if let Some(_file_path) = results_file {
            // FIXME fails silently
            match ProfileCache::load_results(_file_path) {
                Ok(hm) => hm,
                Err(err) => {
                    println!("Error while loading results.data: {}", err);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        ProfileCache {
            data: Arc::new(Mutex::new(data_hm)),
        }
    }

    pub fn get(&self, player: String) -> Option<(ProfileResults, SystemTime)> {
        let _guard = self.data.lock().unwrap();

        if _guard.contains_key(&player) {
            let pair = &_guard[&player];
            Some(pair.clone())
        } else {
            None
        }
    }

    pub fn set(&self, player: String, result: ProfileResults) {
        let mut _guard = self.data.lock().unwrap();

        _guard.insert(player.clone(), (result, SystemTime::now()));
    }
}
