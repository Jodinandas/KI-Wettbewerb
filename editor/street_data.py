from __future__ import annotations
import logging
import math
from editable import EditableList, Editable
from tkinter import StringVar, IntVar, DoubleVar, BooleanVar
import event
import json
from tkinter import filedialog


class Crossing(Editable):
    def __init__(self, position: list, connected: list=[], traffic_lights: bool=False, on_pos_change=None, draw_street=None):
        """class for saving of crossings. connected has to be a
        list in the form of [[crossing, number_of_lanes], [...]]
        """
        Editable.__init__(self)
        self._position = EditableList(IntVar(value=position[0]), IntVar(value=position[1]))
        self.on_pos_change = on_pos_change
        self.draw_street = draw_street 
        self.draw_crossing = None
        self.delete_street = None

        self._position[0].trace("w", lambda *_: self.on_pos_change.notify(self))
        self._position[1].trace("w", lambda *_: self.on_pos_change.notify(self))
        # TODO: Create setter and getter
        self._connected = EditableList()

        self._is_io_node = BooleanVar(value=False)
        # redraw the node if it becomes an IO node
        self._is_io_node.trace("w", lambda *_: self.draw_crossing.notify(self))

        for c, n in connected:
            self._connected.append([c, IntVar(n)])
        self._traffic_lights = BooleanVar(value=traffic_lights)
        
        # Mark for the editor
        self.mark_editable(self._position, name="position: ", range_=(0, 1500))
        self.mark_editable(self._connected, name="connected: ", range_=(1, 5))
        self.mark_editable(self._traffic_lights, name="has traffic lights: ")
        self.mark_editable(self._is_io_node, name="is I/O-Node")

    @property
    def position(self):
        return [self._position[0].get(), self._position[1].get()]
    
    @property
    def is_io_node(self):
        return self._is_io_node.get()

    @is_io_node.setter
    def is_io_node(self, n: bool):
        assert isinstance(n, bool)
        if self.draw_street:
            self.draw_street.notify(self)
        self._is_io_node.set(n)

    @position.setter
    def position(self, new: list):
        for i, pos in enumerate(self._position):
            pos.set(new[i])

    @property
    def traffic_lights(self):
        return self._traffic_lights.get()

    @traffic_lights.setter
    def traffic_lights(self, new: bool):
        self._traffic_lights.set(new)


    def connect(self, other: Crossing, lanes: int):
        """Connects two Crossings, but only one way. if exists, adds lanes"""
        if not self.is_connected(other):
            self._connected.append(EditableList(other, IntVar(value=lanes)))
        else:
            for c, n in self._connected:
                if c == other:
                    n.set(n.get()+lanes)
        
        if self.draw_street:
            self.draw_street.notify(self, other, lanes)

    def connect_both_ways(self, other: Crossing, lanes):
        """Connects two Crossings in both ways. if LANES is a list,
         different amount of lanes in either direction
        can be specified."""
        if isinstance(lanes, list):
            assert len(lanes) == 2, "lanes needs to be of type [int, int] or int"
            lanes1, lanes2 = lanes
        else:
            assert isinstance(lanes, int), "lanes needs to be a number"
            lanes1, lanes2 = lanes, lanes
        
        self.connect(other, lanes1)
        other.connect(self, lanes2)

    def disconnect(self, other: Crossing, lanes, force=False):
        """Disconnects a crossing from another one, but ONLY in that one direction,
        
        force: instead of decreasing the lane count, remove completely"""
        
        # get variable storing the index
        assert isinstance(lanes, int)
        other_indices = [
            i for i, (crossing, _) in enumerate(self._connected) if crossing == other
        ] 
        if not other_indices:
            return
        i = other_indices[0]
        
        lanes_var = self._connected[i][1]
        n_value = lanes_var.get()-min(lanes, lanes_var.get())

        if n_value and not force:
            lanes_var.set(n_value)
        else:
            # delete connection
            if self.delete_street:
                self.delete_street.notify(self, other)
            self._connected.pop(i)

    def is_connected(self, other):
        """Checks if crossing is already connected to other crossing
        Returns Bool, <number_of_lanes> (None, int)"""
        for c, n in self._connected:
            if c == other:
                return n.get()
        return 0

    def delete_streets(self):
        if self.delete_street:
            for other, lanes in self._connected:
                self.delete_street.notify(self, other)
            

class StreetData:
    def __init__(self, log=False):
        self._crossings = []
        self.on_pos_change = event.Event(name="on_pos_change", log=log)
        self.on_pos_change += self._redraw_on_pos_change
        self.draw_crossing = event.Event(name="draw_crossing", log=log)
        self.delete_crossing = event.Event(name="delete_crossing")
        self.draw_street = event.Event(name="draw_street", log=log)
        self.delete_street = event.Event(name="delete_street", log=log)
    

    def add(self, c: Crossing):
        # TODO: Solve the case where the crossing already has observers
        c.on_pos_change = self.on_pos_change
        c.draw_street = self.draw_street
        c.draw_crossing = self.draw_crossing
        c.delete_street = self.delete_street
        self._crossings.append(c)
    
    def delete(self, c: Crossing):
        idel = None
        for i, crossing in enumerate(self._crossings):
            if crossing == c: 
                idel = i
            elif crossing.is_connected(c):
                crossing.disconnect(c, 1, force=True)
        if i is not None:
            c = self._crossings.pop(idel)
            c.delete_streets()
            self.delete_crossing.notify(c)
            for other, lanes in c._connected:
                c.disconnect(other, 1, force=True)

    def _redraw_on_pos_change(self, crossing):
        # redraw streets and crossings
        self.draw_crossing.notify(crossing)
        for c in self._crossings:
            lanes = c.is_connected(crossing)
            if lanes:
                self.draw_street.notify(c, crossing, lanes)
                
        for other, lanes in crossing._connected:
            self.draw_street.notify(crossing, other, lanes)

    def get_nearest(self, x, y):
        nearest_crossing = None
        min_dist_sqr = None
        for c in self._crossings:
            dist_sqr = (c.position[0]-x)**2 + (c.position[1]-y)**2
            if min_dist_sqr is None or dist_sqr < min_dist_sqr:
                nearest_crossing = c
                min_dist_sqr = dist_sqr
        return min_dist_sqr, nearest_crossing
    
    def export_to_json(self, path=None, debug=False):
        """Creates a json file from the street data"""
        
        json_dict = {
            "crossings" : [],
        }
        
        index_mapping = {
            c: i for i, c in enumerate(self._crossings)
        }
        
        for i, c in enumerate(self._crossings):
            json_dict["crossings"].append(
                {
                    "traffic_lights": c.traffic_lights,
                    "is_io_node": c.is_io_node,
                    "connected": [
                        (index_mapping[connection], lanes.get()) for connection, lanes in c._connected
                    ]
                }
            )
        
        if not debug:
            text = json.dumps(json_dict)
            if path:
                with open(path, "w") as f:
                    f.write(text)
            else:
                with filedialog.asksaveasfile() as f:
                    f.write(text)
        else:
            pprint(json_dict)


        



if __name__ == "__main__":
    from tkinter import Tk
    a = Tk()
    cr = Crossing([3,5], [], True)
    cs = Crossing([3,5], [], True)
    cr.connect(cs, 5)
    cr. disconnect(cs, 4)
    print(cr.is_connected(cs))