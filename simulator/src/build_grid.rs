use super::simple::node_builder::{CrossingBuilder, IONodeBuilder, StreetBuilder};
use super::simple::simulation_builder::SimulatorBuilder;
use crate::boxed_node;
use crate::simple::node_builder::{Direction, NodeBuilder};

/// Builds a grid with side length `grid_side_len`
/// The edges are IONodes, the crossings and IONodes
/// are connected to neighbours with streets
pub fn build_grid_sim(grid_side_len: u32) -> SimulatorBuilder {
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
    let mut sim = SimulatorBuilder::new();
    sim.delay(0).max_iter(Some(10000));
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
                    )
                    .unwrap();
                    sim.connect_with_street(
                        ((i * grid_side_len + j - 1) as usize, Direction::E),
                        ((i * grid_side_len + j) as usize, Direction::W),
                        1,
                    )
                    .unwrap();
                } else if is_bottom_edge && (j != 0 && j != grid_side_len - 1) {
                    sim.connect_with_street(
                        ((i * grid_side_len + j) as usize, Direction::S),
                        (((i - 1) * grid_side_len + j) as usize, Direction::N),
                        1,
                    )
                    .unwrap();
                    sim.connect_with_street(
                        (((i - 1) * grid_side_len + j) as usize, Direction::N),
                        ((grid_side_len + j) as usize, Direction::S),
                        1,
                    )
                    .unwrap();
                }
            } else {
                sim.connect_with_street(
                    ((i * grid_side_len + j) as usize, Direction::W),
                    ((i * grid_side_len + j - 1) as usize, Direction::E),
                    1,
                )
                .unwrap();
                sim.connect_with_street(
                    ((i * grid_side_len + j) as usize, Direction::N),
                    (((i - 1) * grid_side_len + j) as usize, Direction::S),
                    1,
                )
                .unwrap();
                sim.connect_with_street(
                    ((i * grid_side_len + j - 1) as usize, Direction::E),
                    ((i * grid_side_len + j) as usize, Direction::W),
                    1,
                )
                .unwrap();
                sim.connect_with_street(
                    (((i - 1) * grid_side_len + j) as usize, Direction::S),
                    ((grid_side_len + j) as usize, Direction::N),
                    1,
                )
                .unwrap();
            }
        }
    }
    sim
}
