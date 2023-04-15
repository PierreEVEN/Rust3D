use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::job_pool::{JobData, JobPool};
use crate::JobPtr;

pub struct WorkerSharedData {
    job_pool: Vec<Mutex<JobPool>>,
}

impl WorkerSharedData {
    pub fn new(num_worker_threads: usize) -> Self {
        let mut job_pool = Vec::with_capacity(num_worker_threads);
        for _ in 0..num_worker_threads {
            job_pool.push(Mutex::new(JobPool::new()))
        }

        Self { job_pool }
    }

    pub fn next_job(&self, worker_index: usize) -> Option<JobData> {
        match self.job_pool[worker_index].lock() {
            Ok(mut pool) => {
                if !pool.is_empty() {
                    return (*pool).pop();
                }
            }
            Err(_) => { panic!("failed to lock pool") }
        }
        self.steal_job()
    }

    pub fn steal_job(&self) -> Option<JobData> {
        for pool in &self.job_pool {
            match pool.lock() {
                Ok(mut pool) => {
                    return (*pool).pop();
                }
                Err(_) => { panic!("failed to lock pool"); }
            };
        }
        None
    }

    pub fn push_job(&self, worker_index: usize, job: JobData) -> JobPtr {
        match self.job_pool[worker_index].lock() {
            Ok(mut pool) => { (*pool).push(job) }
            Err(_) => { panic!("failed to lock pool") }
        }
    }
}

pub struct Worker {
    worker_id: usize,
    _running_thread: JoinHandle<()>,
    shared_data: Arc<WorkerSharedData>,
}

impl Worker {
    pub fn new(worker_id: usize, shared_data: Arc<WorkerSharedData>) -> Self {
        let data_copy = shared_data.clone();
        let running_thread = thread::spawn(move || {
            loop {
                match data_copy.next_job(worker_id) {
                    None => {}
                    Some(mut job) => { job.execute() }
                };
            }
        });

        Self {
            worker_id,
            _running_thread: running_thread,
            shared_data,
        }
    }

    pub fn id(&self) -> usize {
        self.worker_id
    }

    pub fn push_job(&mut self, job: JobData) -> JobPtr {
        self.shared_data.push_job(self.worker_id, job)
    }
}
