from __future__ import annotations
import logging
import math
from tkinter import StringVar, IntVar, DoubleVar, BooleanVar


class Crossing:
    def __init__(self, position: list, connected: list, traffic_lights: bool):
        """class for saving of crossings. connected has to be a
        list in the form of [[crossing, number_of_lanes], [...]]
        """
        #TODO use custom list for connected
        self._position = [IntVar(), IntVar()]
        self._position[0].set(position[0])
        self._position[1].set(position[1])
        self._connected = []
        for c, n in connected:
            var = IntVar()
            var.set(n)
            self._connected.append([c, var])
        self._traffic_lights = BooleanVar()
        self._traffic_lights.set(traffic_lights)

    @property
    def position(self):
        return [self._position[0].get(), self._position[1].get()]

    @position.setter
    def position(self, new: list):
        for i, pos in enumerate(self._position):
            pos.set(new[i])

    def connect(self, other: Crossing, lanes: int):
        """Connects two Crossings, but only one way. if exists, adds lanes"""
        if not self.is_connected(other):
            l = IntVar()
            l.set(lanes)
            self._connected.append([other, l])
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
            l1 = IntVar()
            l1.set(lanes[0])
            l2 = IntVar()
            l2.set(lanes[1])
            self._connected.append([other, l1])
            other._connected.append([self, l2])
        elif isinstance(lanes, int):
            l = IntVar()
            l.set(lanes)
            self._connected.append([other, l])
            other._connected.append([self, l])
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

    def get_nearest(self, x, y):
        nearest_crossing = None
        min_dist_sqr = None
        for c in self.crossings:
            dist_sqr = (c.position[0]-x)**2 + (c.position[1]-y)**2
            if min_dist_sqr is None or dist_sqr < min_dist_sqr:
                nearest_crossing = c
                min_dist_sqr = dist_sqr
        else:
            return nearest_crossing


if __name__ == "__main__":
    from tkinter import Tk
    a = Tk()
    cr = Crossing([3,5], [], True)
    cs = Crossing([3,5], [], True)
    cr.connect(cs, 5)
    cr. disconnect(cs, 4)
    print(cr.is_connected(cs))