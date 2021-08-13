import tkinter
import event
import enum

class Tool(enum.Enum):
    SELECTION = enum.auto()
    DELETION = enum.auto()
    ADD = enum.auto()
    EXPORT = enum.auto()

class Toolbar(tkinter.Frame):
    """A toolbox to store things like making a new road"""
    
    COLOR_SELECTED = "red"
    COLOR_DEFAULT = "grey"

    def __init__(self, master):
        super().__init__(master)
        
        # All tools in the toolbar
        self._tools = {
            Tool.SELECTION: 
                tkinter.Button(
                    self,
                    text="Selection", 
                    bg=Toolbar.COLOR_SELECTED,
                    command = lambda: self.set_selected_tool(Tool.SELECTION)
            ),
            Tool.DELETION: 
                tkinter.Button(
                    self,
                    text="Delete", 
                    bg=Toolbar.COLOR_DEFAULT,
                    command = lambda: self.set_selected_tool(Tool.DELETION)
                ),
            Tool.ADD:
                tkinter.Button(
                    self,
                    text="Add", 
                    bg=Toolbar.COLOR_DEFAULT,
                    command = lambda: self.set_selected_tool(Tool.ADD)
                ),
            Tool.EXPORT:
                tkinter.Button(
                    self,
                    text="Export", 
                    bg=Toolbar.COLOR_DEFAULT,
                    command = lambda: self.on_export.notify() 
                ),
        }

        # Grid tools
        for i, (_tooltype, toolbutton) in enumerate(self._tools.items()):
            toolbutton.grid(row=i, column=0)

        # set up events
        self.tool_changed = event.Event(name="tool_changed")
        self.on_export = event.Event(name="on_export")
        
        self._selected_tool = Tool.SELECTION
    
    @property
    def selected_tool(self):
        return self._selected_tool
    
    @selected_tool.setter
    def selected_tool(self, new: Tool):
        if new != self._selected_tool:
            self._tools[self._selected_tool].config(bg=Toolbar.COLOR_DEFAULT)
            self._selected_tool = new
            self._tools[new].config(bg=self.COLOR_SELECTED)
            self.tool_changed.notify(new)
        
    def set_selected_tool(self, new: Tool):
        self.selected_tool = new