pub mod archetype;
pub mod component;
pub mod ecs;
pub mod entity;
pub mod id_generator;
pub mod query;

use crate::ecs::Ecs;
use crate::query::Query;

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

#[derive(Default)]
struct CompC {
    pub _y: [u64; 32],
    pub x: u64,
}

#[derive(Default)]
struct CompD {
    pub str: String,
}

/*
USAGE
 */

pub fn test_func() {
    use std::time::Instant;

    let mut ecs = Ecs::default();

    let e0 = ecs.create();
    let e1 = ecs.create();
    let e2 = ecs.create();

    /*
    ecs.add(e0, CompA { a: 1 });
    ecs.add(e1, CompB { b: 3, c: 4.0 });
    ecs.add(e2, CompA { a: 2 });
    ecs.add(e2, CompB { b: 5, c: 6.0 });
     */

    let bench_length = 100;

    logger::info!("REFERENCE OPTIMAL");
    let start = Instant::now();
    let mut reference_data = vec![];
    for i in 0..bench_length {
        reference_data.push(CompC {
            x: i,
            _y: Default::default(),
        });
    }
    logger::info!(
        "--Creation : {}ms",
        start.elapsed().as_micros() as f64 / 1000.0
    );
    let start = Instant::now();
    for i in &mut reference_data {
        i.x += 1;
    }
    logger::info!(
        "--Iteration : {}ms",
        start.elapsed().as_micros() as f64 / 1000.0
    );

    logger::info!("REFERENCE NOT OPTIMAL");
    let start = Instant::now();
    let mut reference_data = vec![];
    for i in 0..bench_length {
        reference_data.push(Box::new(CompC {
            x: i,
            _y: Default::default(),
        }));
    }
    logger::info!(
        "--Creation : {}ms",
        start.elapsed().as_micros() as f64 / 1000.0
    );
    let start = Instant::now();
    for i in &mut reference_data {
        i.x += 1;
    }
    logger::info!(
        "--Iteration : {}ms",
        start.elapsed().as_micros() as f64 / 1000.0
    );

    logger::info!("ECS");
    let start = Instant::now();
    let mut entities = Vec::with_capacity(bench_length as usize);
    /*
    for i in 0..bench_length {
        let entity = ecs.create();
        ecs.add(
            entity,
            CompC {
                x: i,
                _y: Default::default(),
            },
        );
        entities.push(entity);
    }
    
     */
    logger::info!(
        "--Creation : {}ms",
        start.elapsed().as_micros() as f64 / 1000.0
    );
    let start = Instant::now();
    Query::<&mut CompC>::new(&mut ecs).for_each(|a| {
        a.x += 1;
    });
    logger::info!(
        "--Iteration : {}ms",
        start.elapsed().as_micros() as f64 / 1000.0
    );

    let entity = ecs.create();
    ecs.add(
        entity,
        CompD {
            str: "Bonjour, je suis un string".to_string()
        },
    );
    
    let mut iter = 0;
    Query::<&mut CompC>::new(&mut ecs).for_each(|a| {
        iter += 1;
        assert_eq!(a.x, iter, "index {iter} is wrong");
    });

    let start = Instant::now();
    entities.reverse();
    for entity in entities {
        ecs.destroy(entity);
    }
    logger::info!(
        "--Deletion : {}ms",
        start.elapsed().as_micros() as f64 / 1000.0
    );

    logger::info!("ECS STATISTICS");
    ecs.print_stats();

    Query::<&CompA>::new(&mut ecs).for_each(|a| {
        logger::info!("ITER B :  a = {}", a.a);
    });

    Query::<&CompB>::new(&mut ecs).for_each(|b| {
        logger::info!("ITER B : b = {}, c = {}", b.b, b.c);
    });

    Query::<(&mut CompA, &CompB)>::new(&mut ecs).for_each(|(a, b)| {
        logger::info!("ITER A ET B : a = {}, b = {}, c = {}", a.a, b.b, b.c);
    });
    Query::<(&mut CompB, &CompA)>::new(&mut ecs).for_each(|(b, a)| {
        logger::info!("ITER A ET B : a = {}, b = {}, c = {}", a.a, b.b, b.c);
    });
    Query::<(&CompB, &CompA)>::new(&mut ecs).for_each(|(b, a)| {
        logger::info!("ITER A ET B : a = {}, b = {}, c = {}", a.a, b.b, b.c);
    });
    Query::<(&mut CompB, &mut CompA)>::new(&mut ecs).for_each(|(b, a)| {
        logger::info!("ITER A ET B : a = {}, b = {}, c = {}", a.a, b.b, b.c);
    });
    Query::<(&CompB, &mut CompA)>::new(&mut ecs).for_each(|(b, a)| {
        logger::info!("ITER A ET B : a = {}, b = {}, c = {}", a.a, b.b, b.c);
    });

    Query::<&CompD>::new(&mut ecs).for_each(|d| {
        logger::info!("D : str = {}", d.str);
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
mod test {
    #[test]
    fn usage_test() {
        crate::test_func()
    }
}
