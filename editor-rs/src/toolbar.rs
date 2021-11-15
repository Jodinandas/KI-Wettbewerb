use std::ops::Add;

use bevy_egui::egui::Ui;

pub enum ToolType {
    Pan,
    AddStreet,
}

pub trait Tool: Send + Sync {
    fn name<'a>(&'a self) -> &'a str;
    fn render(&self, ui: &mut Ui, selected_index: &mut Option<usize>, this_index: usize) {
        if ui.button(self.name()).clicked() {
            *selected_index = Some(this_index)
        }
    }
    fn get_type(&self) -> ToolType;
}

pub struct Toolbar {
    tools: Vec<Box<dyn Tool>>,
    // Can be none if there are no tools
    selected: Option<usize>,
}

impl Toolbar {
    pub fn new() -> Toolbar {
        Toolbar {
            tools: vec![],
            selected: None,
        }
    }

    pub fn get_selected<'a>(&'a mut self) -> Option<&'a Box<dyn Tool>> {
        match self.selected {
            Some(i) => Some(&self.tools[i]),
            None => None,
        }
    }

    pub fn render_tools(&mut self, ui: &mut Ui) {
        for (i, tool) in self.tools.iter().enumerate() {
            tool.render(ui, &mut self.selected, i);
        }
    }
}

impl Default for Toolbar {
    fn default() -> Toolbar {
        let tools: Vec<Box<dyn Tool>> =
            vec![Box::new(PanTool::new()), Box::new(AddStreetTool::new())];

        Toolbar {
            tools,
            selected: Some(0),
        }
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
