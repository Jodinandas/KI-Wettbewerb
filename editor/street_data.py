import logging 
import math

class Street:
    def __init__(self):
        self.points = []
        self.point_items = []
        self.arc = None
    
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