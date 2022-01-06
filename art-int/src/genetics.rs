use rand::{prelude::ThreadRng, Rng};

use crate::Network;

/// Because each crossing in our Simulation has a different NN,
/// we have to perform the crossover for each nn seperatly
/// (We do not want to try to train one perfect Crossing, but rather a perfect Crossing
///     in *that specififc* position in the street network. This should enable different Crossings to learn to interact)
pub trait IndividualComponent {
    /// UniformCrossover: 50% Chance of either weight
    fn crossover(&self, other: &Self, rng: &mut ThreadRng) -> Self;
    /// GaussianMutation: a random value is added to this gene (value between -1 and 1) * `coeff`
    ///
    /// (The decicion, if the Individual should be mutated at all *is not* part of this function)
    fn mutate(&mut self, coeff: f32, rng: &mut ThreadRng); 
}

pub fn crossover_sim_nns(sim_a: &Vec<Network>, sim_b: &Vec<Network>, rng: &mut ThreadRng) -> Vec<Network> {
    sim_a.iter().zip(sim_b.iter()).map( | (nn_a, nn_b) | {
        nn_a.crossover(nn_b, rng)
    }).collect()
}

/// Applies mutation with a chance
pub fn mutate_sim_nns(rng: &mut ThreadRng, sim: &mut Vec<Network>, chance: f32, coeff: f32) {
    if rng.gen_bool(chance.into()) {
        sim.iter_mut().for_each(| nn | {
            nn.mutate(coeff, rng);
        });
    }
}