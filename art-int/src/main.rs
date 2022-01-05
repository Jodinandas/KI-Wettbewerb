use art_int::{Layer, LayerTopology, Network, Neuron};
// Dieser Teil wurde von Patryk Wyochowaniec implementiert und unter der MIT Licence auf github herausgegeben: https://github.com/Patryk27/shorelark
// Copyright (c) 2020-2021, Patryk Wychowaniec pwychowaniec@pm.me

fn main() {
    // explaination: weights are the weights of the "incoming" data, not "out"
    let n = Network::new(vec![
        Layer::new(vec![
            Neuron::new(0.0, vec![0.1, 0.1, 0.1]),
            Neuron::new(0.0, vec![0.5, 0.5, 0.5]),
        ]),
        Layer::new(vec![Neuron::new(1.0, vec![0.2, 0.2])]),
    ]);
    for layer in &n.layers {
        println!("{:?}", layer.neurons);
    }
    let res = n.propagate(vec![1.0, 1.0, 1.0]);
    println!("{:?}", res);
}
