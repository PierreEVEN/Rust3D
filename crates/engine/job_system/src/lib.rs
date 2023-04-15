pub mod worker;
pub mod job_pool;

extern crate num_cpus;

use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use crate::job_pool::{JobData};
use crate::worker::{Worker, WorkerSharedData};

#[derive(Copy, Clone)]
pub struct JobPtr {
    job_ptr: *mut JobData,
}

impl JobPtr {
    pub fn null() -> Self {
        Self { job_ptr: null_mut() }
    }
    pub fn new(pool: &mut JobData) -> Self {
        Self {
            job_ptr: pool as *const JobData as *mut JobData,
        }
    }
    
    pub fn read(&self) -> &JobData {
        if self.job_ptr.is_null() { panic!("Job pointer is null") }
        unsafe { &*self.job_ptr }
    }
}

pub struct JobSystem {
    workers: Vec<Worker>,
    current_worker: usize,
}

impl JobSystem {
    pub fn new(num_worker_threads: usize) -> Self {
        let mut workers = Vec::with_capacity(num_worker_threads);

        let job_pool = Arc::new(WorkerSharedData::new(num_worker_threads));

        for i in 0..num_worker_threads {
            workers.push(Worker::new(i, job_pool.clone()));
        }

        Self {
            workers,
            current_worker: 0,
        }
    }

    pub fn available_cpu_threads() -> usize {
        num_cpus::get()
    }

    fn next_worker(&mut self) -> &mut Worker {
        self.current_worker = (self.current_worker + 1) % self.workers.len();
        &mut self.workers[self.current_worker]
    }

    pub fn schedule<Fn: 'static + FnOnce()>(&mut self, job_fn: Fn) -> JobPtr {
        // Pick a random worker
        let worker = self.next_worker();

        // then queue new job
        worker.push_job(JobData::new(job_fn))
    }

    pub fn wait_idle(&self, _handle: JobPtr) {
        todo!("wait on job semaphore")
    }
}


pub fn test_func() {
    let mut js = JobSystem::new(JobSystem::available_cpu_threads());

    let counter = Arc::new(Mutex::new(0));
    
    for i in 0..10 {
        let cnt = counter.clone();
        let _handle = js.schedule(move || {
            
            println!("Task {i} Start");
            for _ in 0..10000000 {
                *cnt.as_ref().lock().expect("ok") += 1;
            }
            println!("Task {i} done : {}", cnt.as_ref().lock().expect("ok"));
        });
    }

    println!("registration done...");

    println!("result : {}", counter.as_ref().lock().expect("ok"))
}

#[cfg(test)]
mod test {
    use crate::{test_func};

    #[test]
    fn test() {
        test_func()
    }
}