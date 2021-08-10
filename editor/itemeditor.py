import tkinter
import enum
from editable import Editable, EditableField, EditableList
from street_data import Crossing
import logging
from event import Event

class ItemEditor(tkinter.LabelFrame):
    """An editor for editing e.g. Crossings
    
    This frame is recursive and can display fields that are
    Editables themselves"""

    def __init__(self, master, name="", _parents=[]):
        """General design choices"""
        
        super().__init__(master, text=name)

        # holds a reference to all the widgets
        self._displayed_items = []
        self._parents = _parents
        self._name = name
        self._empty_placeholder = tkinter.Label(self, text="No widgets") 
        self._empty_placeholder.grid(row=0, column=0)

    def on_list_change(self, nlist):
        """Is called when the ItemEditor is displaying a list and the list changes"""
        
        print("redrawing: ", nlist)
        self.clear()
        new_editable_fields = EditableField.from_list(nlist)
        self.display([new_editable_fields])

    
    def display(self, item):
        """Creates widgets for all the marked fields in the editable"""
        
        if item:
            self._empty_placeholder.grid_forget()
        
        if isinstance(item, Editable):
            marked_fields = item.marked_fields
            # self._parents.append(type(item))
        else:
            marked_fields = item

        for field in marked_fields:
            widgets = self._generate_widget(field)
            start = len(self._displayed_items)
            for i, w in enumerate(widgets):
                if isinstance(w, ItemEditor):
                    self._displayed_items.append(w)
                    w.grid(row=start+i, column=0, columnspan=2)
                else:
                    label = tkinter.Label(self, text=field.name)
                    label.grid(row=start+i, column=0)
                    w.grid(row=start+i, column=1)
                    self._displayed_items.append((label, w))
    
    def clear(self):
        """Clears all displayed widgets"""
        
        for w in self._displayed_items:
            if isinstance(w, tuple):
                for el in w:
                    el.grid_forget()
            else:
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
                    _parents=self._parents + [type(field.var)]
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
                orient=tkinter.HORIZONTAL,
                length=500
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
                orient=tkinter.HORIZONTAL,
                length=100
            ))
        elif isinstance(field.var, tkinter.BooleanVar):
            # Nice hacky code 
            
            check = tkinter.Checkbutton(
                self,
                fg="green",
                variable=field.var,
                state=tkinter.DISABLED if field.readonly else tkinter.NORMAL
            )

            new_widgets.append(check)
        elif isinstance(field.var, EditableList):
            print("makedy dakedy list", field.name, field.var)
            s = ItemEditor(
                self,
                name=field.name,
                _parents=self._parents
            )
            new_widgets.append(s)
            # Bind update event from EditableList to rerender
            def add_el_of_list(el, delete):
                print("el", el)
                if isinstance(el, EditableList):
                    field = EditableField.from_list(el)
                else:
                    field = EditableField(el) 
                s.add_field(field)
            field.var.event += add_el_of_list
            s.display(field.var)
            self.update()
        else:
            new_widgets.append(tkinter.Label(
                self,
                fg="red",
                text=f"No Widget implemented for type '{type(field.var)}'"
            ))
            
        return new_widgets

    def add_field(self, field: EditableField):
        self._empty_placeholder.grid_forget()
        w = self._generate_widget(field)[0]
        if isinstance(w, ItemEditor):
            w.grid(row=len(self._displayed_items))
            self._displayed_items.append(w)
        else:
            label = tkinter.Label(self, text=field.name)
            label.grid(row=len(self._displayed_items), column=0)
            w.grid(row=len(self._displayed_items), column=1)
            self._displayed_items.append((label, w))
    
    def remove_field(self, i):
        w = self._displayed_items.pop(i)
        w.grid_forget()
        if not self._displayed_items:
            self._empty_placeholder.grid(row=0, column=0)


if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG,
                        format='%(asctime)s (%(levelname)-2s) %(message)s',
                        datefmt='[%d.%m.%Y %H:%M:%S]')
    

    class Epp(tkinter.Tk, Editable):
        def __init__(self):
            tkinter.Tk.__init__(self)
            Editable.__init__(self)
            
            self.item_editor = ItemEditor(self)
            self.item_editor.grid(row=0, column=0)

            # self.a = tkinter.IntVar()
            # self.a.set(3)
            # self.b = tkinter.StringVar()
            # self.b.set(6)
            # self.c = EditableList() 
            # self.d = tkinter.BooleanVar()
            # self.d.set(True)
            # self.e = tkinter.DoubleVar()
            # self.f = EditableList(event=Event(name="Main list got changed"))
            # self.c.event.log = True
            # self.c.event += lambda *a, **b: print(a, b)
            self.crossing1 = Crossing([1, 100])
            self.crossing2 = Crossing([2, 200])
            # for i in range(2):
            #     var = tkinter.IntVar()
            #     var.set(i+1)
            #     var2 = tkinter.IntVar()
            #     var2.set(i+2)
            #     self.c.append(EditableList(var, var2))
            # 
            # 
            # def on_check(*args, **kwargs):
            #     self.c.append(EditableList(tkinter.IntVar(), tkinter.IntVar()))
            #     print("C ---------------------", len(self.c))
            # self.d.trace("w", on_check)

            # self.mark_editable(self.a, name="Toller Slider:", range_=(10, 100))
            # self.mark_editable(self.b, name="tolles entry")
            # self.mark_editable(self.c, name="Tolll", range_=(1, 11))
            # self.mark_editable(self.d, name="Tolle checkbox", readonly=False)
            # self.mark_editable(self.e, name="toller float slider", readonly=False)
            # self.mark_editable(self.f, name="Leeres Ding", readonly=False)
            self.mark_editable(self.crossing1, name="Crossing1", readonly=False)
            self.mark_editable(self.crossing2, name="Crossing2", readonly=False)
            # self.mark_editable(self.f, name="Rekursive liste", range_=(1, 11))
            def on_change(*args):
                # self.f.append(EditableList(EditableList(self.f), tkinter.IntVar()))
                self.crossing1.connect_both_ways(self.crossing2, 2)
                # self.f.append(EditableList(Crossing([1, 2]), tkinter.IntVar(value=1)))
                # self.c.event += self.item_editor.on_list_change
                # self.c.event.notify(self)
            self.after(3000, on_change)

            
            self.item_editor.display(self.crossing1)
            # self.item_editor.clear()
            # self.item_editor.display(self.crossing2)


    
    e = Epp()
    e.mainloop()