from street_data import StreetData, Street

class InputParser:
    def __init__(self, street_data: StreetData):
        self.street_data = street_data
        self._bindings = {
            "add_street_segment": [], 
            "remove_street_segment": [],
            "select_street": [],
            "unselect_street": []
        }
        self.selected = None
    def bind(self, event, *funcs):
        assert event in self._bindings, f"Event '{event}' does not exist."
        assert min(callable(f) for f in funcs), "Events must be callable"
        
        self._bindings[event].extend(funcs)
    
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
            for f in self._bindings["select_street"]:
                f(self.selected, sel)
        elif isinstance(self.selected, Street):
            for f in self._bindings["add_street_segment"]:
                f(self.selected, event.x, event.y)

    def parse_mouse_right(self, event):
        pass