use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use simulator::{
    datastructs::IntMut,
    debug::build_grid_sim,
    path::{MovableServer, PathAwareCar},
};

fn performance_simulation_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_simulation_creation");
    let mut size: u32 = 100;
    for _i in 1..4 {
        size *= 2;
        let mut sim_builder = build_grid_sim(size);
        let mut mv_server = MovableServer::<PathAwareCar>::new();
        mv_server.register_simulator_builder(&sim_builder);
        let mv_server = IntMut::new(mv_server);
        // build once to populate the cache
        let build = sim_builder.build(&mv_server);
        println!("Build finished with {} nodes", build.nodes.len());
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &_size| {
            b.iter(|| sim_builder.build(&mv_server))
        });
    }
    group.finish()
}

criterion_group!(benches, performance_simulation_creation);
criterion_main!(benches);
