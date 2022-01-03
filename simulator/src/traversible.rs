use crate::movable::MovableStatus;

use super::{movable::RandCar, traits::Movable};

/// This structs represents a sidewalk, a street or something else that can be walked on
#[derive(Debug, Clone)]
pub struct Traversible<T = RandCar>
where
    T: Movable,
{
    /// These are for example people, cars, bycicles etc.
    movables: Vec<(T, f32)>,
    /// The total length of the traversible
    length: f32,
}

impl<T: Movable> Traversible<T> {
    /// returns a new traversible with given length
    pub fn new<E: Movable>(length: f32) -> Traversible<E> {
        Traversible {
            movables: Vec::new(),
            length,
        }
    }
    /// update all the movables by timestep `t` and return all that have reached the end
    ///
    /// TODO
    pub fn update_movables(&mut self, t: f64) -> Vec<T> {
        // return all movables that are
        let mut out = Vec::<T>::new();
        for i in 0..self.movables.len() {
            let (m, mut dist) = &self.movables[i];
            dist += t as f32 * m.get_speed();
            if dist >= self.length {
                out.push(self.movables.remove(i).0);
            }
        }
        out
    }

    /// puts a movable on the beginning of the road
    pub fn add(&mut self, movable: T) {
        self.movables.push((movable, 0.0));
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
                position: *t,
                lane_index: 0,
                movable_id: m.get_id()
            })
            .collect()
    }
}
