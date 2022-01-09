# AICO²
Submission to the Austrian competition "Ki-Wettbewerb" by Team Rusted.

A Street simulation that uses Neural Networks and a Genetic Algorithm to optimize the Traffic lights. It is possible to optimize the Street simulation either to be more economically friendly or to allow for maximum car speed. This can be changed in an intuitive-to-use Editor that is completely separate from the Backend to allow for maximum efficiency.

## Usage

In order to run this project, the rust programming language needs to be installed. To do so, follow the instructions on https://www.rust-lang.org/learn/get-started 

Afterwards, clone this github repository using
```bash
git clone https://github.com/Jodinandas/KI-Wettbewerb
```
If rust and its dependencies were installed properly (you may need to add it to $PATH), the program can be launched using (Note: compiling may take a while because all dependencies need to be installed first)
```bash
cargo run --release
```
(you need to be in the top-level  of the repository).

# The editor
TODO: Add image

The editor has two modes: the Street Editor mode and the simulation mode.

## Street editor mode

Here, the user can create a custom Street network fitting their needs. Theoretically, the network can be indefinetly large. However, we do not recommend very big sizes, because the optimization gets exponentially more difficult as the AI has to understand the coherence of all nodes. There are 3 NodeTypes available by default: 
* Crossings (standard: 4 Inputs and 4 Outputs)
* Streets (going only in one direction)
* I/O Nodes (the points where cars can enter or leave the Simulation)

### Moving around
Use the "pan" tool are navigate using W,A,S,D or the arrow keys. You can zoom in & out using Q, E or the scroll wheel.

### Adding crossings
Simply click the "add Crossing" tool on the right hand side and click on the canvas, where you want to place it.
### Adding I/O nodes
The same procedure as with Crossings
### Adding streets
When having selected the "add street" tool, they can be added by hovering over a Crossing or I/O Node and selecting the direction by clicking on the corresponding circle.

The output of the street is then determined in a similar manner.

### Changing attributes
By selecting the "select" tool and clicking on any Crossing or I/O Node, you can change their attributes.
* Crossings: The lane count can be determined for each steet. This essentially changes the (car-)capacity of the street.
* I/O Nodes: The spawn rate can be changed. This is the probability of the Node to spawn a new car each iteration. Values of 0.01 and below are recommended.

### Deleting crossings and I/O nodes

Using the "delete" tool, you can delete any Crossing or I/O node. The streets connected to it are automatically deleted.

## Simulation mode
Here, the created node network can be simulated. You can change the following parameters:

* Simulation delay: Useful to watch the movement of the cars. All simulations are waiting this amount of time after each step.
* Optimization target: Choose between optimal speed or optimal environmental impact
+ Time step in seconds: How many seconds in the real world are simulated each step. Low values produce more accurate simulations.
* Population size: The amount of different genomes for the Genentic Algorithm to try each iteration. Values above 500 are recommended
* Iterations to stop: How many iterations to perform before calculating the success of a specific genome
* Mutation chance: The mutation chance for a single Chromosome. Recommended value <0.001
* Mutation coefficient: The coefficent to apply to the randomly generated mutation. Recommended value <0.1
* Disable tracking [...]: Do not show the movables in the frontend. This will greatly improve performance.

After clicking the "Start Simulation" button, the different generational reports can be watched in the "Generation Report" window in the lower part of the window. You will need to resize it first (hover over the edge over "Generation Report" and drag it upwards).

## Preferences
For those of you who like to destroy their eyes, there is also a light mode available. You can change it in the simulation tab

## File dialog
Currently, you can not change the output dir. Files that are loaded or saved are located in the repository root under the name `StreetSimulation.json`.
