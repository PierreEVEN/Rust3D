use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub struct AssetTypeID {
    pub name: String,
}

impl AssetTypeID {
    pub fn from(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl PartialEq<Self> for AssetTypeID {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for AssetTypeID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

impl Eq for AssetTypeID {}
