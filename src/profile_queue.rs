extern crate mt_job_queue;

use mt_job_queue::Queue;
use mt_job_queue::queue::JobState;
use std::sync::{Arc, Mutex};
use super::profile_cache::ProfileCache;
use super::performance_calculator::calculate_profile;
use super::performance_calculator::ProfileResults;

use std::collections::{BTreeSet, HashMap};

pub struct ProfileQueue {
    calculation_errors: BTreeSet<String>,
    user_job_id: Arc<Mutex<HashMap<String, usize>>>,
    job_queue: Queue<String>,
    profile_cache: Arc<ProfileCache>
}

#[derive(Clone, Copy, PartialEq)]
pub enum RequestStatus {
    Pending(usize),
    Calculating,
    Done,
    Error
}

impl ProfileQueue {
    pub fn new(profile_cache: Arc<ProfileCache>, num_threads: usize) -> Self {
        let calculation_errors = BTreeSet::new();
        let process_job = Arc::new(|user: String| {
            let opt = match calculate_profile(user.clone()) {
                Ok(result) => Some(result),
                Err(_) => None
            };

            (user, opt)
        });

        let job_completed_profile_cache = profile_cache.clone();
        let on_job_completed = Arc::new(move |(user, result): (String, Option<ProfileResults>)| {
            match result {
                Some(profile_results) => job_completed_profile_cache.set(user, profile_results),
                None => {
                    //calculation_errors.insert(user);
                }
            }
        });

        ProfileQueue {
            calculation_errors: calculation_errors,
            job_queue: Queue::new(num_threads, process_job, on_job_completed),
            profile_cache: profile_cache,
            user_job_id: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn enqueue(&self, user: String) {
        let mut _guard = self.user_job_id.lock().unwrap();

        if _guard.contains_key(&user) {
            return;
        } 

        let job_id = self.job_queue.enqueue(user.clone());

        _guard.insert(user, job_id);
    }

    pub fn status(&self, user: String) -> Option<RequestStatus> {
        match self.user_job_id.lock().unwrap().get(&user) {
            Some(job_id) => Some(match self.job_queue.job_state(*job_id) {
                JobState::Pending => RequestStatus::Pending(self.job_queue.position(*job_id)),
                JobState::Acknowledged => RequestStatus::Calculating,
                JobState::Complete => {
                    if self.calculation_errors.contains(&user) {
                        RequestStatus::Error
                    } else {
                        RequestStatus::Done
                    }
                }
            }),
            None => None
        }
    }
}
