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
        def generate_new_child(new, delete):
            if not delete:
                if isinstance(new, list):
                    new_items.append(EditableField.from_list(new, *nargs, **nkwargs))
                else:
                    new_items.append(EditableField(new, *nargs, **nkwargs))
            else:
                new_items.pop(new)
        
        items.event += generate_new_child
        # if isinstance(items, EditableList):
        #     new_items.event = items.event
        

        return EditableField(new_items, *args, **kwargs)


n = 0
class EditableList(list):
    def __init__(self, *args, event=None):
        global n
        list.__init__(self, args)
        # Is called when an element is added or removed
        # if removed delete is set to true and the first argument is an index
        self.event = event if event else Event(name=f'on_list_edit{n}')
                
        # Make sure to call the event if something changes
        def _on_change_wrapper(self, func):
            def notify(*args, **kwargs):
                # print(args, kwargs)
                result = func(self, *args, **kwargs) 

                if self.event:
                    print(self.event.name, "exisitert",
                    self.event._observer_funcs, func.__name__)
                    self.event.notify(self)
                return result
            return notify
        # Bind functions 
        # TODO: Make special wrappers for iadd, imul etc.
        # self.extend = _on_change_wrapper(self, list.extend)
        # self.append = _on_change_wrapper(self, list.append)
        # self.remove = _on_change_wrapper(self, list.remove)
        # self.pop = _on_change_wrapper(self, list.pop)
        # self.__delitem__ = _on_change_wrapper(self, list.__delitem__)
        # self.__setitem__ = _on_change_wrapper(self, list.__setitem__)
        # self.__iadd__ = _on_change_wrapper(self, list.__iadd__)
        # self.__imul__ = _on_change_wrapper(self, list.__imul__)
        n += 1

    def extend(self, n_elements):
        """special wrapper for extend
        
        When extend is called, the list has to check if the new elements are editable lists 
        as well to make sure the callback is invoked if necessary"""
        
        # for el in n_elements:
        #     if isinstance(el, EditableList):
        #         el.event += lambda c_els: self.event.notify(self)

        list.extend(self, n_elements) 
        for el in n_elements:
            self.event.notify(el, delete=False)
    
    def append(self, n_element):
        """special wrapper for append 
        
        When append is called, the list has to check if the new element is an editable list 
        to make sure the callback is invoked if necessary"""
        
        # if isinstance(n_element, EditableList):
        #     n_element.event += lambda c_els: self.event.notify(self)

        list.append(self, n_element) 
        self.event.notify(n_element, delete=False)

    def pop(self, i):
        el = list.pop(self, i)
        self.event.notify(i, delete=True)
        return el

    def remove(self, el):
        i = self.index(el)
        self.pop(i)

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
    
if __name__ == "__main__":
    import logging
    logging.basicConfig(level=logging.DEBUG)
    l = EditableList(1, 2, 3, 4)
    l.event += print
    l.append(5)
    l.extend([6, 7])
    l.remove(7)
    l.pop(-1)
    l.append(EditableList(EditableList(123, 124, 125), 1043))
    l[-1].append(4)
    l[-1][0].extend([1, 2, 3])
    




    
    