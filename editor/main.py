from app import *

logging.basicConfig(level=logging.DEBUG,
                    format='%(asctime)s (%(levelname)-2s) %(message)s',
                    datefmt='[%d.%m.%Y %H:%M:%S]')

application = App()
application.mainloop()


"""
How will this editor be structured?
There should be three main components:
    - ToolBox: The toolbox provides different tools to modify both the
        StreetView and in the background the actual, non graphical Street
        Representation
    - StreetView: Should draw the street network and handle removing/editing different
        parts of the street. IMPORTANT/TODO: The Streetview should later on be able
        to visualise a simulation using callbacks. Maybe even callbacks from Rust.
        (As the simulator will probably be written in Rust for huge performance benefits)
    - Street: The actual Street object, storing information about the street in a non graphical
        manner, holding less information than the full graphical view. It should then be easy to
        later on convert this for use in a genetic algorithm

Different (possibly better) approach:
    Scrap the idea of a non graphical representation, making programming easier.
    Then, when preparing the data for the genetic algorithm, just convert the
    Data to a suitable form, deleting everything unnecessary. This would remove
    the complexity of updating the non graphical representation whenever the graphical
    one changes.
"""