use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use simulator::build_grid::build_grid_sim;

fn performance_simulation_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_simulation_creation");
    let mut size: u32 = 100;
    for _i in 1..4 {
        size *= 2;
        let mut sim_builder = build_grid_sim(size);
        // build once to populate the cache
        let build = sim_builder.build();
        println!("Build finished with {} nodes", build.nodes.len());
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &_size|{
            b.iter(
                || {
                    sim_builder.build()
                }
            )
        });

    }
    group.finish()
}

criterion_group!(benches, performance_simulation_creation);
criterion_main!(benches);