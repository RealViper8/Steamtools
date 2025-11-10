use std::ops::Index;


#[derive(Default)]
pub struct Plugin {
    pub code: String,
    pub name: String,
}

#[derive(Default)]
pub struct Plugins {
    pub active: bool,
    pub ceditor: bool,
    pub fetched: bool,
    pub selected_plugin: Option<usize>,
    pub list: Vec<Plugin>
}

impl Plugins {
    pub fn get_selected(&self) -> Option<&Plugin> {
        if let Some(p) = self.selected_plugin {
            Some(&self[p])
        } else {
            None
        }
    }
}

impl Index<usize> for Plugins {
    type Output = Plugin;
    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}