use std::{ptr, collections::{VecDeque, HashMap}};

use crate::{movable::MovableStatus, node::CostCalcParameters, simulation::calculate_cost, CAR_SPACING, node_builder::Direction};

use super::{movable::RandCar, traits::Movable};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

/// This structs represents a sidewalk, a street or something else that can be walked on
#[derive(Debug, Clone)]
pub struct Traversible<T = RandCar>
where
    T: Movable,
{
    /// These are for example people, cars, bycicles etc.
    movables: VecDeque<(T, f32)>,
    /// The total length of the traversible
    length: f32,
    /// the number of movables that are waiting at the end to go on a crossing
    movables_waiting: u32,
}

impl<T: Movable> Traversible<T> {
    /// returns a new traversible with given length
    pub fn new<E: Movable>(length: f32) -> Traversible<E> {
        Traversible {
            movables: VecDeque::new(),
            length,
            movables_waiting: 0,
        }
    }
    /// update all the movables by timestep `t` and return the index of all that have reached the end
    pub fn update_movables(&mut self, t: f32) -> Vec<usize> {
        // let mut out = Vec::<&mut T>::new();
        // for i in 0..self.movables.len() {
        let mut out = Vec::new();
        let l = self.length;
        let mut part_of_waiting = false;
        let mut dist_last = 0.0;
        let mut movables_waiting = 0;
        self.movables.iter_mut().enumerate().rev().for_each( | (i, (m, dist)) | {
            let is_at_end= *dist >= l;
            if is_at_end {
                out.push(i)
            }
            m.update(t);
            let speed = m.get_speed();
            let pos_delta = t as f32 * (speed[1] - speed[0])*0.3;
            m.set_current_speed((speed[1] - speed[0])*0.3);
            m.add_to_dist(pos_delta);
            if is_at_end || (part_of_waiting && (dist_last - (*dist + pos_delta)) <= CAR_SPACING) {
                part_of_waiting = true;
                movables_waiting += 1;
            } else {
                *dist += pos_delta;
                part_of_waiting = false;
            }
            dist_last = *dist;
        });
        self.movables_waiting = movables_waiting;
        // for i in 0..self.movables.len() {
        //     let (m, dist) = &mut self.movables[i];
        //     *dist += t as f32 * m.get_speed();
        //     if *dist >= l {
        //         self.movables_waiting += 1;
        //         out.push(i);
        //     }
        // }
        out
    }
    /// returns the number of movables that are waiting to go on a crossing
    pub fn num_movables_waiting(&self) -> u32 {
        self.movables_waiting
    }
    /// 
    pub fn get_overnext_node_ids(&self) -> HashMap<usize, u32> {
        let mut map = HashMap::new();
        self.movables.iter().rev().take(self.movables_waiting as usize).for_each(| (car, _pos ) | {
            let id = car.overnext_node_id().unwrap();
            *map.entry(id).or_insert(0) += 1;
        });
        map
    }
    /// removes a movable using a reference to it. This can be useful for
    /// removing cars lazily and checking conditions outside the traversible
    /// before removing it
    pub fn rm_movable_by_ref(&mut self, movable: &T) -> Result<T, &'static str> {
        let index = match self
            .movables
            .iter()
            .enumerate()
            .find(|(_i, (m, _p))| ptr::eq(movable, m))
        {
            Some((i, _)) => i,
            None => return Err("Invalid reference passed to rm_movable_by_ref"),
        };
        if self.movables_waiting > 0 {
            self.movables_waiting -= 1;
        }
        Ok(self.movables.remove(index).unwrap().0)
    }

    /// puts a movable on the beginning of the road
    pub fn add(&mut self, movable: T) {
        self.movables.push_front((movable, 0.0));
    }

    /// returns the number of movables on the traversible
    pub fn num_movables(&self) -> usize {
        self.movables.len()
    }
    /// generates a status object for all of the movables on the
    /// traversable. All lane indices are set to 0
    pub fn get_movable_status(&self) -> Vec<MovableStatus> {
        self.movables
            .iter()
            .map(|(m, t)| MovableStatus {
                position: t.min(self.length) / self.length,
                lane_index: 0,
                movable_id: m.get_id(),
                delete: false,
            })
            .collect()
    }
    ///
    pub fn get_target_id_of_movable_at_end(&self) -> Option<usize> {
        if let Some(movable) = self.movables.back() {
            if self.movables_waiting == 0 {
                return None
            }
            return movable.0.overnext_node_id()
        }
        None
    }

    pub fn remove_movable(&mut self, i: usize) -> T {
        self.movables.remove(i).unwrap().0
    }

    pub fn get_movable_by_index<'a>(&'a self, i: usize) -> &'a T {
        &self.movables[i].0
    }

    pub fn calculate_cost_of_movables(&self, params: &CostCalcParameters) -> [f64; 2] {
        self.movables
            .iter()
            .fold([0.0, 0.0], | [cost, co2], (mr, _) | {
                let [ncost, nco2] = calculate_cost(mr.get_report(), params);
                [
                    cost + ncost,
                    co2 + nco2
                ]
            })
    }

    pub fn reset(&mut self) -> Vec<MovableStatus> {
        let to_return = self.movables.iter().map(| (m, _dist) | {
            MovableStatus {
                position: 0.0,
                lane_index: 0,
                movable_id: m.get_id(),
                delete: true,
            }
        }).collect();
        self.movables = VecDeque::new();
        self.movables_waiting = 0;
        to_return
    }
}
