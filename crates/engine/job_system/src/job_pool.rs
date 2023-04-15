use std::collections::VecDeque;
use std::mem::{MaybeUninit, size_of};
use crate::JobPtr;

pub struct JobData {
    valid: bool,
    runner: MaybeUninit<fn(JobPtr)>,
    payload: Vec<u8>,
}

impl JobData {
    pub fn new<Fn: 'static + FnOnce()>(job_fn: Fn) -> Self {

        let payload = Vec::with_capacity(size_of::<Fn>());
        unsafe { (payload.as_ptr() as *mut Fn).write(job_fn); }
        
        Self {
            valid: true,
            runner: MaybeUninit::new(move |job_ptr| {
                unsafe {
                    let mut func_ptr = MaybeUninit::<Fn>::zeroed();
                    func_ptr.write((job_ptr.read().payload.as_ptr() as *mut Fn).read());
                    func_ptr.assume_init()();
                }
            }),
            payload,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
    
    pub fn execute(&mut self) {
        unsafe { (self.runner.assume_init())(JobPtr::new(self)) }
    }
}

#[derive(Default)]
pub struct JobPool {
    jobs: VecDeque<JobData>,
}

impl JobPool {
    pub fn new() -> Self {
        Self { jobs: VecDeque::new() }
    }

    pub fn push(&mut self, job: JobData) -> JobPtr {
        self.jobs.push_back(job);
        JobPtr::new(self.jobs.back_mut().expect("should not be empty there"))
    }

    pub fn pop(&mut self) -> Option<JobData> {
        self.jobs.pop_front()
    }

    pub fn is_empty(&self) -> bool { self.jobs.is_empty() }
}