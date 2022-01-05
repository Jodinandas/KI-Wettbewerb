use rand::prelude::ThreadRng;

/// Because each crossing in our Simulation has a different NN,
/// we have to perform the crossover for each nn seperatly
/// (We do not want to try to train one perfect Crossing, but rather a perfect Crossing
///     in *that specififc* position in the street network. This should enable different Crossings to learn to interact)
pub trait IndividualComponent {
    /// UniformCrossover: 50% Chance of either weight
    fn crossover(&self, other: Self, rng: &mut ThreadRng) -> Self;
    /// GaussianMutation: a random value is added to this gene (value between -1 and 1) * `coeff`
    ///
    /// (The decicion, if the Individual should be mutated at all *is not* part of this function)
    fn mutate(&mut self, coeff: f32, rng: &mut ThreadRng); 
}
 