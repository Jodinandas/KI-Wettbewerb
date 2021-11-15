use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use simulator::debug::build_grid_sim;

fn simulation_builder_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_performance_bench");
    let mut size: u32 = 100;
    for _i in 1..2 {
        size *= 2;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &_size| {
            b.iter(|| {
                build_grid_sim(size);
            })
        });
    }
    group.finish()
}
criterion_group!(benches, simulation_builder_bench);
criterion_main!(benches);
