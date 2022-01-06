use crate::ActivationFunc;

#[derive(Clone, Copy, Debug)]
pub struct LayerTopology {
    pub neurons: usize,
    pub activation: ActivationFunc
}

impl LayerTopology {
    pub fn with_activation(mut self, activation: ActivationFunc) -> Self {
        self.activation = activation;
        self
    }
    pub fn new(neurons: usize) -> LayerTopology {
        LayerTopology {
            neurons,
            activation: ActivationFunc::ReLu
        }
    }
}
