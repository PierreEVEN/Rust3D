use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use crate::job_pool::{JobData, JobPool};
use crate::{JobPtr};

pub struct WorkerSharedData {
    job_pool: Vec<Mutex<JobPool>>,
    sleep_condition: Condvar,
    sleep_mutex: Mutex<()>,
    stop: Mutex<bool>,
}

impl WorkerSharedData {
    pub fn new(num_worker_threads: usize) -> Self {
        let mut job_pool = Vec::with_capacity(num_worker_threads);
        for _ in 0..num_worker_threads {
            job_pool.push(Mutex::new(JobPool::new()))
        }

        Self {
            job_pool,
            sleep_condition: Default::default(),
            sleep_mutex: Mutex::new(()),
            stop: Mutex::new(false),
        }
    }

    pub fn next_job(&self, worker_index: usize) -> Option<JobData> {
        match self.job_pool[worker_index].lock() {
            Ok(mut pool) => {
                if !pool.is_empty() {
                    return (*pool).pop();
                }
            }
            Err(_) => { logger::fatal!("failed to lock pool") }
        }
        self.steal_job()
    }

    pub fn steal_job(&self) -> Option<JobData> {
        for pool in &self.job_pool {
            match pool.lock() {
                Ok(mut pool) => {
                    match (*pool).pop() {
                        None => {}
                        Some(job) => { return Some(job); }
                    };
                }
                Err(_) => { logger::fatal!("failed to lock pool"); }
            };
        }
        None
    }

    pub fn push_job(&self, worker_index: usize, job: JobData) -> JobPtr {
        match self.job_pool[worker_index].lock() {
            Ok(mut pool) => { (*pool).push(job) }
            Err(_) => { logger::fatal!("failed to lock pool") }
        }
    }

    pub fn stop(&self) {
        *self.stop.lock().expect("failed to lock") = true;
        self.sleep_condition.notify_all();
    }

    pub fn contains_jobs(&self) -> bool {
        for pool in &self.job_pool {
            if !pool.lock().expect("lock failed").is_empty() {
                return true;
            }
        }
        false
    }
}

pub struct Worker {
    worker_id: usize,
    running_thread: JoinHandle<()>,
    shared_data: Arc<WorkerSharedData>,
}

impl Worker {
    pub fn new(job_system_name: &str, worker_id: usize, shared_data: Arc<WorkerSharedData>) -> Self {
        let data_copy = shared_data.clone();
        let js_name = job_system_name.to_string();
        let running_thread = thread::spawn(move || {
            logger::set_thread_label(thread::current().id(), format!("{js_name}::{worker_id}").as_str());
            
            loop {
                let mut stop = false;
                while !stop {
                    match data_copy.next_job(worker_id) {
                        None => {
                            match data_copy.sleep_condition.wait(data_copy.sleep_mutex.lock().expect("failed to lock")) {
                                Ok(_) => {
                                    if *data_copy.stop.lock().expect("lock failed") {
                                        stop = true;
                                    }
                                }
                                Err(_) => { logger::fatal!("Wait for new task failed somewhere") }
                            };
                        }
                        Some(mut job) => { job.execute() }
                    };
                }
                if !data_copy.contains_jobs() { break; }
            }
        });
        
        Self {
            worker_id,
            running_thread,
            shared_data,
        }
    }

    pub fn id(&self) -> usize {
        self.worker_id
    }

    pub fn push_job(&mut self, job: JobData) -> JobPtr {
        let job = self.shared_data.push_job(self.worker_id, job);
        self.shared_data.sleep_condition.notify_one();
        job
    }

    pub fn join(&self) {
        while !self.running_thread.is_finished() {
            self.shared_data.sleep_condition.notify_one();
            thread::sleep(Duration::from_millis(1));
        }
    }
}
