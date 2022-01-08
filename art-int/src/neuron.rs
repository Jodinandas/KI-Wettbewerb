use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neuron {
    pub bias: f32,
    pub weights: Vec<f32>,
}


impl Neuron {
    pub fn new(bias: f32, weights: Vec<f32>) -> Self {
        assert!(!weights.is_empty());

        Self { bias, weights}
    }

    pub fn random(rng: &mut dyn RngCore, output_neurons: usize) -> Self {
        let bias = rng.gen_range(-1.0..=1.0);

        let weights = (0..output_neurons)
            .map(|_| rng.gen_range(-1.0..=1.0))
            .collect();

        Self::new(bias, weights)
    }

    pub fn from_weights(output_neurons: usize, weights: &mut dyn Iterator<Item = f32>) -> Self {
        let bias = weights.next().expect("got not enough weights");

        let weights = (0..output_neurons)
            .map(|_| weights.next().expect("got not enough weights"))
            .collect();

        Self::new(bias, weights)
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
            let neuron = Neuron::random(&mut rng, 4);

            approx::assert_relative_eq!(neuron.bias, -0.6255188);

            approx::assert_relative_eq!(
                neuron.weights.as_slice(),
                [0.67383957, 0.8181262, 0.26284897, 0.5238807].as_slice(),
            );
        }
    }

    mod from_weights {
        use super::*;

        #[test]
        fn test() {
            let actual = Neuron::from_weights(3, &mut vec![0.1, 0.2, 0.3, 0.4].into_iter());
            let expected = Neuron::new(0.1, vec![0.2, 0.3, 0.4]);

            approx::assert_relative_eq!(actual.bias, expected.bias);
            approx::assert_relative_eq!(actual.weights.as_slice(), expected.weights.as_slice());
        }
    }
}
