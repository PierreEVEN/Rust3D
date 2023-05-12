use crate::archetype::signature::ArchetypeSignature;
use crate::archetype::{Archetype, ArchetypeID};
use crate::component::ComponentID;
use crate::ecs::Ecs;

pub struct Query<'ecs, Cs: ComponentBatch> {
    signature: ArchetypeSignature,
    ecs: &'ecs mut Ecs,
    ids: Vec<ComponentID>,
    _default_value: Cs::ComponentType,
    archetypes: Vec<(ArchetypeID, Vec<usize>)>,
}

impl<'ecs, Cs: ComponentBatch> Query<'ecs, Cs> {
    pub fn new(ecs: &'ecs mut Ecs) -> Self {
        Self {
            signature: Cs::ids().into(),
            ecs,
            archetypes: vec![],
            ids: Cs::ids(),
            _default_value: Cs::initialized(),
        }
    }

    #[inline]
    pub fn for_each<Fn: FnMut(Cs::Item<'ecs>)>(&mut self, mut func: Fn) {
        self.update_archetypes();

        for (archetype, mapping) in &self.archetypes {
            let archetype = self.ecs.get_archetype_mut(archetype);
            let entity_count = archetype.entity_count();
            for i in 0..entity_count {
                func(Cs::fetch(archetype, i, 0, mapping));
            }
        }
    }

    pub fn update_archetypes(&mut self) {
        let archetypes = self.ecs.match_archetypes(&self.signature);

        let mut archetype_mapped = vec![];

        for archetype in &archetypes {
            let mut mapping = vec![];
            for id in &self.ids {
                mapping.push(self.ecs.get_archetype(archetype).signature().index_of(id))
            }
            archetype_mapped.push((*archetype, mapping))
        }

        self.archetypes = archetype_mapped;
    }
}

pub trait ComponentBatch {
    type ComponentType;
    type Item<'ecs>;
    fn ids() -> Vec<ComponentID>;
    fn initialized() -> Self::ComponentType;
    fn fetch<'ecs>(
        archetype: &mut Archetype,
        index: usize,
        comp_index: usize,
        mapping: &[usize],
    ) -> Self::Item<'ecs>;
}

impl<T: Default + 'static> ComponentBatch for &T {
    type ComponentType = T;
    type Item<'ecs> = &'ecs T;

    fn ids() -> Vec<ComponentID> {
        vec![ComponentID::of::<T>()]
    }
    fn initialized() -> Self::ComponentType {
        T::default()
    }

    fn fetch<'ecs>(
        archetype: &mut Archetype,
        index: usize,
        comp_index: usize,
        mapping: &[usize],
    ) -> Self::Item<'ecs> {
        &archetype.data[mapping[comp_index]].as_component::<Self::ComponentType>()[index]
    }
}

impl<T: Default + 'static> ComponentBatch for &mut T {
    type ComponentType = T;
    type Item<'ecs> = &'ecs mut T;

    fn ids() -> Vec<ComponentID> {
        vec![ComponentID::of::<T>()]
    }
    fn initialized() -> Self::ComponentType {
        T::default()
    }

    fn fetch<'ecs>(
        archetype: &mut Archetype,
        index: usize,
        comp_index: usize,
        mapping: &[usize],
    ) -> Self::Item<'ecs> {
        &mut archetype.data[mapping[comp_index]].as_component_mut::<Self::ComponentType>()[index]
    }
}

macro_rules! batch_for_tuples {
    ($($name:ident, $state:expr), *) => {
        impl<$($name: ComponentBatch + 'static), *> ComponentBatch for ($($name),*) {
            type ComponentType = ($($name::ComponentType),*);
            type Item<'ecs> = ($($name::Item<'ecs>),*);

            fn ids() -> Vec<ComponentID> {
                let mut ids= vec![];
                $(let mut other = $name::ids(); ids.append(&mut other);) *
                ids
            }
            fn initialized() -> Self::ComponentType { ($($name::initialized()),*) }

            fn fetch<'ecs>(archetype: &mut Archetype, index: usize, _: usize, mapping: &[usize]) -> Self::Item<'ecs> {
                ($($name::fetch(archetype, index, $state, mapping)), *)
            }
        }
    }
}

/*
impl<T1: ComponentBatch + 'static, T2: ComponentBatch + 'static> ComponentBatch for (T1, T2) {
    type ComponentType = (T1::ComponentType, T2::ComponentType);
    type Item<'ecs> = (T1::Item<'ecs>, T2::Item<'ecs>);

    fn ids() -> Vec<ComponentID> {
        let mut ids= vec![];
        let mut other = T1::ids();
        ids.append(&mut other);
        let mut other = T2::ids();
        ids.append(&mut other);
        ids
    }
    fn initialized() -> Self::ComponentType { (T1::initialized(), T2::initialized()) }

    fn fetch<'ecs>(_archetype: &mut Archetype, _index: usize, _: usize, mapping: &[usize]) -> Self::Item<'ecs> {
        (T1::fetch(_archetype, _index, mapping[0], mapping), T2::fetch(_archetype, _index, mapping[1], mapping))
    }
}
*/

batch_for_tuples!(T1, 0, T2, 1);
batch_for_tuples!(T1, 0, T2, 1, T3, 2);
batch_for_tuples!(T1, 0, T2, 1, T3, 2, T4, 3);
batch_for_tuples!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4);
batch_for_tuples!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5);
batch_for_tuples!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6);
batch_for_tuples!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7);
