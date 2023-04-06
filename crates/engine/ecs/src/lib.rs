pub mod entity;
pub mod archetype;
pub mod ecs;
pub mod component;
pub mod id_generator;
pub mod system;

/*
TESTS
 */

#[cfg(test)]
mod tests {
    use crate::ecs::Ecs;

    struct CompA {
        pub a: u32,
    }

    struct CompB {
        pub b: usize,
        pub c: f64,
    }
    #[macro_export]
    macro_rules! lambda {
        ( $($x:expr, $ty:ty),* ) => {
            
        };
        ( $a:block ) => {
            
        };
    }

    #[test]
    fn usage_test() {
        let mut ecs = Ecs::default();

        let e0 = ecs.create();
        let e1 = ecs.create();
        let e2 = ecs.create();

        ecs.add(e0, CompA { a: 5489 });
        ecs.add(e1, CompB { b: 50, c: 5.0 });
        ecs.add(e2, CompA { a: 15 });
        ecs.add(e2, CompB { b: 5454, c: 5563.0 });

        ecs.remove::<CompA>(e2);

        ecs.destroy(e0);
        ecs.destroy(e2);
        ecs.remove::<CompB>(e1);
        ecs.destroy(e1);

        lambda!(_compa, &CompA, _comp_b, &CompB);

        ecs.add_system("MyWorkFlow", |comp_a: &mut CompA, comp_b: &mut CompB| {
            comp_b.c += comp_a.a as f64;
            comp_b.b = comp_b.c  as usize * 2 as usize;
        });
        
        ecs.add_system("MySecondWorkflow", |comp_a: &mut CompA| {
            comp_a.a += 2;
        }).after("MyWorkFlow");
        
        ecs.add_system("MyThirdWorkflow", |comp_b: &mut CompB| {
            comp_b.b = comp_b.c as usize - 10;            
        }).before("MySecondWorkflow");
    }
}