from street_data import StreetData, Crossing
from toolbar import Tool
from event import Event


class InputParser:
    def __init__(self, street_data: StreetData):
        self.street_data = street_data
        self.add_crossing = Event(name="add_crossing")
        self.add_street = Event(name="add_street")
        self.remove_crossing = Event(name="remove_crossing")
        self.move_crossing = Event(name="move_crossing")
        self.select_crossing = Event(name="select_crossing")
        self.unselect_crossing = Event(name="unselect_crossing")
        # TODO: Find a better way to synchronise this with the
        #  internal state of the toolbar. As is, this could cause
        #  bugs, where the different componenents think different
        #  tools are selected. This would e.g. happen if one were
        #  to change the default Tool in toolbar.py
        self.selected_tool = Tool.SELECTION
        self.selected = None
        self.dragging = None
    
    def parse_mouse_left(self, event):
        if self.selected_tool == Tool.ADD:
            c = Crossing([event.x, event.y])
            self.add_crossing.notify(c)
            if self.selected:
                self.unselect_crossing.notify(self.selected)
                self.add_street.notify(self.selected, c)
            self.select_crossing.notify(c)
            self.selected = c
        elif self.selected_tool == Tool.SELECTION:
            dist, sel = self.street_data.get_nearest(event.x, event.y)
            if dist <= 50:
                self.unselect_crossing.notify(self.selected)
                self.select_crossing.notify(sel)
                self.dragging = sel
                self.selected = sel
    def on_left_release(self, event):
        self.dragging = None
            
    
    def on_mouse_move(self, event):
        if self.dragging:
            self.selected.position = [event.x, event.y]
            self.move_crossing.notify(self.dragging, event.x, event.y)

    def parse_mouse_right(self, event):
        pass
    
    def on_tool_change(self, tool: Tool):
        self.selected_tool = tool