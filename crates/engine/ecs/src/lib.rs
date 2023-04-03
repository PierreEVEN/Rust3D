pub mod entity;
pub mod archetype;
pub mod ecs;
pub mod component;


#[cfg(test)]
mod tests {
    use crate::component::{Component, ComponentID};
    use crate::ecs::Ecs;
    use crate::entity::{EntityRegistry};

    #[derive(Default)]
    struct Component1 {
        a: u32,
    }

    impl Component for Component1 {
        fn component_id() -> ComponentID {
            0
        }
    }

    #[derive(Default)]
    struct Component2 {
        b: u32,
    }

    impl Component for Component2 {
        fn component_id() -> ComponentID {
            1
        }
    }

    #[test]
    fn ecs_impl_test() {
        let mut ecs = Ecs::default();
        let entity = ecs.new();
        ecs.add(entity, Component1::default());
        ecs.remove::<Component1>(entity);
        ecs.destroy(entity);
    }
}