import tkinter

class EditableField:
    def __init__(self, var, readonly=False, name="", event=None, range_=(-5, 5), step=None):
        self.name = name
        self.var = var
        self.readonly = readonly
        self.event = event
        self.range = range_
        self.step = step
    
            
    def inherits_editable(self):
        return isinstance(self.var, Editable)


class Editable:
    """A class that can be inherited to automatically generate input fields in the UI"""
    
    def __init__(self):
        self._marked_fields = []

    def mark_editable(self, var, *args, **kwargs):

        new_editable = None

        if isinstance(var, list):
            new_editable = self._all_to_editable_fields(var, *args, **kwargs)
        else:
            new_editable = EditableField(var, *args, **kwargs)
        self._marked_fields.append(new_editable)

    @property
    def marked_fields(self):
        return self._marked_fields
    
    def _all_to_editable_fields(self, items: list, *args, **kwargs) -> list:
        """Recursively turns all list items into EditableField"""

        new_items = []
        nargs = [arg for arg in args if arg != "name"]
        nkwargs = {k: v for k, v in kwargs.items() if k != "name"}

        for i, item in enumerate(items):
            if isinstance(item, list):
                new_items.append(EditableField(
                    self._all_to_editable_fields(item),
                    name=f"# {i}",
                    *nargs,
                    **nkwargs
                ))
            else:
                new_items.append(
                    EditableField(
                        item,
                        name=f"# {i}",
                        *nargs,
                        **nkwargs
                    )
                )

        return EditableField(new_items, *args, **kwargs)




    
    