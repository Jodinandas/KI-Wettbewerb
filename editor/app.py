import tkinter
import enum
import logging
import itertools

logging.basicConfig(level=logging.DEBUG)


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

class Street:
    def __init__(self):
        self.points = []
        self.point_items = []
        self.arc = None
    
        
class StreetView(tkinter.Canvas):
    def __init__(self, master):
        super().__init__(master, width=800, height=500)
        self.bind("<Button-1>", self.parse_mouse_left)
        self.bind("<Button-2>", self.parse_mouse_right)
        self.selected = Street()
    
    def add_waypoint(self, x, y):
        oval_size = 10 
        return self.create_oval(
            x - oval_size/2, y - oval_size/2,
            x + oval_size/2, y + oval_size/2,
            fill="red"
        )

    
    def parse_mouse_left(self, event):
        if isinstance(self.selected, Street):
            p = self.add_waypoint(event.x, event.y)
            self.selected.points.extend((event.x, event.y))
            self.selected.point_items.append(p)
            if not self.selected.arc:
                self.selected.arc = self.create_line(*self.selected.points)
            else:
                self.coords(self.selected.arc,
                    *self.selected.points
                )
            
            logging.debug("Added waypoint")
    def parse_mouse_right(self, event):
        if isinstance(self.selected, Street):
            pass
        
        
        
    
    