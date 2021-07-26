from __future__ import annotations
import logging
import math

class Street:
    def __init__(self):
        self._points = []
        self.point_items = []
        self.arc = None


class Crossing:
    def __init__(self, position: list, connected: list, traffic_lights: bool):
        """class for saving of crossings. connected has to be a
        list in the form of [[crossing, number_of_lanes], [...]]
        """
        self.position = position
        self.connected = connected
        self.traffic_lights = traffic_lights

    def connect(self, other: Crossing, lanes: int):
        """Connects two Crossings, but only one way. if exists, adds lanes"""
        if not self.is_connected(other):
            self.connected.append([other, lanes])
        else:
            for i, (c, n) in enumerate(self.connected):
                if c == other:
                    self.connected[i] = [c, n+lanes]


    def connect_both_ways(self, other: Crossing, lanes):
        """Connects two Crossings in both ways. if LANES is a list,
         different amount of lanes in either direction
        can be specified."""
        if isinstance(lanes, list):
            if (not isinstance(lanes[0], int)) or (not isinstance(lanes[1], int)):
                raise ValueError("Connect_both_ways: Number needs to be of the type list [int, int] or int.")
            self.connected.append([other, lanes[0]])
            other.connected.append([self, lanes[1]])
        elif isinstance(lanes, int):
            self.connected.append([other, lanes])
            other.connected.append([self, lanes])
        else:
            raise ValueError("Connect_both_ways: Number needs to be of the type list [int, int] or int.")

    def disconnect(self, other: Crossing, lanes):
        """Disconnects a crossing from another one, but ONLY in that one direction,"""
        if isinstance(lanes, int):
            # disconnect lane
            for i, (c, n) in enumerate(self.connected):
                if c == other:
                    # lanes cannot be negative, so min() is used
                    self.connected[i] = [c, n-min(lanes, n)]
        else:
            raise ValueError("lanes to dissconnect needs to be a valid int")

    def is_connected(self, other):
        """Checks if crossing is already connected to other crossing
        Returns Bool, <number_of_lanes> (None, int)"""
        for i, (c, n) in enumerate(self.connected):
            if c == other:
                return n
        return 0



class StreetData:
    def __init__(self):
        self.streets = []

    def get_nearest(self, x, y):
        # get the nearest street waypoint
        nearest_street = None
        min_dist_sqr = None
        for street in self.streets:
            logging.debug(street)
            # reminder: the points are a flattened list
            for i in range(0, len(street.points), 2):
                px = street.points[i]
                py = street.points[i+1]
                dist_sqr = (px - x)**2 + (py - y)**2
                if min_dist_sqr is None or dist_sqr < min_dist_sqr:
                    nearest_street = street
                    min_dist_sqr = dist_sqr
            else: return (None, None)
        else: return (None, None)
        return math.sqrt(min_dist_sqr), nearest_street