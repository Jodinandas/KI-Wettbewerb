from street_data import StreetData, Street
from toolbar import Tool
from event import Event


class InputParser:
    def __init__(self, street_data: StreetData):
        self.street_data = street_data
        self.add_street_segment = Event(name="add_street_segment")
        self.remove_street_segment = Event(name="remove_street_segment")
        self.select_street = Event(name="select_street")
        self.unselect_street = Event(name="unselect_street")
        self.add_street = Event(name="add_street")
        # TODO: Find a better way to synchronise this with the
        #  internal state of the toolbar. As is, this could cause
        #  bugs, where the different componenents think different
        #  tools are selected. This would e.g. happen if one were
        #  to change the default Tool in toolbar.py
        self.selected_tool = Tool.SELECTION
        self.selected = None
    
    def parse_mouse_left(self, event):
        if self.selected_tool == Tool.ADD:
            if self.selected is None:
                # select the nearest element
                dist, sel = self.street_data.get_nearest(event.x, event.y)
                if dist is None:
                    s = Street()
                    self.selected = s
                    self.add_street.notify(s)
                else:
                    self.selected = sel
                self.select_street.notify(self.selected)
                self.add_street_segment.notify(self.selected, event.x, event.y)
            elif isinstance(self.selected, Street):
                self.add_street_segment.notify(self.selected, event.x, event.y)

    def parse_mouse_right(self, event):
        pass
    
    def on_tool_change(self, tool: Tool):
        self.selected_tool = tool