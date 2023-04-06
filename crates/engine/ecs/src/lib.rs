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
    use crate::ecs::Ecs;

    struct CompA {
        pub _a: u32,
    }

    struct CompB {
        pub _b: usize,
        pub _c: f64,
    }

    #[test]
    fn usage_test() {
        let mut ecs = Ecs::default();
        
        let e0 = ecs.create();
        let e1 = ecs.create();
        let e2 = ecs.create();
        
        ecs.add(e0, CompA { _a: 5489 });
        ecs.add(e1, CompB { _b: 50, _c: 5.0 });
        ecs.add(e2, CompA { _a: 15 });
        ecs.add(e2, CompB { _b: 5454, _c: 5563.0 });

        ecs.destroy(e1);
        ecs.remove::<CompA>(e2);
        
        ecs.destroy(e0);
        ecs.destroy(e2);
    }
}