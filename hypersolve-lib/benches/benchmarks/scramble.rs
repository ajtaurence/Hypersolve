use super::*;
use rand;

fn scramble_benchmark(c: &mut Criterion) {
    c.bench_function("scramble", |b| {
        b.iter(|| {
            let index = rand::random::<u128>() % N_CUBE_STATES;
            generate_scramble(black_box(index))
        })
    });
}

criterion_group! {scramble, scramble_benchmark}
