# KI-Wettbewerb

## TODO:

* Look at different crates for machine learning
    * for genetic algorithm: [darwin-rs](https://rustrepo.com/repo/willi-kappler-darwin-rs-rust-machine-learning)
* Decide wether to use Reinforcement Learning or Genetic Evolution
    * arguments for RL: 
        - more complex simulation possible
    * arguments for GA:
        - can be easily executed in parallel
        - easier to implement?
* Implement speed limit and length of streets in frontend

## TODO (Frontend: bevy + egui)
* Implement basic canvas-ish in the middle to display output
    from bevy
* Add egui library and build the Buttons etc.
    * Look at how to implement 2-way coupling. *I think*, egui redraws
        the user interface everytime something changes, so two way coupling
        boils down to the question how to redraw the UI when some variable
        in the background changes. (I also don't know how to call a callback
        when a variable changes, but I guess it will be **way easier** compared
        to the older Python frontend.)
    * => Two way coupling is maybe not THAT important, as the only way to change
        the internal state should be through the editor anyways
* Should have a locked simulation mode, where it is impossible to change anything. (To
    prevent the one simulation that is currently displayed to go out of sync, while
    the other simulations in the generation (If it is a genetic algorithm) continue simulating)
* The simulation progress of all simulations should be tracked in some way. Either using a proper progress
    report (With % finished, fitness etc.), or just a signal at the end of the simulation.
    * If all simulations apart from the displayed one have finished (The main one will be slower,
        because it hast to draw the simulation in addition to simulating it), add a button to **maybe** (depending
        on how hard this feature would be to implement) abort the displayed simulation and continue with all others
        or to stop displaying the simulation. If one generation takes a long time to simulate, this won't do much.
