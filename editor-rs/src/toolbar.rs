use egui::Ui;


pub trait Tool: Send + Sync {
    fn name<'a>(&'a self) -> &'a str;
    fn render(&self, ui: &mut Ui) {
        ui.label(self.name());
    }
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
            selected: None
        }
    }
    
    pub fn get_selected<'a>(&'a mut self) -> Option<&'a Box<dyn Tool>> {
        match self.selected {
            Some(i) => Some(&self.tools[i]),
            None => None
        }
    }
    
    pub fn render_tools(&self, ui: &mut Ui) {
        for tool in self.tools.iter() {
            tool.render(ui);
        }
    }
}

impl Default for Toolbar {
    fn default() -> Toolbar {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(PanTool::new())
        ];

        Toolbar {
            tools,
            selected: Some(0)
        }        
    }
}

pub struct PanTool;

impl Tool for PanTool {
    fn name<'a>(&'a self) -> &'a str {
        "Pan"
    }
}
impl PanTool {
    pub fn new() -> PanTool {
        PanTool {}
    }
}

