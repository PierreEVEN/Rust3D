#[derive(Debug)]
pub struct ListOf<T> {
    items: Vec<T>
}

impl<T> ListOf<T> {
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
}
