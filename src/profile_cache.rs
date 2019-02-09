//! A thread-safe cache implementation for `ProfileResults`.
//!
//! Alongside the calculation results, this structure also stores
//! the time when the calculation happened (more accurately, when it
//! was placed into the cache). This information is used to determine
//! whether this calculation is "too fresh", and as so to avoid user
//! abuse.

use super::performance_calculator::ProfileResults;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// A cache for the profile calculation results.
pub struct ProfileCache {
    data: Arc<Mutex<HashMap<String, (ProfileResults, SystemTime)>>>,
}

impl ProfileCache {
    /// Load results stored in `results_file` into a HashMap.
    ///
    /// # Errors
    ///
    /// Will error if the `results_file` couldn't be opened, read,
    /// or if its contents aren't a valid cache representation.
    fn load_results(
        results_file: String,
    ) -> Result<HashMap<String, (ProfileResults, SystemTime)>, Box<Error>> {
        let file = File::open(results_file)?;
        let reader = BufReader::new(file);

        let results = serde_json::from_reader(reader)?;

        Ok(results)
    }

    /// Save the results stored in `data` into `results_file`.
    ///
    /// # Errors
    ///
    /// Will error if the HashMap fails to be written to `results_file`.
    fn save_results(
        data: &HashMap<String, (ProfileResults, SystemTime)>,
        results_file: String,
    ) -> Result<(), Box<Error>> {
        let file = File::create(results_file)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, data)?;

        Ok(())
    }

    /// Setup a signal handler, to save the current profile cache into
    /// `save_location` on program termination.
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

    /// Create a new `ProfileCache`.
    ///
    /// If `results_file` is `Some(file)`, the cache will be loaded with
    /// the contents stored in `file`.
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

    /// Gets a `ProfileResults`, and the time it was calculated,
    /// associated with said `player`.
    ///
    /// If no result is found, returns None.
    pub fn get(&self, player: String) -> Option<(ProfileResults, SystemTime)> {
        let _guard = self.data.lock().unwrap();

        if _guard.contains_key(&player) {
            let pair = &_guard[&player];
            Some(pair.clone())
        } else {
            None
        }
    }

    /// Associates the `result` with this `player`. Also stores
    /// the time this was set.
    pub fn set(&self, player: String, result: ProfileResults) {
        let mut _guard = self.data.lock().unwrap();

        _guard.insert(player.clone(), (result, SystemTime::now()));
    }
}
