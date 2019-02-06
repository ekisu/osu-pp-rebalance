use std::collections::HashMap;
use std::error::Error;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::thread::{JoinHandle};
use std::time::{Duration, Instant, SystemTime};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use super::performance_calculator::{calculate_profile, ProfileResults};

#[derive(Clone, Copy, PartialEq)]
pub enum CalcStatus {
    Pending(u64, Instant),
    Calculating,
    Done,
    Error
}

#[derive(Clone, Copy, PartialEq)]
pub enum EnqueueResult {
    AlreadyDone,
    Enqueued,
    CantForce(Duration)
}

pub struct Worker {
    handle: JoinHandle<()>
}

impl Worker {
    pub fn new(calc_status: Arc<Mutex<HashMap<String, CalcStatus>>>,
                data: Arc<Mutex<HashMap<String, (ProfileResults, SystemTime)>>>,
                rx_request: Arc<Mutex<Receiver<String>>>,
                current_queue: Arc<Mutex<u64>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let player_request : String = rx_request.lock().unwrap().recv().unwrap();
                {
                    let mut _guard = calc_status.lock().unwrap();
                    if let CalcStatus::Pending(current, last_ping) = _guard[&player_request] {
                        let ten_s = Duration::from_secs(15);
                        if last_ping.elapsed() > ten_s {
                            _guard.insert(player_request.clone(), CalcStatus::Error);
                            continue;
                        }

                        *current_queue.lock().unwrap() = current; // uhh this should always happen, but idk
                    }
                    _guard.insert(player_request.clone(), CalcStatus::Calculating);
                }

                let result = calculate_profile(player_request.clone());
                match result {
                    Ok(perf) => {
                        calc_status.lock().unwrap().insert(player_request.clone(), CalcStatus::Done);
                        data.lock().unwrap().insert(player_request.clone(), (perf, SystemTime::now()));
                    },
                    Err(_) => {
                        calc_status.lock().unwrap().insert(player_request.clone(), CalcStatus::Error);
                    }
                }
            }
        });

        Worker {
            handle: thread
        }
    }
}

pub struct PlayerCache {
    calc_status: Arc<Mutex<HashMap<String, CalcStatus>>>,
    data: Arc<Mutex<HashMap<String, (ProfileResults, SystemTime)>>>,
    tx_request: Arc<Mutex<Sender<String>>>,
    worker_handles: Vec<Worker>,
    last_queue: Arc<Mutex<u64>>,
    current_queue: Arc<Mutex<u64>>
}

impl PlayerCache {
    fn load_results(results_file: String) -> 
        Result<HashMap<String, (ProfileResults, SystemTime)>, Box<Error>> {
        let file = File::open(results_file)?;
        let reader = BufReader::new(file);

        let results = serde_json::from_reader(reader)?;

        Ok(results)
    }

    fn save_results(data: &HashMap<String, (ProfileResults, SystemTime)>,
                    results_file: String) -> Result<(), Box<Error>> {
            
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

            match PlayerCache::save_results(&*guard, location_clone) {
                Ok(_) => println!("Saved results data successfully!"),
                Err(e) => println!("Error while saving: {}", e)
            };
        });
    }

    pub fn new(workers: usize, results_file: Option<String>) -> Self {
        let (tx_request, rx_req) = channel();
        let calc_status = Arc::new(Mutex::new(HashMap::new()));
        let calc_clone = calc_status.clone();

        let data_hm = if let Some(_file_path) = results_file {
            // FIXME fails silently
            match PlayerCache::load_results(_file_path) {
                Ok(hm) => hm,
                Err(err) => {
                    println!("Error while loading results.data: {}", err);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        let data = Arc::new(Mutex::new(data_hm));
        let data_clone = data.clone();
        let last_queue = Arc::new(Mutex::new(0u64));
        let current_queue = Arc::new(Mutex::new(0u64));
        let current_clone = current_queue.clone();

        let rx_req_arc = Arc::new(Mutex::new(rx_req));
        let worker_handles = (0..workers).map(|_| Worker::new(calc_status.clone(), data.clone(),
                                                              rx_req_arc.clone(), current_queue.clone())).collect();

        PlayerCache {
            worker_handles, data: data_clone, calc_status: calc_clone,
            tx_request: Arc::new(Mutex::new(tx_request)),
            current_queue: current_clone,
            last_queue: last_queue
        }
    }

    pub fn calculate_request(&self, player: String, force: bool) -> EnqueueResult {
        let _guard = self.data.lock().unwrap();

        if _guard.contains_key(&player) {
            if !force {
                EnqueueResult::AlreadyDone
            } else {
                let (_, last_updated) = _guard[&player];
                let cooldown_time = Duration::from_secs(15 * 60);
                let elapsed_result = last_updated.elapsed();
                let mut elapsed = Duration::from_secs(0); // bruh #1

                // Uhh what if the time goes backwards (or whatever causes elapsed to err)?
                // Should we allow it or not?
                if elapsed_result.is_ok()
                   // bruh #2
                   && { elapsed = elapsed_result.unwrap(); elapsed } < cooldown_time {
                    EnqueueResult::CantForce(cooldown_time - elapsed)
                } else {
                    // copypaste ;w;
                    let mut _guard_status = self.calc_status.lock().unwrap();
                    if !_guard_status.contains_key(&player) 
                       || _guard_status[&player] == CalcStatus::Error
                       // Now we consider Done because we're forcing. Actually, we just don't want to overwrite
                       // Pending or Calculating.
                       || _guard_status[&player] == CalcStatus::Done {
                        let mut _last = self.last_queue.lock().unwrap();
                        *_last += 1;
                        _guard_status.insert(player.clone(), CalcStatus::Pending(*_last, Instant::now()));
                        self.tx_request.lock().unwrap().send(player.clone());
                    }

                    EnqueueResult::Enqueued
                }
            }
        } else {
            let mut _guard_status = self.calc_status.lock().unwrap();
            if !_guard_status.contains_key(&player) || _guard_status[&player] == CalcStatus::Error {
                let mut _last = self.last_queue.lock().unwrap();
                *_last += 1;
                _guard_status.insert(player.clone(), CalcStatus::Pending(*_last, Instant::now()));
                self.tx_request.lock().unwrap().send(player.clone());
            }
            EnqueueResult::Enqueued
        }
    }

    pub fn check_status(&self, player: String) -> Option<CalcStatus> {
        let _guard = self.calc_status.lock().unwrap();

        if _guard.contains_key(&player) {
            Some(_guard[&player])
        } else {
            None
        }
    }

    pub fn ping(&self, player: String) {
        let mut _guard = self.calc_status.lock().unwrap();

        if _guard.contains_key(&player) {
            if let CalcStatus::Pending(pos, _) = _guard[&player] {
                _guard.insert(player.clone(), CalcStatus::Pending(pos, Instant::now()));
            }
        }
    }

    pub fn get_current_in_queue(&self) -> u64 {
        *self.current_queue.lock().unwrap()
    }

    pub fn get_performance(&self, player: String) -> Option<ProfileResults> {
        let _guard = self.data.lock().unwrap();

        if _guard.contains_key(&player) {
            let (perf, _) = &_guard[&player];
            Some(perf.clone())
        } else {
            None
        }
    }
}
