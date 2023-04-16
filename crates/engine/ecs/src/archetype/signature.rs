use std::hash::{Hash, Hasher};
use std::ops;
use crate::component::ComponentID;

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct ArchetypeSignature {
    components: Vec<ComponentID>,
}

impl ArchetypeSignature {
    pub fn new(components: Vec<ComponentID>) -> ArchetypeSignature {
        let mut components = components;
        components.sort();
        ArchetypeSignature {
            components,
        }
    }
    
    pub fn insert(&self, component: ComponentID) -> ArchetypeSignature {
        let mut components = self.components.clone();
        if let Err(index) = components.binary_search(&component) { components.insert(index, component); }
        ArchetypeSignature {
            components,
        }
    }

    pub fn erase(&self, component: ComponentID) -> ArchetypeSignature {
        let mut components = self.components.clone();
        if let Ok(index) = components.binary_search(&component) { components.remove(index); }
        ArchetypeSignature {
            components,
        }
    }
    
    pub fn count(&self) -> usize {
        self.components.len()
    }
    
    pub fn ids(&self) -> &Vec<ComponentID> {
        &self.components
    }
    
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
    
    pub fn index_of(&self, component: &ComponentID) -> usize {
        for (i, c) in self.components.iter().enumerate() {
            if *component == *c {
                return i
            }
        }
        logger::fatal!("signature doesn't contains component {:?}", component)
    }
}

impl ops::BitAnd<ComponentID> for &ArchetypeSignature {
    type Output = bool;
    fn bitand(self, rhs: ComponentID) -> Self::Output {
        self.components.contains(&rhs)
    }
}

impl ops::BitAnd for &ArchetypeSignature {
    type Output = bool;
    fn bitand(self, rhs: &ArchetypeSignature) -> Self::Output {
        let mut l_index = 0;
        for component in &rhs.components {
            let mut found = false;
            for index in l_index..self.components.len() {
                if self.components[index] == *component {
                    found = true;
                    l_index = index;
                    break;
                }
            }
            if !found { return false }
        }
        true
    }
}

impl From<Vec<ComponentID>> for ArchetypeSignature {
    fn from(value: Vec<ComponentID>) -> Self {
        Self::new(value)
    }
}

impl From<Vec<ArchetypeSignature>> for ArchetypeSignature {
    fn from(value: Vec<ArchetypeSignature>) -> Self {
        let mut count = 0;
        for id in &value { count += id.count() }
        let mut components = Vec::with_capacity(count);
        for id in &value { for comp in &id.components { components.push(*comp); } }
        
        components.sort();
        components.dedup_by(|a, b| {a == b});
        
        Self::new(components)
    }
}

impl Hash for ArchetypeSignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.components.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::archetype::signature::ArchetypeSignature;
    use crate::component::ComponentID;

    #[test]
    fn test_compare() {        
        let a = ArchetypeSignature::new(vec![ComponentID::of::<i32>(), ComponentID::of::<f64>(), ComponentID::of::<bool>()]);
        let b = ArchetypeSignature::new(vec![ComponentID::of::<bool>(), ComponentID::of::<i32>()]);

        assert_ne!(a, b);
        assert!(&a & &a);
        assert!(&b & &b);
        assert!(&a & &b);
        assert!(!(&b & &a));

        assert!(&a & ComponentID::of::<i32>());
        assert!(&a & ComponentID::of::<f64>());
        assert!(&a & ComponentID::of::<i32>());
        assert!(!(&b & ComponentID::of::<f64>()));
        
        let c = a.erase(ComponentID::of::<f64>());
        let d = b.insert(ComponentID::of::<f64>());
        
        assert_eq!(c, b);
        assert_eq!(d, a);
    }
}