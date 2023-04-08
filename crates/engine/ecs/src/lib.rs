pub mod entity;
pub mod archetype;
pub mod ecs;
pub mod component;
pub mod id_generator;
pub mod system;

use crate::archetype::{ArchetypeID, ComponentData};

use crate::component::ComponentID;
use crate::ecs::Ecs;
use crate::entity::EntityID;

/*
TYPE DECLARATION
 */
#[derive(Default)]
struct CompA {
    pub a: u32,
}

#[derive(Default)]
struct CompB {
    pub b: usize,
    pub c: f64,
}

pub trait Component { fn id() -> ComponentID; }

impl Component for CompB { fn id() -> ComponentID { ComponentID::of::<Self>() } }

impl Component for CompA { fn id() -> ComponentID { ComponentID::of::<Self>() } }

/*
WORK IN PROGRESS
 */

pub struct Query<'ecs, Cs: ComponentBatch> {
    _ids: Cs::ComponentIDs,
    ecs: &'ecs mut Ecs,
    archetypes: Vec<ArchetypeID>,
}

impl<'ecs, Cs: ComponentBatch> Query<'ecs, Cs> {
    pub fn new(ecs: &'ecs mut Ecs) -> Self {
        Self {
            _ids: Cs::ids(),
            ecs,
            archetypes: vec![],
        }
    }
    pub fn for_each<Fn: FnMut(Cs)>(&mut self, mut _func: Fn) {
        self.update_archetypes();
        
        for _archetype in &self.archetypes {
        }
    }

    pub fn update_archetypes(&mut self) {
        self.archetypes.clear();
        for arch in 0..self.ecs.get_archetype_count() {
            let mut valid = true;
            for comp in self.ecs.get_archetype(&(arch as ArchetypeID)).components() {
                if !Cs::has_component(comp) {
                    valid = false;
                    break;
                }
            }
            if valid {
                self.archetypes.push(arch as ArchetypeID);
            }
        }
    }
}

pub trait ComponentBatch {
    type ComponentType;
    type ComponentIDs;
    fn ids() -> Self::ComponentIDs;
    fn initialized() -> Self::ComponentType;
    fn has_component(id: &ComponentID) -> bool;
    fn project_component<'ecs>(entity: &EntityID, comp: &ComponentData) -> &'ecs Self::ComponentType;
}

impl<T: Component + Default> ComponentBatch for &T {
    type ComponentType = T;
    type ComponentIDs = ComponentID;

    fn ids() -> Self::ComponentIDs {
        T::id()
    }
    fn initialized() -> Self::ComponentType {
        T::default()
    }
    fn has_component(id: &ComponentID) -> bool { T::id() == *id }
    fn project_component<'ecs>(_entity: &EntityID, _comp: &ComponentData) -> &'ecs Self::ComponentType {
        todo!()       
    }
}

impl<T: Component + Default> ComponentBatch for &mut T {
    type ComponentType = T;
    type ComponentIDs = ComponentID;

    fn ids() -> Self::ComponentIDs {
        T::id()
    }
    fn initialized() -> Self::ComponentType {
        T::default()
    }
    fn has_component(id: &ComponentID) -> bool { T::id() == *id }
    fn project_component<'ecs>(_entity: &EntityID, _comp: &ComponentData) -> &'ecs Self::ComponentType {
        todo!()
    }
}

macro_rules! query_for_tuples {
        ($($name:ident), *) => {            
            impl<$($name: ComponentBatch), *> ComponentBatch for ($($name),*) {
                type ComponentType = ($($name::ComponentType),*);
                type ComponentIDs = ($($name::ComponentIDs),*);
                
                fn ids() -> Self::ComponentIDs {
                    ($($name::ids()),*)
                }
                
                fn initialized() -> Self::ComponentType {
                    ($($name::initialized()),*)
                }
                
                fn has_component(id: &ComponentID) -> bool { 
                    $($name::has_component(id)) && *
                }
                
                fn project_component<'ecs>(_entity: &EntityID, _comp: &ComponentData) -> &'ecs Self::ComponentType {
                    todo!()       
                }
            }   
        }
    }

query_for_tuples!(T1, T2);
query_for_tuples!(T1, T2, T3);
query_for_tuples!(T1, T2, T3, T4);
query_for_tuples!(T1, T2, T3, T4, T5);
query_for_tuples!(T1, T2, T3, T4, T5, T6);
query_for_tuples!(T1, T2, T3, T4, T5, T6, T7);
query_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8);

/*
USAGE
 */

pub fn test_func() {

    let mut ecs = Ecs::default();

    let e0 = ecs.create();
    let e1 = ecs.create();
    let e2 = ecs.create();
    
    ecs.add(e0, CompA { a: 5489 });
    ecs.add(e1, CompB { b: 50, c: 5.0 });
    ecs.add(e2, CompA { a: 15 });
    ecs.add(e2, CompB { b: 5454, c: 5563.0 });

    let mut query = Query::<(&mut CompA, &CompB)>::new(&mut ecs);
    query.for_each(|(a, b)| {
        a.a = b.b as u32 + b.c as u32;
    });

    ecs.remove::<CompA>(e2);
    ecs.destroy(e0);
    ecs.destroy(e2);
    ecs.remove::<CompB>(e1);
    ecs.destroy(e1);
}


/*
TESTS
 */

#[cfg(test)]
mod tests {
    #[test]
    fn usage_test() {
        crate::test_func()
    }
}
