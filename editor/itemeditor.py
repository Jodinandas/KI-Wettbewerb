import tkinter
import enum
from editable import Editable, EditableField

class ItemEditor(tkinter.LabelFrame):
    """An editor for editing e.g. Crossings
    
    This frame is recursive and can display fields that are
    Editables themselves"""

    def __init__(self, master, name="", _parents=[]):
        """General design choices"""
        
        super().__init__(master)

        # holds a reference to all the widgets
        self._displayed_items = []
        self._last_row = 0
        self._parents = _parents
        self._name = name
    
    def display(self, item):
        """Creates widgets for all the marked fields in the editable"""
        
        if isinstance(item, Editable):
            marked_fields = item.marked_fields
        else:
            marked_fields = item
        self._parents.append(type(item))
        self.config(text=self._name)

        for field in marked_fields:
            widgets = self._generate_widget(field)
            start = self._last_row
            for i, w in enumerate(widgets):
                print(w, type(w))
                if isinstance(w, ItemEditor):
                    self._displayed_items.append(w)
                    w.grid(row=start+i, column=0, columnspan=2)
                else:
                    label = tkinter.Label(self, text=field.name)
                    label.grid(row=start+i, column=0)
                    w.grid(row=start+i, column=1)
                    self._displayed_items.extend((label, w))
                self._last_row += 1
    
    def clear(self):
        """Clears all displayed widgets"""
        
        for w in self._displayed_items:
            w.grid_forget()
        
        self._displayed_items.clear()
        self._parents.clear()
        self._last_row = 0

    def _generate_widget(self, field: EditableField):
        new_widgets = [] 
        if isinstance(field.var, Editable):
            # prevent infinite recursion
            if type(field.var) in self._parents:
                new_widgets.append(tkinter.Label(
                    self,
                    text=str(type(field.var))
                ))
            else:
                new_widgets.append(ItemEditor(
                    self,
                    _parents=self._parents
                ))
                new_widgets[-1].display(field.var)
            return new_widgets
        
        if isinstance(field.var, tkinter.IntVar):
            new_widgets.append(tkinter.Scale(
                self,
                from_=field.range[0],
                to=field.range[1],
                variable=field.var,
                resolution=field.step if field.step else 1,
                orient=tkinter.HORIZONTAL
            ))
        elif isinstance(field.var, tkinter.StringVar):
            new_widgets.append(tkinter.Entry(
                self,
                textvariable=field.var,
                state=tkinter.DISABLED if field.readonly else tkinter.NORMAL
            ))
        elif isinstance(field.var, tkinter.DoubleVar):
            new_widgets.append(tkinter.Scale(
                self,
                from_=field.range[0],
                to=field.range[1],
                resolution=field.step if field.step else 0.01,
                variable=field.var,
                orient=tkinter.HORIZONTAL
            ))
        elif isinstance(field.var, tkinter.BooleanVar):
            # Nice hacky code 
            
            print(field.readonly)
            check = tkinter.Checkbutton(
                self,
                fg="green",
                variable=field.var,
                state=tkinter.DISABLED if field.readonly else tkinter.NORMAL
            )

            new_widgets.append(check)
        elif isinstance(field.var, list):
            new_widgets.append(ItemEditor(
                self,
                name=field.name,
                _parents=self._parents
            ))
            new_widgets[-1].display(field.var)
        else:
            new_widgets.append(tkinter.Label(
                self,
                fg="red",
                text=f"No Widget implemented for type '{type(field.var)}'"
            ))
            
        return new_widgets


if __name__ == "__main__":
    class Epp(tkinter.Tk, Editable):
        def __init__(self):
            tkinter.Tk.__init__(self)
            Editable.__init__(self)
            
            self.item_editor = ItemEditor(self)
            self.item_editor.grid(row=0, column=0)

            self.a = tkinter.IntVar()
            self.a.set(3)
            self.b = tkinter.StringVar()
            self.b.set(6)
            self.c = []
            self.d = tkinter.BooleanVar()
            self.d.set(True)
            self.e = tkinter.DoubleVar()
            self.a.trace("w", lambda *_: print(self.a.get()))
            self.b.trace("w", lambda *_: print(self.b.get()))
            self.d.trace("w", lambda *_: print(self.d.get()))
            self.e.trace("w", lambda *_: print(self.e.get()))
            for i in range(10):
                var = tkinter.IntVar()
                var.set(i+1)
                def _temp(v, _, m): print(self.c[i], i)
                var.trace("w", _temp)
                self.c.append(var)

            self.mark_editable(self.a, name="Toller Slider:", range_=(10, 100))
            self.mark_editable(self.b, name="tolles entry")
            self.mark_editable(self.c, name="Tolll", range_=(1, 11))
            self.mark_editable(self.d, name="Tolle checkbox", readonly=False)
            self.mark_editable(self.e, name="toller float slider", readonly=False)
            
            self.item_editor.display(self)


    
    e = Epp()
    e.mainloop()