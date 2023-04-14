
extern crate num_cpus;

use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub type JobHandle = usize;

pub trait Job {
    fn exec(self);
}

pub struct JobImpl<Fn: FnOnce()> {
    job_fn: Fn
}

impl <Fn: FnOnce()>Job for JobImpl<Fn> {
    fn exec(self) {
        (self.job_fn)();
    }
}

impl <Fn: 'static + FnOnce()>JobImpl<Fn> {
    pub fn new(job_fn: Fn) -> Self {
        Self {
            job_fn
        }
    }
}

pub struct Worker {
    worker_id: usize,
    job_pool: Arc<JobPool>,
    running_thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(worker_id: usize, job_pool: Arc<JobPool>) -> Self {
        
        let running_thread = thread::spawn(|| {
            
        });
        
        Self {
            worker_id,
            job_pool,
            running_thread,
        }
    }
    
    pub fn id(&self) -> usize {
        self.worker_id
    }
    
    pub fn push_job(&mut self, job: Box<dyn Job>) -> JobHandle {
        todo!()
    }
}

pub struct JobPool {
    jobs: Vec<Box<dyn Job>>,
}

impl JobPool {
    pub fn new() -> Self {
        Self { jobs: vec![] }
    }
    
    pub fn push(&self, job: &dyn Job) -> JobHandle {
        todo!()
    }
    
    pub fn pop(&self) -> &dyn Job {
        todo!()
    }
}

pub struct JobSystem {
    workers: Vec<Worker>,
    current_worker: usize,
}

impl JobSystem {
    pub fn new(num_parallel_worker: usize) -> Self {
        let mut workers = Vec::with_capacity(num_parallel_worker);
        
        let job_pool = Arc::new(JobPool::new());
        
        for i in 0..num_parallel_worker {
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
    
    pub fn schedule<Fn: 'static + FnOnce()>(&mut self, job_fn: Fn) -> JobHandle {
        // Pick a random worker
        let worker = self.next_worker();
        
        // then queue new job
        worker.push_job(Box::new(JobImpl::new(job_fn)))
    }
    
    pub fn wait_idle(&self, _handle : JobHandle) {
        todo!()
    }
    
}


#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};
    use crate::JobSystem;

    #[test]
    pub fn tests() {
        
        let mut js = JobSystem::new(JobSystem::available_cpu_threads());
        
        let counter = Arc::new(Mutex::new(0));
        
        let cnt = counter.clone();
        let handle = js.schedule(move || {
            for i in 0..10000 {
                *cnt.as_ref().lock().expect("ok") += 1;
            }
            println!("i'll do my task");
        });
        
        js.wait_idle(handle);
        println!("result : {}", counter.as_ref().lock().expect("ok"))
    }
}