pub mod entity;
pub mod ecs;
pub mod component;
pub mod id_generator;
pub mod query;
pub mod archetype;

use crate::ecs::Ecs;
use crate::query::{Query};

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

/*
USAGE
 */

pub fn test_func() {
    use std::time::{Instant};
    
    let mut ecs = Ecs::default();

    let e0 = ecs.create();
    let e1 = ecs.create();
    let e2 = ecs.create();
    
    ecs.add(e0, CompA { a: 1 });
    ecs.add(e1, CompB { b: 3, c: 4.0 });
    ecs.add(e2, CompA { a: 2 });
    ecs.add(e2, CompB { b: 5, c: 6.0 });

    let bench_length = 1000000;
    
    println!("REFERENCE OPTIMAL");
    let start =  Instant::now();
    let mut reference_data = vec![];
    for i in 0..bench_length {
        reference_data.push(CompC { x: i, _y: Default::default()});
    }
    println!("--Creation : {}ms", start.elapsed().as_micros() as f64 / 1000.0);
    let start =  Instant::now();
    for i in &mut reference_data { i.x += 1; }
    println!("--Iteration : {}ms", start.elapsed().as_micros() as f64 / 1000.0);

    println!("REFERENCE NOT OPTIMAL");
    let start =  Instant::now();
    let mut reference_data = vec![];
    for i in 0..bench_length {
        reference_data.push(Box::new(CompC { x: i, _y: Default::default()}));
    }
    println!("--Creation : {}ms", start.elapsed().as_micros() as f64 / 1000.0);
    let start =  Instant::now();
    for i in &mut reference_data { i.x += 1; }
    println!("--Iteration : {}ms", start.elapsed().as_micros() as f64 / 1000.0);
    
    println!("ECS");
    let start = Instant::now();
    let mut entities = Vec::with_capacity(bench_length as usize);
    for i in 0..bench_length {
        let entity = ecs.create();
        ecs.add(entity, CompC { x: i, _y: Default::default()});
        entities.push(entity);
    }
    println!("--Creation : {}ms", start.elapsed().as_micros() as f64 / 1000.0);    
    let start = Instant::now();
    Query::<&mut CompC>::new(&mut ecs).for_each(|a| { a.x += 1; });
    println!("--Iteration : {}ms", start.elapsed().as_micros() as f64 / 1000.0);
    
    let mut iter = 0;
    Query::<&mut CompC>::new(&mut ecs).for_each(|a| { iter += 1; assert_eq!(a.x, iter, "index {iter} is wrong"); });

    let start = Instant::now();
    entities.reverse();
    for entity in entities {
        ecs.destroy(entity);
    }
    println!("--Deletion : {}ms", start.elapsed().as_micros() as f64 / 1000.0);
    
    
    println!("ECS STATISTICS");
    ecs.print_stats();
    
    Query::<&CompB>::new(&mut ecs).for_each(|b| {
        println!("ITER B :  b = {}, c = {}", b.b, b.c);
    });
    
    Query::<(&mut CompA, &CompB)>::new(&mut ecs).for_each(|(a, b)| {
        a.a = b.b as u32 + b.c as u32;
        println!("ITER A ET B : a = {}, b = {}, c = {}", a.a, b.b, b.c);
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
