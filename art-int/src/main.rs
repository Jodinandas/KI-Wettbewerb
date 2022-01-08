use art_int::{Layer, LayerTopology, Network, Neuron};
// This code was orginally developed by Patryk Wyochowaniec and released under the MIT Licence on github: https://github.com/Patryk27/shorelark
// Copyright (c) 2020-2021, Patryk Wychowaniec pwychowaniec@pm.me
// It was heavily modified by us (We added Support for different activation functions and implemeted crossover and mutation directly on the
// NN instead of having seperate files)

fn main() {
    // explaination: weights are the weights of the "incoming" data, not "out"
    // let n = Network::new(vec![
    //     Layer::new(vec![
    //         Neuron::new(0.0, vec![0.1, 0.1, 0.1]),
    //         Neuron::new(0.0, vec![0., 0.5, 0.5]),
    //     ], art_int::ActivationFunc::ReLu),
    //     Layer::new(vec![Neuron::new(1.0, vec![0.2, 0.2]), Neuron::new(1.0, vec![0.1, 0.3])], art_int::ActivationFunc::ReLu),
    //     Layer::new(vec![], art_int::ActivationFunc::SoftMax)
    // ]);
    let n = Network::new(vec![
        Layer::new(vec![
            Neuron::new(0.0, vec![0.5, 2.0]),
            Neuron::new(0.0, vec![1.0, 0.5]),
        ], art_int::ActivationFunc::ReLu),
        Layer::new(vec![], art_int::ActivationFunc::SoftMax)
    ]);
    for layer in &n.layers {
        println!("{:?}", layer.neurons);
    }
    let res = n.propagate(vec![1.0, 1.0, 1.0]);
    println!("{:?}", res);
}
