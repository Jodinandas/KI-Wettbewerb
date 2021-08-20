use super::super::traits::Movable;


/// This structs represents a sidewalk, a street or something else that can be walked on
#[derive(Debug)]
pub struct Traversible<T: Movable> {
    /// These are for example people, cars, bycicles etc.
    movables: Vec<(T, f32)>,
    /// The total length of the traversible
    length: f32,
}

impl<T: Movable> Traversible<T> {
    pub fn new<E: Movable>(length: f32) -> Traversible<E> {
        Traversible {
            movables: Vec::new(),
            length
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
                out.push(
                    self.movables.remove(i).0
                );
            }
        }
        out
    }
    pub fn add(&mut self, movable: T) {
        self.movables.push((movable, 0.0));
    }
}