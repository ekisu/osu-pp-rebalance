//! A multi-threaded job queue for profile PP calculations.
//!
//! Relies on `mt_job_queue` internally, but stores the information
//! needed for the polling-based web interface to work.
//!
//! A single user name/ID can be requested multiple times, however,
//! while in the queue, they will always be associated with a single job.
//! This avoid unnecessary computations.
extern crate mt_job_queue;

use super::performance_calculator::calculate_profile;
use super::performance_calculator::ProfileResults;
use super::profile_cache::ProfileCache;
use mt_job_queue::queue::JobState;
use mt_job_queue::Queue;
use std::sync::{Arc, Mutex};

use std::collections::{BTreeSet, HashMap};

/// The ProfileQueue struct.
pub struct ProfileQueue {
    calculation_errors: Arc<Mutex<BTreeSet<String>>>,
    user_job_id: Arc<Mutex<HashMap<String, usize>>>,
    job_queue: Queue<String>,
    profile_cache: Arc<ProfileCache>,
}

/// A enum, that represents the status of a request. When it's `Pending`,
/// the associated `usize` is the position of this request on the queue
/// (i.e. how many people are ahead of you.)
#[derive(Clone, Copy, PartialEq)]
pub enum RequestStatus {
    Pending(usize),
    Calculating,
    Done,
    Error,
}

impl ProfileQueue {
    /// Creates a new `ProfileQueue`, with `num_threads` workers.
    ///
    /// The results will be stored into `profile_cache`.
    pub fn new(profile_cache: Arc<ProfileCache>, num_threads: usize) -> Self {
        let calculation_errors = Arc::new(Mutex::new(BTreeSet::new()));
        let process_job = Arc::new(|user: String| {
            let opt = match calculate_profile(user.clone()) {
                Ok(result) => Some(result),
                Err(_) => None,
            };

            (user, opt)
        });

        let job_completed_profile_cache = profile_cache.clone();
        let job_completed_calculation_errors = calculation_errors.clone();
        let on_job_completed =
            Arc::new(
                move |(user, result): (String, Option<ProfileResults>)| match result {
                    Some(profile_results) => job_completed_profile_cache.set(user, profile_results),
                    None => {
                        job_completed_calculation_errors
                            .lock()
                            .unwrap()
                            .insert(user);
                    }
                },
            );

        ProfileQueue {
            calculation_errors: calculation_errors,
            job_queue: Queue::new(num_threads, process_job, on_job_completed),
            profile_cache: profile_cache,
            user_job_id: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Places a new `user` into the calculation queue. If the user already is
    /// on the queue, nothing happens.
    pub fn enqueue(&self, user: String) {
        let mut _guard = self.user_job_id.lock().unwrap();

        if _guard.contains_key(&user) {
            return;
        }

        let job_id = self.job_queue.enqueue(user.clone());

        _guard.insert(user, job_id);
    }

    /// Obtains the status of a calculation request for a `user`.
    pub fn status(&self, user: String) -> Option<RequestStatus> {
        match self.user_job_id.lock().unwrap().get(&user) {
            Some(job_id) => Some(match self.job_queue.job_state(*job_id) {
                JobState::Pending => RequestStatus::Pending(self.job_queue.position(*job_id)),
                JobState::Acknowledged => RequestStatus::Calculating,
                JobState::Complete => {
                    if self.calculation_errors.lock().unwrap().contains(&user) {
                        RequestStatus::Error
                    } else {
                        RequestStatus::Done
                    }
                }
            }),
            None => None,
        }
    }
}
