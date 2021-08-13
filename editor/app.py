import tkinter
import enum
import logging
import itertools
import math
from input_parser import InputParser
from street_data import *
from toolbar import Toolbar
from itemeditor import ItemEditor



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
        self.input_parser.delete_crossing += self.street_view.street_data.delete
        def connect_streets(c1, c2):
            c1.connect_both_ways(c2, 1)
            print("connected streets", c1._connected, c2._connected)

        # Export functionality
        self.toolbox.on_export += self.street_view.street_data.export_to_json
        self.input_parser.add_street += connect_streets
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
        self.street_data.draw_crossing += self.draw_crossing
        self.street_data.delete_crossing += self.delete_crossing
        self.street_data.draw_street += self.draw_street
        self.street_data.delete_street += self.delete_street

        # A dictionary that hold everything that is drawn on the screen
        # is used to later delete/redraw streets, crossings etc. 
        # crossings are used as keys themselves
        # streets are saved in the format (c1, c2)
        self._graphics_objects = {}
    
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
        color = "green" if crossing.is_io_node else "red" 
        if crossing in self._graphics_objects:
            self.coords(self._graphics_objects[crossing],
                x - oval_size/2, y - oval_size/2,
                x + oval_size/2, y + oval_size/2,
            )
            self.itemconfig(self._graphics_objects[crossing],
                fill=color)
        else:
            self._graphics_objects[crossing] = self.create_oval(
                x - oval_size/2, y - oval_size/2,
                x + oval_size/2, y + oval_size/2,
                fill= color            )
    
    def delete_crossing(self, crossing):
        print("Deleting crossing: ", crossing)
        g_obj = self._graphics_objects.pop(crossing)
        self.delete(g_obj)
    
    def draw_street(self, c1, c2, lanes):
        # Ignore lanes for now
        if (c1, c2) in self._graphics_objects:
            line = self._graphics_objects[(c1, c2)]
            self.coords(line, *c1.position, *c2.position)
        else:
            line = self.create_line(*c1.position, *c2.position)
            self._graphics_objects[(c1, c2)] = line
    
    def delete_street(self, c1, c2):
        """Deletes a street c1 -> c2. 
        
        Be careful! Only works in direction c1 -> c2"""
        
        print("Delete: ", (c1, c2))

        if (c1, c2) in self._graphics_objects:
            g_obj = self._graphics_objects.pop((c1, c2))
            self.delete(g_obj)
    


        
        
    
    