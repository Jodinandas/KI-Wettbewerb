use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use simulator::simple::simulation::Simulator;
use simulator::simple::node::{Street, IONode, Crossing};

/// Builds a grid with side length `grid_side_len` 
/// The edges are IONodes, the crossings and IONodes
/// are connected to neighbours with streets
fn build_grid_sim(grid_side_len: u32) -> Simulator{
    // generate a number of streets connected with crossings in a grid
    //    0 1  2  3  4  5  6  7
    // 0    IO IO IO IO IO IO
    // 1 IO C  C  C  C  C  C  IO
    // 2 IO C  C  C  C  C  C  IO
    // 3 IO C  C  C  C  C  C  IO
    // 4 IO C  C  C  C  C  C  IO
    // 5 IO C  C  C  C  C  C  IO
    // 6 IO C  C  C  C  C  C  IO
    // 7    IO IO IO IO IO IO
    let mut sim = Simulator::new();
    sim.delay(0).max_iter(Some(10000));
    for i in 0..grid_side_len {
        for j in 0..grid_side_len {
            let is_corner = (i == 0 || i == grid_side_len-1) && (j == 0 || j == grid_side_len-1);
            let is_lower_edge = i == 0 || j == 0;
            let is_higher_edge =  i == grid_side_len-1 || j == grid_side_len-1;
            let is_edge = is_lower_edge || is_higher_edge;
            if is_corner {
                sim.add_node(Street::new().into());
                continue;
            }
            match is_edge {
                true => sim.add_node(IONode::new().into()),
                false => sim.add_node(Crossing::new().into())
            };

        }
    }
    for i in 0..grid_side_len {
        for j in 0..grid_side_len {
            let is_corner = (i == 0 || i == grid_side_len-1) && (j == 0 || j == grid_side_len-1);
            let is_lower_edge = i == 0 || j == 0;
            let is_higher_edge =  i == grid_side_len-1 || j == grid_side_len-1;
            let is_right_edge = j == grid_side_len-1;
            let is_bottom_edge = i == grid_side_len-1;
            let is_edge = is_lower_edge || is_higher_edge;
            if is_corner {continue;}
            if is_edge {
                if is_right_edge && (i != 0 && i != grid_side_len-1) {
                    sim.connect_with_street(
                        (i * grid_side_len + j ) as usize,
                        (i * grid_side_len + j - 1 ) as usize,
                        1
                    ).unwrap();
                    sim.connect_with_street(
                        (i * grid_side_len + j - 1 ) as usize,
                        (i * grid_side_len + j ) as usize,
                        1
                    ).unwrap();
                } else if is_bottom_edge  && (j != 0 && j != grid_side_len-1){
                    sim.connect_with_street(
                        (i * grid_side_len + j ) as usize,
                        ((i-1) * grid_side_len + j) as usize,
                        1
                    ).unwrap();
                    sim.connect_with_street(
                        ((i-1) * grid_side_len + j) as usize,
                        (i * grid_side_len + j ) as usize,
                        1
                    ).unwrap();
                }
            } else {
                sim.connect_with_street(
                    (i * grid_side_len + j ) as usize,
                    (i * grid_side_len + j - 1 ) as usize,
                    1
                ).unwrap();
                sim.connect_with_street(
                    (i* grid_side_len + j ) as usize,
                    ((i-1) * grid_side_len + j ) as usize,
                    1
                ).unwrap();
                sim.connect_with_street(
                    (i * grid_side_len + j - 1 ) as usize,
                    (i * grid_side_len + j ) as usize,
                    1
                ).unwrap();
                sim.connect_with_street(
                    ((i-1) * grid_side_len + j ) as usize,
                    (i* grid_side_len + j ) as usize,
                    1
                ).unwrap();
            }

        }
    }
    sim
}

fn simulation_performance_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_performance_bench");
    let mut size: u32 = 100;
    for _i in 1..5 {
        size *= 2;
        let mut sim = build_grid_sim(size);
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