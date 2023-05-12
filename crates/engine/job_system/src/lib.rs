pub mod job_pool;
pub mod worker;

extern crate num_cpus;

use crate::job_pool::JobData;
use crate::worker::{Worker, WorkerSharedData};
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone)]
pub struct JobPtr {
    job_ptr: *mut JobData,
}

impl JobPtr {
    pub fn null() -> Self {
        Self {
            job_ptr: null_mut(),
        }
    }
    pub fn new(pool: &mut JobData) -> Self {
        Self {
            job_ptr: pool as *const JobData as *mut JobData,
        }
    }

    pub fn read(&self) -> &JobData {
        if self.job_ptr.is_null() {
            logger::fatal!("Job pointer is null")
        }
        unsafe { &*self.job_ptr }
    }
}

pub struct JobSystem {
    workers: Vec<Worker>,
    current_worker: usize,
    shared_data: Arc<WorkerSharedData>,
}

impl JobSystem {
    pub fn new(debug_name: &str, num_worker_threads: usize) -> Self {
        logger::info!("create job system '{debug_name}' with {num_worker_threads} worker threads");
        let mut workers = Vec::with_capacity(num_worker_threads);

        let shared_data = Arc::new(WorkerSharedData::new(num_worker_threads));

        for i in 0..num_worker_threads {
            workers.push(Worker::new(debug_name, i, shared_data.clone()));
        }

        Self {
            workers,
            current_worker: 0,
            shared_data,
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

impl Drop for JobSystem {
    fn drop(&mut self) {
        self.shared_data.stop();
        for worker in &self.workers {
            worker.join();
        }
    }
}

pub fn test_func() -> JobSystem {
    let mut js = JobSystem::new("JS_Global", JobSystem::available_cpu_threads());

    let counter = Arc::new(Mutex::new(0));

    let n_tasks = 50;
    for i in 1..n_tasks + 1 {
        logger::info!("spawn heavy task #{i}/{n_tasks}");
        let cnt = counter.clone();
        let _handle = js.schedule(move || {
            for _ in 0..1000000 {
                *cnt.as_ref().lock().expect("ok") += 1;
            }
            logger::info!("finished heavy task #{i}/{n_tasks}");
        });
    }
    js
}

#[cfg(test)]
mod test {
    use crate::test_func;

    #[test]
    fn test() {
        test_func();
    }
}
