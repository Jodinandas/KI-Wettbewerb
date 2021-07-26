
__all__ = ["Editable"]

class EditableField:
    def __init__(self, parent, name, readonly=False, event=None):
        self.parent = parent
        self.name = name
        self.readonly = readonly
        self.event = event
    
    def update(self, new_value):
        self.new_value = new_value
        if self.event:
            self.event.notify(self.name, new_value)
            
    def inherits_editable(self):
        value = getattr(self.parent, self.name)
        return isinstance(value, Editable)


class Editable:
    """A class that can be inherited to automatically generate input fields in the UI"""
    
    def __init__(self):
        self._marked_fields = []

    def mark_editable(self, attr_name: str, *args, **kwargs):
        assert hasattr(self, attr_name), "Field to track doesn't exist."
        # TODO: Make it possible to check if the childs are editable as well
        
        self._marked_fields.append(
            EditableField(self, attr_name, *args, **kwargs)
        )

    @property
    def marked_fields(self):
        return self._marked_fields
    
    