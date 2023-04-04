use std::collections::LinkedList;
use std::ops::{Add, Sub};
use num::One;

#[derive(Default)]
pub struct IdBase<C: Default + Copy> {
    free_ids: LinkedList<C>,
    max_id: C,
}

impl<C: Default + Copy + Add<Output = C> + Sub<Output = C> + One> IdBase<C> {
    pub fn acquire(&mut self) -> C
    {
        if !self.free_ids.is_empty() {
            return self.free_ids.pop_back().expect("list should not be empty there");
        }
        self.max_id = self.max_id + One::one();
        self.max_id - One::one()
    }
    
    pub fn release(&mut self, id: C) {
        self.free_ids.push_back(id);
    }
}