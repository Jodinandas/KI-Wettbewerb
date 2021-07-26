import tkinter
import enum
import logging
import itertools
import math
from input_parser import InputParser
from street_data import *
from toolbar import Toolbar
from itemeditor import ItemEditor

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
        self.item_editor = ItemEditor(self, name="ItemEditor")
        
        # place widgets
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)
        self.item_editor.grid(row=0, column=0)
        self.street_view.grid(row=0, column=1)
        self.toolbox.grid(row=0, column=2, sticky="W")
        self.input_parser = InputParser(self.street_view.street_data)

        # Register events
        self.input_parser.add_crossing += self.street_view.on_new_crossing
        self.input_parser.add_street += self.street_view.draw_connection_both_ways
        self.input_parser.add_street += lambda c1, c2: c1.connect_both_ways(c2, 1)
        # self.input_parser.add_crossing += self.street_view.expand_street_arc
        self.input_parser.select_crossing += lambda crossing: self.item_editor.display(crossing)
        self.input_parser.unselect_crossing += lambda *_: self.item_editor.clear()
        self.toolbox.tool_changed += self.input_parser.on_tool_change
    
        self.street_view.bind("<Button-1>", self.input_parser.parse_mouse_left)
        self.street_view.bind("<Button-2>", self.input_parser.parse_mouse_right)
        self.street_view.bind("<Motion>", self.input_parser.on_mouse_move)
        self.street_view.bind("<ButtonRelease-1>", self.input_parser.on_left_release)

class StreetView(tkinter.Canvas):
    def __init__(self, master):
        super().__init__(master, width=800, height=500)
        self.street_data = StreetData()
        self.street_data.on_pos_change += self.draw_crossing
        self.street_data.on_pos_change += self.connection_update
    
    def expand_street_arc(self, street, x, y):
        if not len(street.points) >= 4: return

        if not street.arc:
            street.arc = self.create_line(*street.points)
        else:
            self.coords(street.arc,
                *street.points
            )
    
    def on_new_crossing(self, c):
        self.draw_crossing(c)
        self.street_data.add(c)
        return c
        

    def draw_crossing(self, crossing):
        x, y = crossing.position
        oval_size = 10 
        if crossing.graphic_object:
            self.coords(crossing.graphic_object,
                x - oval_size/2, y - oval_size/2,
                x + oval_size/2, y + oval_size/2,
            )
        else:
            crossing.graphic_object = self.create_oval(
                x - oval_size/2, y - oval_size/2,
                x + oval_size/2, y + oval_size/2,
                fill="red"
            )
    
    def connection_update(self, crossing):
        for c in self.street_data.crossings:
            self.draw_connection(c, crossing, _update=True)
        for street in crossing.street_connections:
            self.draw_connection(crossing, street, _update=True)

    def draw_connection_both_ways(self, c1, c2):
        # TODO: Add distinction between roads with different lane number
        if c1 in c2.street_connections:
            self.draw_connection(c2, c1)
        elif c2 in c1.street_connections:
            self.draw_connection(c1, c2)
        else:
            self.draw_connection(c1, c2)

    def draw_connection(self, c1: Crossing, c2: Crossing, _update=False):
        if c2 in c1.street_connections:
            line = c1.street_connections[c2]  
            self.coords(line, *c1.position, *c2.position)
        else:
            if not _update:
                line = self.create_line(*c1.position, *c2.position)
                c1.street_connections[c2] = line


        
        
    
    