
/*
WORK IN PROGRESS
 */

pub trait Component: Default + 'static { fn id() -> ComponentID; }

use crate::archetype::archetype::{Archetype, ArchetypeID};
use crate::archetype::signature::ArchetypeSignature;
use crate::component::ComponentID;
use crate::ecs::Ecs;

pub struct Query<'ecs, Cs: ComponentBatch> {
    signature: ArchetypeSignature,
    _ids : Cs::ComponentIDs,
    ecs: &'ecs mut Ecs,
    archetypes: Vec<ArchetypeID>,
}

impl<'ecs, Cs: ComponentBatch> Query<'ecs, Cs> {
    pub fn new(ecs: &'ecs mut Ecs) -> Self {
        Self {
            signature: Cs::ids(),
            _ids: Cs::ComponentIDs::default(),
            ecs,
            archetypes: vec![],
        }
    }
    pub fn for_each<Fn: FnMut(Cs::Item<'ecs>)>(&mut self, mut func: Fn) {
        self.update_archetypes();

        for archetype in &self.archetypes {
            let archetype = self.ecs.get_archetype_mut(archetype);
            let entity_count = archetype.entity_count();
            for i in 0..entity_count {
                func(Cs::fetch(archetype, i));
            }
        }
    }

    pub fn update_archetypes(&mut self) {
        self.archetypes = self.ecs.match_archetypes(&self.signature);
    }
}

pub trait ComponentBatch {
    type ComponentType;
    type ComponentIDs: Default;
    type Item<'ecs>;
    fn ids() -> ArchetypeSignature;
    fn initialized() -> Self::ComponentType;
    fn fetch<'ecs>(archetype: &mut Archetype, index: usize) -> Self::Item<'ecs>;
}

impl<T: Component> ComponentBatch for &T {
    type ComponentType = T;
    type ComponentIDs = ArchetypeSignature;
    type Item<'ecs> = &'ecs T;

    fn ids() -> ArchetypeSignature { vec![T::id()].into() }
    fn initialized() -> Self::ComponentType {
        T::default()
    }
    
    fn fetch<'ecs>(archetype: &mut Archetype, index: usize) -> Self::Item<'ecs> {
        &archetype.data[0].as_component::<Self::ComponentType>()[index]
    }
}

impl<T: Component> ComponentBatch for &mut T {
    type ComponentType = T;
    type ComponentIDs = ArchetypeSignature;
    type Item<'ecs> = &'ecs mut T;

    fn ids() -> ArchetypeSignature { vec![T::id()].into() }
    fn initialized() -> Self::ComponentType {
        T::default()
    }
    
    fn fetch<'ecs>(archetype: &mut Archetype, index: usize) -> Self::Item<'ecs> {
        &mut archetype.data[0].as_component_mut::<Self::ComponentType>()[index]        
    }
}

macro_rules! batch_for_tuples {
    ($($name:ident), *) => {            
        impl<$($name: ComponentBatch + 'static), *> ComponentBatch for ($($name),*) {
            type ComponentType = ($($name::ComponentType),*);
            type ComponentIDs = ArchetypeSignature;
            type Item<'ecs> = ($($name::Item<'ecs>),*);
            
            fn ids() -> ArchetypeSignature { 
                vec![$($name::ids()), *].into()
            }                
            fn initialized() -> Self::ComponentType { ($($name::initialized()),*) }
            
            fn fetch<'ecs>(_archetype: &mut Archetype, _index: usize) -> Self::Item<'ecs> {
                ($($name::fetch(_archetype, _index)), *)
            }
        }
    }
}

/*
impl<T1: ComponentBatch + 'static, T2: ComponentBatch + 'static> ComponentBatch for (T1, T2) {
    type ComponentType = (T1::ComponentType, T2::ComponentType);
    type ComponentIDs = ArchetypeSignature;
    type Item<'ecs> = (T1::Item<'ecs>, T2::Item<'ecs>);
    
    fn ids() -> ArchetypeSignature {
        vec![T1::ids(), T2::ids()].into()
    }
    fn initialized() -> Self::ComponentType { (T1::initialized(), T2::initialized()) }
    
    fn fetch<'ecs>(_archetype: &mut Archetype, _index: usize) -> Self::Item<'ecs> {
        (T1::fetch(_archetype, _index), T2::fetch(_archetype, _index))
    }
}
 */

batch_for_tuples!(T1, T2);
batch_for_tuples!(T1, T2, T3);
batch_for_tuples!(T1, T2, T3, T4);
batch_for_tuples!(T1, T2, T3, T4, T5);
batch_for_tuples!(T1, T2, T3, T4, T5, T6);
batch_for_tuples!(T1, T2, T3, T4, T5, T6, T7);
batch_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8);
