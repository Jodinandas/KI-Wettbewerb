# KI-Wettbewerb

## Ideas
Have dedicated "ports" for streets in each crossing per direction, like with blender nodes

## TODO/Design Goals

### Frontend
* Functions (nodes = IONode, Crossing, Street):
    ~~* Pan around
        * Mouse
        * Keyboard~~
    * add Nodes
    * delete nodes
    * change nodes
        * Streets: lanes, length (?) -> very careful (?)
        * IONodes: frequency
        * Crossing
* Modes
    * Standard create-Street-constellation
    * Simulation (lock nodes)
        * show one thread simulating
        * Diagram id & score
        * 
> The simulation progress of all simulations should be tracked in some way. Either using a proper progress report (With % finished, fitness etc.), or just a        signal at the end of the simulation. If all simulations apart from the displayed one have finished (The main one will be slower, because it hast to draw the simulation in addition to simulating it), add a button to **maybe** (depending on how hard this feature would be to implement) abort the displayed simulation and continue with all others or to stop displaying the simulation. If one generation takes a long time to simulate, this won't do much.

### Backend
#### Must-Have
* Ampelschaltung
* GA, preferably multithread
* Jede Straße 1 IONode (Auch in Editor angezeigt, würde ich sagen [Jonas])
* Fußgänger

#### Optional
* Do not use optimal path for cars each time, because more realistic 
