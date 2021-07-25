from street_data import StreetData, Street
from event import Event


class InputParser:
    def __init__(self, street_data: StreetData):
        self.street_data = street_data
        self.add_street_segment = Event(name="add_street_segment")
        self.remove_street_segment = Event(name="remove_street_segment")
        self.select_street = Event(name="select_street")
        self.unselect_street = Event(name="unselect_street")
        self.selected = None
    
    def parse_mouse_left(self, event):
        if self.selected is None:
            # select the nearest element
            dist, sel = self.street_data.get_nearest(event.x, event.y)
            if dist is None:
                s = Street()
                self.street_data.streets.append(s)
                self.selected = s
            else:
                self.selected = sel
            self.select_street.notify(self.selected)
        elif isinstance(self.selected, Street):
            self.add_street_segment.notify(self.selected, event.x, event.y)

    def parse_mouse_right(self, event):
        pass