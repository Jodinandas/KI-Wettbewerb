import tkinter
from event import Event

class EditableField:
    def __init__(self, var, readonly=False, name="", event=None, range_=(-5, 5), step=None, slider=False):
        self.name = name
        self.var = var
        self.readonly = readonly
        self.event = event
        self.range = range_
        self.step = step
        self.slider = slider
    
            
    def inherits_editable(self):
        return isinstance(self.var, Editable)


    @staticmethod
    def from_list(items: list, *args, **kwargs):
        """Recursively turns all list items into EditableField"""

        new_items = EditableList()
        nargs = [arg for arg in args if arg != "name"]
        nkwargs = {k: v for k, v in kwargs.items() if k != "name"}
        
        for i, item in enumerate(items):
            if isinstance(item, EditableList):
                new_items.append(EditableField(
                    EditableField.from_list(item),
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

def _on_change_wrapper(func):
    def notify(self, *args, **kwargs):
        if self.event:
            self.event.notify(self)
        return func(self, *args, **kwargs) 
    return notify

class EditableList(list):
    extend = _on_change_wrapper(list.extend)
    append = _on_change_wrapper(list.append)
    remove = _on_change_wrapper(list.remove)
    pop = _on_change_wrapper(list.pop)
    __delitem__ = _on_change_wrapper(list.__delitem__)
    __setitem__ = _on_change_wrapper(list.__setitem__)
    __iadd__ = _on_change_wrapper(list.__iadd__)
    __imul__ = _on_change_wrapper(list.__imul__)


    def __init__(self, *args, event=None):
        list.__init__(self, args)
        self.event = event if event else Event('on_list_edit')

    def __getitem__(self,item):
        if isinstance(item,slice):
            return self.__class__(list.__getitem__(self,item))
        else:
            return list.__getitem__(self,item)



class Editable:
    """A class that can be inherited to automatically generate input fields in the UI"""
    
    def __init__(self):
        self._marked_fields = []

    def mark_editable(self, var, *args, **kwargs):

        new_editable = None

        if isinstance(var, EditableList):
            new_editable = EditableField.from_list(var, *args, **kwargs)
        else:
            new_editable = EditableField(var, *args, **kwargs)
        self._marked_fields.append(new_editable)

    @property
    def marked_fields(self):
        return self._marked_fields
    




    
    