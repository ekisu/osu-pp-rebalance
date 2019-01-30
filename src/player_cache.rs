use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::thread::{JoinHandle};
use super::performance_calculator::{calculate_performance, PerformanceResults};

#[derive(Clone, Copy, PartialEq)]
pub enum CalcStatus {
    Pending(u64),
    Calculating,
    Done,
    Error
}

pub struct PlayerCache {
    calc_status: Arc<Mutex<HashMap<String, CalcStatus>>>,
    data: Arc<Mutex<HashMap<String, PerformanceResults>>>,
    tx_request: Arc<Mutex<Sender<String>>>,
    worker_handle: JoinHandle<()>,
    last_queue: Arc<Mutex<u64>>,
    current_queue: Arc<Mutex<u64>>
}

impl PlayerCache {
    pub fn new() -> Self {
        let (tx_request, rx_req) = channel();
        let calc_status = Arc::new(Mutex::new(HashMap::new()));
        let calc_clone = calc_status.clone();
        let data = Arc::new(Mutex::new(HashMap::new()));
        let data_clone = data.clone();
        let last_queue = Arc::new(Mutex::new(0u64));
        let current_queue = Arc::new(Mutex::new(0u64));
        let current_clone = current_queue.clone();

        let worker_handle = thread::spawn(move || {
            let _calc = calc_status.clone();
            let _data = data.clone();
            let _current = current_queue.clone();

            loop {
                let player_request : String = rx_req.recv().unwrap();
                {
                    let mut _guard = _calc.lock().unwrap();
                    if let CalcStatus::Pending(current) = _guard[&player_request] {
                        *_current.lock().unwrap() = current; // uhh this should always happen, but idk
                    }
                    _guard.insert(player_request.clone(), CalcStatus::Calculating);
                }

                let result = calculate_performance(player_request.clone());
                if let Ok(perf) = result {
                    _calc.lock().unwrap().insert(player_request.clone(), CalcStatus::Done);
                    _data.lock().unwrap().insert(player_request.clone(), perf);
                } else {
                    _calc.lock().unwrap().insert(player_request.clone(), CalcStatus::Error);
                }
            }
        });

        PlayerCache {
            worker_handle, data: data_clone, calc_status: calc_clone,
            tx_request: Arc::new(Mutex::new(tx_request)),
            current_queue: current_clone,
            last_queue: last_queue
        }
    }

    // if already calculated, Some(results), otherwise None and it's enqueued.
    pub fn calculate_request(&self, player: String) -> Option<PerformanceResults> {
        let _guard = self.data.lock().unwrap();

        if _guard.contains_key(&player) {
            Some(_guard[&player].clone())
        } else {
            let mut _guard_status = self.calc_status.lock().unwrap();
            if !_guard_status.contains_key(&player) || _guard_status[&player] == CalcStatus::Error {
                let mut _last = self.last_queue.lock().unwrap();
                *_last += 1;
                _guard_status.insert(player.clone(), CalcStatus::Pending(*_last));
                self.tx_request.lock().unwrap().send(player.clone());
            }
            None
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

    pub fn get_current_in_queue(&self) -> u64 {
        *self.current_queue.lock().unwrap()
    }

    pub fn get_performance(&self, player: String) -> Option<PerformanceResults> {
        let _guard = self.data.lock().unwrap();

        if _guard.contains_key(&player) {
            Some(_guard[&player].clone())
        } else {
            None
        }
    }
}
