use super::node_builder::{CrossingBuilder, IONodeBuilder};
use super::simulation_builder::SimulatorBuilder;
use crate::node_builder::{Direction, NodeBuilder};
use crate::pathfinding::PathAwareCar;

/// Builds a grid with side length `grid_side_len`
/// The edges are IONodes, the crossings and IONodes
/// are connected to neighbours with streets
pub fn build_grid_sim(grid_side_len: u32, street_len: f32) -> SimulatorBuilder {
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
    let mut sim = SimulatorBuilder::<PathAwareCar>::new();
    sim.with_delay(0).with_max_iter(Some(10000));
    for i in 0..grid_side_len {
        for j in 0..grid_side_len {
            let is_corner =
                (i == 0 || i == grid_side_len - 1) && (j == 0 || j == grid_side_len - 1);
            let is_lower_edge = i == 0 || j == 0;
            let is_higher_edge = i == grid_side_len - 1 || j == grid_side_len - 1;
            let is_edge = is_lower_edge || is_higher_edge;
            if is_corner {
                sim.add_node(NodeBuilder::IONode(IONodeBuilder::new()));
                continue;
            }
            match is_edge {
                true => sim.add_node(NodeBuilder::IONode(IONodeBuilder::new())),
                false => sim.add_node(NodeBuilder::Crossing(CrossingBuilder::new())),
            };
        }
    }
    for i in 0..grid_side_len {
        for j in 0..grid_side_len {
            let is_corner =
                (i == 0 || i == grid_side_len - 1) && (j == 0 || j == grid_side_len - 1);
            let is_lower_edge = i == 0 || j == 0;
            let is_higher_edge = i == grid_side_len - 1 || j == grid_side_len - 1;
            let is_right_edge = j == grid_side_len - 1;
            let is_bottom_edge = i == grid_side_len - 1;
            let is_edge = is_lower_edge || is_higher_edge;
            if is_corner {
                continue;
            }
            if is_edge {
                if is_right_edge && (i != 0 && i != grid_side_len - 1) {
                    sim.connect_with_street(
                        ((i * grid_side_len + j) as usize, Direction::W),
                        ((i * grid_side_len + j - 1) as usize, Direction::E),
                        1,
                        street_len
                    )
                    .expect("p1");
                    sim.connect_with_street(
                        ((i * grid_side_len + j - 1) as usize, Direction::E),
                        ((i * grid_side_len + j) as usize, Direction::W),
                        1,
                        street_len
                    )
                    .expect("p2");
                } else if is_bottom_edge && (j != 0 && j != grid_side_len - 1) {
                    sim.connect_with_street(
                        ((i * grid_side_len + j) as usize, Direction::N),
                        (((i - 1) * grid_side_len + j) as usize, Direction::S),
                        1,
                        street_len
                    )
                    .expect("p3");
                    sim.connect_with_street(
                        (((i - 1) * grid_side_len + j) as usize, Direction::S),
                        ((i * grid_side_len + j) as usize, Direction::N),
                        1,
                        street_len
                    )
                    .expect("p4");
                }
            } else {
                sim.connect_with_street(
                    ((i * grid_side_len + j) as usize, Direction::W),
                    ((i * grid_side_len + j - 1) as usize, Direction::E),
                    1,
                        street_len
                )
                .expect("p5");
                sim.connect_with_street(
                    ((i * grid_side_len + j) as usize, Direction::N),
                    (((i - 1) * grid_side_len + j) as usize, Direction::S),
                    1,
                        street_len
                )
                .expect("p6");
                sim.connect_with_street(
                    ((i * grid_side_len + j - 1) as usize, Direction::E),
                    ((i * grid_side_len + j) as usize, Direction::W),
                    1,
                        street_len
                )
                .expect("p7");
                sim.connect_with_street(
                    (((i - 1) * grid_side_len + j) as usize, Direction::S),
                    ((i * grid_side_len + j) as usize, Direction::N),
                    1,
                        street_len
                )
                .expect("p8");
            }
        }
    }
    sim
}
