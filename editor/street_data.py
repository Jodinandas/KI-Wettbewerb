from __future__ import annotations
import logging
import math
from editable import EditableList, Editable
from tkinter import StringVar, IntVar, DoubleVar, BooleanVar
import event


class Crossing(Editable):
    def __init__(self, position: list, connected: list=[], traffic_lights: bool=False, on_pos_change=None):
        """class for saving of crossings. connected has to be a
        list in the form of [[crossing, number_of_lanes], [...]]
        """
        Editable.__init__(self)
        #TODO use custom list for connected
        self._position = EditableList(IntVar(value=position[0]), IntVar(value=position[1]))
        self.on_pos_change = on_pos_change
        self._position[0].trace("w", lambda *_: self.on_pos_change.notify(self))
        self._position[1].trace("w", lambda *_: self.on_pos_change.notify(self))
        self._connected = EditableList()

        self.graphic_object = None
        self.street_connections = {}

        for c, n in connected:
            self._connected.append([c, IntVar(n)])
        self._traffic_lights = BooleanVar(value=traffic_lights)
        
        # Mark for the editor
        self.mark_editable(self._position, name="position: ", range_=(0, 1500))
        self.mark_editable(self._connected, name="connected: ", range_=(0, 1500))
        self.mark_editable(self._traffic_lights, name="has traffic lights: ")

    @property
    def position(self):
        return [self._position[0].get(), self._position[1].get()]

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

    def connect_both_ways(self, other: Crossing, lanes):
        """Connects two Crossings in both ways. if LANES is a list,
         different amount of lanes in either direction
        can be specified."""
        if isinstance(lanes, list):
            if (not isinstance(lanes[0], int)) or (not isinstance(lanes[1], int)):
                raise ValueError("Connect_both_ways: Number needs to be of the type list [int, int] or int.")
            self._connected.append(EditableList(other, IntVar(value=lanes[0])))
            other._connected.append(EditableList(self, IntVar(value=lanes[1])))
        elif isinstance(lanes, int):
            self._connected.append(EditableList(other, IntVar(value=lanes)))
            other._connected.append(EditableList(self, IntVar(value=lanes)))
        else:
            raise ValueError("Connect_both_ways: Number needs to be of the type list [int, int] or int.")

    def disconnect(self, other: Crossing, lanes):
        """Disconnects a crossing from another one, but ONLY in that one direction,"""
        if isinstance(lanes, int):
            # disconnect lane
            for c, n in self._connected:
                if c == other:
                    # lanes cannot be negative, so min() is used
                    n.set(n.get()-min(lanes, n.get()))
        else:
            raise ValueError("lanes to dissconnect needs to be a valid int")

    def is_connected(self, other):
        """Checks if crossing is already connected to other crossing
        Returns Bool, <number_of_lanes> (None, int)"""
        for c, n in self._connected:
            if c == other:
                return n.get()
        return 0


class StreetData:
    def __init__(self):
        self.crossings = []
        self.on_pos_change = event.Event(name="on_pos_change", log=False)
    

    def add(self, c: Crossing):
        c.on_pos_change = self.on_pos_change
        self.crossings.append(c)

    def get_nearest(self, x, y):
        nearest_crossing = None
        min_dist_sqr = None
        for c in self.crossings:
            dist_sqr = (c.position[0]-x)**2 + (c.position[1]-y)**2
            if min_dist_sqr is None or dist_sqr < min_dist_sqr:
                nearest_crossing = c
                min_dist_sqr = dist_sqr
        return dist_sqr, nearest_crossing



if __name__ == "__main__":
    from tkinter import Tk
    a = Tk()
    cr = Crossing([3,5], [], True)
    cs = Crossing([3,5], [], True)
    cr.connect(cs, 5)
    cr. disconnect(cs, 4)
    print(cr.is_connected(cs))