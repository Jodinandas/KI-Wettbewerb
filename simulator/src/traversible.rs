use std::{ptr, collections::VecDeque};

use crate::{movable::MovableStatus, node::CostCalcParameters, simulation::calculate_cost, CAR_SPACING};

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
    pub fn update_movables(&mut self, t: f64) -> Vec<usize> {
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
            if is_at_end || (part_of_waiting && (*dist - dist_last) <= CAR_SPACING) {
                part_of_waiting = true;
                movables_waiting += 1;
            } else {
                *dist += t as f32 * m.get_speed();
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

    pub fn remove_movable(&mut self, i: usize) -> T {
        self.movables.remove(i).unwrap().0
    }

    pub fn get_movable_by_index<'a>(&'a self, i: usize) -> &'a T {
        &self.movables[i].0
    }

    pub fn calculate_cost_of_movables(&self, params: &CostCalcParameters) -> f32 {
        self.movables
            .iter()
            .map(|(m, _)| calculate_cost(m.get_report(), params))
            .sum()
    }
}
