UniformCrossover: 50% Chance of either weight
GaussianMutation: `chance` Wahrscheinlichkeit, dass zu dem Gen ein zufälliger (Wert zwischen -1 und 1) * `coeff` hinzuaddiert wird
Im Beispiel: `chance`: 0.01, `coeff`: 0.3

# Evolution
* Zwei Eltern werden zufällig anhand ihrer Fitness ausgewählt (Funktion choose_weighted von SliceRandom)
* UniformCrossover der beiden Eltern
* GaussianMutation