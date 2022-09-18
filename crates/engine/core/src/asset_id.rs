use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub struct AssetID {
    pub name: String,
}

impl AssetID {
    pub fn from(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl Hash for AssetID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

impl PartialEq<Self> for AssetID {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for AssetID {
    
}