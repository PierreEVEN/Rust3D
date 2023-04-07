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
    use std::any::{type_name, TypeId};

    use crate::component::ComponentID;
    use crate::ecs::Ecs;

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

    pub struct Query<Cs: ComponentBatch> {
        ids: Cs::ComponentIDs,
    }

    impl<Cs: ComponentBatch> Query<Cs> {
        pub fn new() -> Self {
            Self {
                ids: Cs::ids()
            }
        }
        pub fn for_each<Fn: FnMut(Cs)>(&mut self, mut func: Fn) {
            
            
            func()
            
        }
    }

    pub trait ComponentBatch {
        type Component;
        type ComponentIDs;
        fn ids() -> Self::ComponentIDs;
        fn initialized() -> Self::Component;
    }

    impl<T: Component + Default> ComponentBatch for &T {
        type Component = T;
        type ComponentIDs = ComponentID;

        fn ids() -> Self::ComponentIDs {
            T::id()
        }

        fn initialized() -> Self::Component {
            T::default()
        }
    }

    impl<T: Component + Default> ComponentBatch for &mut T {
        type Component = T;
        type ComponentIDs = ComponentID;

        fn ids() -> Self::ComponentIDs {
            T::id()
        }

        fn initialized() -> Self::Component {
            T::default()
        }
    }

    macro_rules! query_for_tuples {
        ($($name:ident), *) => {            
            impl<$($name: ComponentBatch), *> ComponentBatch for ($($name),*) {
                type Component = ($($name::Component),*);
                type ComponentIDs = ($($name::ComponentIDs),*);
                
                fn ids() -> Self::ComponentIDs {
                    ($($name::ids()),*)
                }
                
                fn initialized() -> Self::Component {
                    ($($name::initialized()),*)
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
        
        let mut query = Query::<(&mut CompA, &CompB)>::new();
        query.for_each(|(a, b)| {
            a.a = b.b as u32 + b.c as u32;
        });
        
        ecs.remove::<CompA>(e2);
        ecs.destroy(e0);
        ecs.destroy(e2);
        ecs.remove::<CompB>(e1);
        ecs.destroy(e1);
    }
}