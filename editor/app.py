import tkinter
import enum
import logging
import itertools
import math
from input_parser import InputParser
from street_data import *
from toolbar import Toolbar

logging.basicConfig(level=logging.DEBUG,
                    format='%(asctime)s (%(levelname)-2s) %(message)s',
                    datefmt='[%d.%m.%Y %H:%M:%S]')


class App(tkinter.Tk):
    def __init__(self):
        super().__init__()
        # create widgets
        # TODO: make canvas size dynamic
        self.street_view = StreetView(self)
        self.toolbox = Toolbar(self)
        
        # place widgets
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(0, weight=1)
        self.street_view.grid(row=0, column=0)
        self.toolbox.grid(row=0, column=1, sticky="W")
        self.input_parser = InputParser(self.street_view.street_data)

        # Register events
        self.input_parser.add_street_segment += self.street_view.add_waypoint
        self.input_parser.add_street_segment += self.street_view.expand_street_arc
        self.toolbox.tool_changed += self.input_parser.on_tool_change
    
        self.street_view.bind("<Button-1>", self.input_parser.parse_mouse_left)
        self.street_view.bind("<Button-2>", self.input_parser.parse_mouse_right)

class StreetView(tkinter.Canvas):
    def __init__(self, master):
        super().__init__(master, width=800, height=500)
        self.street_data = StreetData()
        self.street_data.streets.append(Street())
    
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

        
        
        
    
    