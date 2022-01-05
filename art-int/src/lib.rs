pub use self::layer_topology::*;

pub use self::{layer::*, neuron::*};
use genetics::IndividualComponent;
use rand::prelude::ThreadRng;
use rand::{Rng, RngCore};
use std::iter::once;

pub mod genetics;
pub mod layer;
mod layer_topology;
pub mod neuron;

#[derive(Clone, Debug)]
pub struct Network {
    pub layers: Vec<Layer>,
}

impl Network {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self { layers }
    }

    pub fn random(rng: &mut dyn RngCore, layers: &[LayerTopology]) -> Self {
        assert!(layers.len() > 1);

        let layers = layers
            .windows(2)
            .map(|layers| Layer::random(rng, layers[0].neurons, layers[1].neurons))
            .collect();

        Self::new(layers)
    }

    pub fn from_weights(layers: &[LayerTopology], weights: impl IntoIterator<Item = f32>) -> Self {
        assert!(layers.len() > 1);

        let mut weights = weights.into_iter();

        let layers = layers
            .windows(2)
            .map(|layers| Layer::from_weights(layers[0].neurons, layers[1].neurons, &mut weights))
            .collect();

        if weights.next().is_some() {
            panic!("got too many weights");
        }

        Self::new(layers)
    }

    pub fn propagate(&self, inputs: Vec<f32>) -> Vec<f32> {
        self.layers
            .iter()
            .fold(inputs, |inputs, layer| layer.propagate(inputs))
    }

    pub fn weights(&self) -> impl Iterator<Item = f32> + '_ {
        self.layers
            .iter()
            .flat_map(|layer| layer.neurons.iter())
            .flat_map(|neuron| once(&neuron.bias).chain(&neuron.weights))
            .cloned()
    }
}


/// Performs crossover on two neurons
fn crossover_neurons(n1: &Neuron, n2: &Neuron, rng: &mut ThreadRng) -> Neuron {
    let output_neurons = n1.weights.len();
    // the first element is ALWAYS the bias of the neuron
    let mut bias_and_weights_iterator = once(&n1.bias)
        .chain(n1.weights.iter())
        .zip(once(&n2.bias)
                .chain(n2.weights.iter())
        ).map( | (w_or_b_left, w_or_b_right) | {
            match rng.gen_bool(0.5) {
                true => *w_or_b_left,
                false => *w_or_b_right,
            }
        });
    Neuron::from_weights(output_neurons, &mut bias_and_weights_iterator)
}

impl IndividualComponent for Network {
    fn crossover(&self, other: Self, rng: &mut ThreadRng) -> Self {
        // operate on two layers in the same position at the same time
        let new_layers = self.layers.iter().zip(other.layers.iter()).map(| (this_layer, other_layer) | {
            // operate on two neurons in the same position at the same time
            Layer::new(
                this_layer.neurons.iter().zip(other_layer.neurons.iter()).map(
                    | (this_neuron, other_neuron) | crossover_neurons(this_neuron, other_neuron, rng)
                ).collect()
            )
        }).collect::<Vec<Layer>>();
        Network::new(new_layers)
    }

    fn mutate(&mut self, coeff: f32, rng: &mut ThreadRng) {
        // for each layer
        self.layers.iter_mut().for_each(| layer | {
            // for each neuron
            layer.neurons.iter_mut().for_each( | neuron | {
                // for each weight and bias (bias is the first value)
                once(&mut neuron.bias).chain(neuron.weights.iter_mut()).for_each( | w_or_b  | {
                    let sign = match rng.gen_bool(0.5) {
                        true => -1.0,
                        false => 1.0,
                    };
                    *w_or_b += sign * coeff * rng.gen::<f32>();
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod random {
        use super::*;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        #[test]
        fn test() {
            let mut rng = ChaCha8Rng::from_seed(Default::default());

            let network = Network::random(
                &mut rng,
                &[
                    LayerTopology { neurons: 3 },
                    LayerTopology { neurons: 2 },
                    LayerTopology { neurons: 1 },
                ],
            );

            assert_eq!(network.layers.len(), 2);
            assert_eq!(network.layers[0].neurons.len(), 2);

            approx::assert_relative_eq!(network.layers[0].neurons[0].bias, -0.6255188);

            approx::assert_relative_eq!(
                network.layers[0].neurons[0].weights.as_slice(),
                &[0.67383957, 0.8181262, 0.26284897].as_slice()
            );

            approx::assert_relative_eq!(network.layers[0].neurons[1].bias, 0.5238807);

            approx::assert_relative_eq!(
                network.layers[0].neurons[1].weights.as_slice(),
                &[-0.5351684, 0.069369555, -0.7648182].as_slice()
            );

            assert_eq!(network.layers[1].neurons.len(), 1);

            approx::assert_relative_eq!(
                network.layers[1].neurons[0].weights.as_slice(),
                &[-0.48879623, -0.19277143].as_slice()
            );
        }
    }

    mod from_weights {
        use super::*;

        #[test]
        fn test() {
            let layers = &[LayerTopology { neurons: 3 }, LayerTopology { neurons: 2 }];
            let weights = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];

            let actual: Vec<_> = Network::from_weights(layers, weights.clone())
                .weights()
                .collect();

            approx::assert_relative_eq!(actual.as_slice(), weights.as_slice());
        }
    }

    mod propagate {
        use super::*;

        #[test]
        fn test() {
            let layers = (
                Layer::new(vec![
                    Neuron::new(0.0, vec![-0.5, -0.4, -0.3]),
                    Neuron::new(0.0, vec![-0.2, -0.1, 0.0]),
                ]),
                Layer::new(vec![Neuron::new(0.0, vec![-0.5, 0.5])]),
            );
            let network = Network::new(vec![layers.0.clone(), layers.1.clone()]);

            let actual = network.propagate(vec![0.5, 0.6, 0.7]);
            let expected = layers.1.propagate(layers.0.propagate(vec![0.5, 0.6, 0.7]));

            approx::assert_relative_eq!(actual.as_slice(), expected.as_slice());
        }
    }

    mod weights {
        use super::*;

        #[test]
        fn test() {
            let network = Network::new(vec![
                Layer::new(vec![Neuron::new(0.1, vec![0.2, 0.3, 0.4])]),
                Layer::new(vec![Neuron::new(0.5, vec![0.6, 0.7, 0.8])]),
            ]);

            let actual: Vec<_> = network.weights().collect();
            let expected = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];

            approx::assert_relative_eq!(actual.as_slice(), expected.as_slice());
        }
    }
}