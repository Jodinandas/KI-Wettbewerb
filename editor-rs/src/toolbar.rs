use bevy_egui::egui::Ui;

#[derive(PartialEq)]
pub enum ToolType {
    None,
    Pan,
    AddStreet,
    AddCrossing,
    AddIONode,
    Select,
    DeleteNode,
}

pub trait Tool: Send + Sync {
    fn name<'a>(&'a self) -> &'a str;
    fn render(
        &self,
        ui: &mut Ui,
        selected_index: &mut Option<usize>,
        this_index: usize,
        locked: bool,
    ) {
        if ui.button(self.name()).clicked() {
            if !locked {
                *selected_index = Some(this_index)
            }
        }
    }
    fn get_type(&self) -> ToolType;
}

pub struct Toolbar {
    tools: Vec<Box<dyn Tool>>,
    // Can be none if there are no tools
    selected: Option<usize>,
    // if it is locked, no tool can be selected
    pub locked: bool,
}

impl Toolbar {
    pub fn new() -> Toolbar {
        Toolbar {
            tools: vec![],
            selected: None,
            locked: false,
        }
    }

    pub fn get_selected<'a>(&'a self) -> Option<&'a Box<dyn Tool>> {
        match self.selected {
            Some(i) => Some(&self.tools[i]),
            None => None,
        }
    }
    pub fn get_tooltype(&self) -> ToolType {
        match self.get_selected() {
            Some(tool) => tool.get_type(),
            None => ToolType::None,
        }
    }

    pub fn render_tools(&mut self, ui: &mut Ui) {
        for (i, tool) in self.tools.iter().enumerate() {
            tool.render(ui, &mut self.selected, i, self.locked);
        }
    }
}

impl Default for Toolbar {
    fn default() -> Toolbar {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(PanTool::new()),
            Box::new(SelectTool::new()),
            Box::new(AddStreetTool::new()),
            Box::new(AddCrossingTool::new()),
            Box::new(DeleteNodeTool::new()),
            Box::new(AddIONodeTool::new()),
        ];

        Toolbar {
            tools,
            selected: Some(0),
            locked: false,
        }
    }
}

pub struct SelectTool;

impl Tool for SelectTool {
    fn name<'a>(&'a self) -> &'a str {
        "Select"
    }
    fn get_type(&self) -> ToolType {
        ToolType::Select
    }
}
impl SelectTool {
    pub fn new() -> SelectTool {
        SelectTool {}
    }
}
pub struct PanTool;

impl Tool for PanTool {
    fn name<'a>(&'a self) -> &'a str {
        "Pan"
    }
    fn get_type(&self) -> ToolType {
        ToolType::Pan
    }
}
impl PanTool {
    pub fn new() -> PanTool {
        PanTool {}
    }
}

pub struct AddStreetTool;

impl Tool for AddStreetTool {
    fn name<'a>(&'a self) -> &'a str {
        "Add Street"
    }
    fn get_type(&self) -> ToolType {
        ToolType::AddStreet
    }
}
impl AddStreetTool {
    pub fn new() -> AddStreetTool {
        AddStreetTool {}
    }
}
pub struct AddCrossingTool;

impl Tool for AddCrossingTool {
    fn name<'a>(&'a self) -> &'a str {
        "Add Crossing"
    }
    fn get_type(&self) -> ToolType {
        ToolType::AddCrossing
    }
}
impl AddCrossingTool {
    pub fn new() -> AddCrossingTool {
        AddCrossingTool {}
    }
}

pub struct AddIONodeTool;

impl Tool for AddIONodeTool {
    fn name<'a>(&'a self) -> &'a str {
        "Add IO-Node"
    }
    fn get_type(&self) -> ToolType {
        ToolType::AddIONode
    }
}
impl AddIONodeTool {
    pub fn new() -> AddIONodeTool {
        AddIONodeTool {}
    }
}
pub struct DeleteNodeTool;

impl Tool for DeleteNodeTool {
    fn name<'a>(&'a self) -> &'a str {
        "Delete Crossing/IO-Node"
    }
    fn get_type(&self) -> ToolType {
        ToolType::DeleteNode
    }
}
impl DeleteNodeTool {
    pub fn new() -> DeleteNodeTool {
        DeleteNodeTool {}
    }
}
