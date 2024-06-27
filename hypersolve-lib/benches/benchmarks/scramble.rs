use rand;

use super::*;

fn scramble_benchmark(c: &mut Criterion) {
    c.bench_function("scramble", |b| {
        b.iter(|| {
            let index = CubeIndex::try_from(rand::random::<u128>() % N_CUBE_STATES).unwrap();
            new_scramble(black_box(index))
        })
    });
}

criterion_group! {scramble, scramble_benchmark}
