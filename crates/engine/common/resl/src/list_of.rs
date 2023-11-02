use std::ops::AddAssign;

#[derive(Debug, Clone)]
pub struct ListOf<T: Clone> {
    items: Vec<T>,
}

impl<T: Clone> ListOf<T> {
    pub fn new() -> Self { Self { items: vec![] } }

    pub fn push(mut self, instr: T) -> Self {
        self.items.push(instr);
        self
    }

    pub fn push_front(mut self, instr: T) -> Self {
        self.items.insert(0, instr);
        self
    }

    pub fn concat(mut self, other: &mut Self) -> Self {
        self.items.append(&mut other.items);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> {
        self.items.iter_mut()
    }

    pub fn join<V: Default + for<'a> AddAssign<&'a mut U>, U: Clone, Fn: FnMut(&T) -> U>(&self, sep: U, mut f: Fn) -> V {
        let mut list = V::default();
        let mut first = true;
        for item in self.iter() {
            if !first {
                list += &mut sep.clone();
            }
            first = false;
            list += &mut f(item);
        }
        list
    }
}
