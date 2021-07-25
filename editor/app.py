import tkinter
import enum
import logging
import itertools
import math
from input_parser import InputParser
from street_data import *

logging.basicConfig(level=logging.DEBUG,
                    format='%(asctime)s (%(levelname)-2s) %(message)s',
                    datefmt='[%d.%m.%Y %H:%M:%S]')


class App(tkinter.Tk):
    def __init__(self):
        super().__init__()
        # create widgets
        # TODO: make canvas size dynamic
        self.canvas = StreetView(self)
        self.toolbox = Toolbox(self)
        
        for i in range(5):
            placeholder = tkinter.Button(self.toolbox, text=str(i), font=("Computer Modern", 20))
            placeholder.grid(column=0, row=i, sticky="W")
        
        # place widgets
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(0, weight=1)
        self.canvas.grid(row=0, column=0)
        self.toolbox.grid(row=0, column=1, sticky="W")
    

    
class Toolbox(tkinter.Frame):
    """A toolbox to store things like making a new road"""
    def __init__(self, master):
        super().__init__(master)


        
class StreetView(tkinter.Canvas):
    def __init__(self, master):
        super().__init__(master, width=800, height=500)
        self.street_data = StreetData()
        self.street_data.streets.append(Street())
        self.input_parser = InputParser(self.street_data)

        # Register events
        self.input_parser.add_street_segment += self.add_waypoint
        self.input_parser.add_street_segment += self.expand_street_arc
    
        self.bind("<Button-1>", self.input_parser.parse_mouse_left)
        self.bind("<Button-2>", self.input_parser.parse_mouse_right)
    def add_waypoint(self, street, x, y):
        oval_size = 10 
        street.points.extend((x, y))
        street.point_items.append(self.create_oval(
            x - oval_size/2, y - oval_size/2,
            x + oval_size/2, y + oval_size/2,
            fill="red"
        ))
    def expand_street_arc(self, street, x, y):
        if not len(street.points) >= 4: return

        if not street.arc:
            street.arc = self.create_line(*street.points)
        else:
            self.coords(street.arc,
                *street.points
            )

        
        
        
    
    