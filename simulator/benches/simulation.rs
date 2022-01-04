use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use simulator::{
    datastructs::IntMut,
    debug::build_grid_sim,
    path::{MovableServer, PathAwareCar},
};

fn simulation_performance_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_performance_bench");
    let mut size: u32 = 100;
    for _i in 1..5 {
        size *= 2;
        let mut sim_builder = build_grid_sim(size);
        let mut mv_server = MovableServer::<PathAwareCar>::new();
        mv_server.register_simulator_builder(&sim_builder);
        let mv_server = IntMut::new(mv_server);
        // make sure it works when using the cached value
        drop(sim_builder.build(&mv_server));
        sim_builder.drop_cache();
        let mut sim = sim_builder.build(&mv_server);
        // iterate a few times to get the cars to enter the simulation
        for _ in 0..100 {
            sim.sim_iter()
        }
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &_size| {
            b.iter(|| sim.sim_iter())
        });
    }
    group.finish()
}

criterion_group!(benches, simulation_performance_bench);
criterion_main!(benches);
