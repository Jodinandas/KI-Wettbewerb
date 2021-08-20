use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use simulator::simple::node_builder::{IONodeBuilder, StreetBuilder, CrossingBuilder};
use simulator::simple::simulation_builder::SimulatorBuilder;
use simulator::build_grid::build_grid_sim;


fn simulation_performance_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_performance_bench");
    let mut size: u32 = 100;
    for _i in 1..5 {
        size *= 2;
        let sim_builder = build_grid_sim(size);
        let mut sim = sim_builder.build();
        // iterate a few times to get the cars to enter the simulation
        for _ in 0..100 {
            sim.sim_iter(1.0)
        }
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &_size|{
            b.iter(
                || {
                    sim.sim_iter(1.0)
                }
            )
        });

    }
    group.finish()
}

criterion_group!(benches, simulation_performance_bench);
criterion_main!(benches);