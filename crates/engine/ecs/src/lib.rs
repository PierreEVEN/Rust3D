pub mod entity;
pub mod archetype;
pub mod ecs;
pub mod component;
pub mod id_generator;

/*
TESTS
 */

#[cfg(test)]
mod tests {
    use crate::archetype::ArchetypeRegistry;
    use crate::component::{Component, ComponentID, ComponentRegistry};

    struct CompA {
        a: u32,
    }

    struct CompB {
        b: usize,
        c: f64,
    }

    impl Component for CompA {
        fn component_id() -> ComponentID {
            0
        }
    }

    impl Component for CompB {
        fn component_id() -> ComponentID {
            1
        }
    }

    #[test]
    fn usage_test() {
        let mut registry = crate::archetype::ArchetypeRegistry::default();
        let mut components = crate::component::ComponentRegistry::default();

        let id_a = registry.find_or_get([CompA::component_id(), CompB::component_id()].as_slice(), &components);
        let id_b = registry.find_or_get([CompA::component_id()].as_slice(), &components);

        registry.get_archetype_mut(id_a).push_entity(0);

        registry.get_archetype_mut(id_a).drop_entity(0);

        registry.get_archetype_mut(id_a).push_entity(1);
        registry.get_archetype_mut(id_a).push_entity(0);

        registry.get_archetype_mut(id_b).push_entity(2);

        let mut target = registry.get_archetype_mut(id_b);
        let mut src = registry.get_archetype_mut(id_a);

        src.move_entity_to(0, target);

        registry.get_archetype_mut(id_a).drop_entity(0);
        registry.get_archetype_mut(id_b).drop_entity(1);
        registry.get_archetype_mut(id_b).drop_entity(0);
    }
}