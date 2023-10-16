#[cfg(not(debug_assertions))]
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fmt::{Display, Formatter};

use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct PassID {
    #[cfg(not(debug_assertions))]
    internal_id: u64,

    #[cfg(debug_assertions)]
    internal_id: String,
}

impl PassID {
    pub fn new(id_name: &str) -> PassID {
        #[cfg(not(debug_assertions))]
        {
            let mut hasher = DefaultHasher::new();
            id_name.hash(&mut hasher);
            Self {
                internal_id: hasher.finish(),
            }
        }
        #[cfg(debug_assertions)]
        Self {
            internal_id: id_name.to_string(),
        }
    }
}

impl Hash for PassID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[cfg(not(debug_assertions))]
        state.write_u64(self.internal_id);

        #[cfg(debug_assertions)]
        state.write(self.internal_id.as_bytes());
    }
}

#[cfg(debug_assertions)]
impl PartialEq<Self> for PassID {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}

#[cfg(not(debug_assertions))]
impl PartialEq<Self> for PassID {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}

impl Display for PassID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.internal_id)
    }
}

impl Eq for PassID {}
