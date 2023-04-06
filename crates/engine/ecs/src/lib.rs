pub mod entity;
pub mod archetype;
pub mod ecs;
pub mod component;
pub mod id_generator;

/*
TESTS
 */

#[cfg(test)]
pub mod tests {
    use crate::component::{Component, ComponentID};
    use crate::ecs::Ecs;

    pub struct CompA {
        pub a: u32,
    }

    pub struct CompB {
        pub b: usize,
        pub c: f64,
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
        let mut ecs = Ecs::default();
        ecs.add(0, CompA { a: 0 });
        ecs.remove::<CompA>(0);
    }
}