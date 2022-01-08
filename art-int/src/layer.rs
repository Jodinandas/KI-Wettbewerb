use crate::*;

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActivationFunc {
    ReLu,
    SoftMax,
}

impl ActivationFunc {
    pub fn propagate(&self, neurons: &[Neuron], mut inputs: Vec<f32>) -> Vec<f32> {
        match self {
            ActivationFunc::ReLu => {
                neurons.iter().map( | n | {
                    let output = inputs
                        .iter()
                        .zip(&n.weights)
                        .map(|(input, weight)| input * weight)
                        .sum::<f32>();

                    (n.bias + output).max(0.0)
                }).collect()
            },
            ActivationFunc::SoftMax =>  {
                let mut sum: f32 = 0.0;
                inputs.iter_mut().for_each( | value | {
                    *value = value.exp();
                    sum += *value;
                });
                inputs.iter_mut().for_each( | value | {
                    *value /= sum;
                });

                inputs
            },
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub neurons: Vec<Neuron>,
    pub activation: ActivationFunc
}

impl Layer {
    pub fn new(neurons: Vec<Neuron>, activation: ActivationFunc) -> Self {
        if activation != ActivationFunc::SoftMax {
            assert!(!neurons.is_empty());
        }

        assert!(neurons
            .iter()
            .all(|neuron| neuron.weights.len() == neurons[0].weights.len()));

        Self { neurons, activation}
    }

    pub fn from_weights(
        input_size: usize,
        output_size: usize,
        weights: &mut dyn Iterator<Item = f32>,
        activation: ActivationFunc
    ) -> Self {
        let neurons = (0..output_size)
            .map(|_| Neuron::from_weights(input_size, weights))
            .collect();

        Self::new(neurons, activation)
    }

    pub fn random(rng: &mut dyn RngCore, input_neurons: usize, output_neurons: usize, activation: ActivationFunc) -> Self {
        let neurons = (0..output_neurons)
            .map(|_| Neuron::random(rng, input_neurons))
            .collect();

        Self::new(neurons, activation)
    }

    pub fn propagate(&self, inputs: Vec<f32>) -> Vec<f32> {
        self.activation.propagate(&self.neurons, inputs)
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
            let layer = Layer::random(&mut rng, 3, 2, ActivationFunc::ReLu);

            let actual_biases: Vec<_> = layer.neurons.iter().map(|neuron| neuron.bias).collect();
            let expected_biases = vec![-0.6255188, 0.5238807];

            let actual_weights: Vec<_> = layer
                .neurons
                .iter()
                .map(|neuron| neuron.weights.as_slice())
                .collect();
            let expected_weights: Vec<&[f32]> = vec![
                &[0.67383957, 0.8181262, 0.26284897],
                &[-0.53516835, 0.069369674, -0.7648182],
            ];

            approx::assert_relative_eq!(actual_biases.as_slice(), expected_biases.as_slice());
            approx::assert_relative_eq!(actual_weights.as_slice(), expected_weights.as_slice());
        }
    }


    mod from_weights {
        use super::*;

        #[test]
        fn test() {
            let layer = Layer::from_weights(
                3,
                2,
                &mut vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8].into_iter(),
                ActivationFunc::ReLu
            );

            let actual_biases: Vec<_> = layer.neurons.iter().map(|neuron| neuron.bias).collect();
            let expected_biases = vec![0.1, 0.5];

            let actual_weights: Vec<_> = layer
                .neurons
                .iter()
                .map(|neuron| neuron.weights.as_slice())
                .collect();
            let expected_weights: Vec<&[f32]> = vec![&[0.2, 0.3, 0.4], &[0.6, 0.7, 0.8]];

            approx::assert_relative_eq!(actual_biases.as_slice(), expected_biases.as_slice());
            approx::assert_relative_eq!(actual_weights.as_slice(), expected_weights.as_slice());
        }
    }
}
