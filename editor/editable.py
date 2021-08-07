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
        print("regenerating: ", items)

        new_items = EditableList()
        nargs = [arg for arg in args if arg != "name"]
        nkwargs = {k: v for k, v in kwargs.items() if k != "name"}
        
        for i, item in enumerate(items):
            if isinstance(item, EditableList):
                new_items.append(
                    EditableField.from_list(item,
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


n = 0
class EditableList(list):


    def __init__(self, *args, event=None):
        global n
        list.__init__(self, args)
        self.event = event if event else Event(name=f'on_list_edit{n}')
        def _on_change_wrapper(self, func):
            def notify(*args, **kwargs):
                print(args, kwargs)
                result = func(self, *args, **kwargs) 

                if self.event:
                    print(self.event.name, "exisitert",
                    self.event._observer_funcs)
                    self.event.notify(self)
                return result
            return notify
        self.extend = _on_change_wrapper(self, list.extend)
        self.append = _on_change_wrapper(self, list.append)
        self.remove = _on_change_wrapper(self, list.remove)
        self.pop = _on_change_wrapper(self, list.pop)
        self.__delitem__ = _on_change_wrapper(self, list.__delitem__)
        self.__setitem__ = _on_change_wrapper(self, list.__setitem__)
        self.__iadd__ = _on_change_wrapper(self, list.__iadd__)
        self.__imul__ = _on_change_wrapper(self, list.__imul__)
        n += 1

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
    




    
    