use std::{ops::{Index, IndexMut}, sync::{Arc, Mutex}};

#[derive(Default)]
pub struct Plugin {
    pub code: String,
    pub name_buffer: String,
    pub name: Arc<Mutex<String>>,
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
    pub fn get(&self) -> Option<usize> {
        self.selected_plugin
    }
    pub fn get_selected(&mut self) -> Option<&mut Plugin> {
        if let Some(p) = self.selected_plugin {
            Some(&mut self[p])
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

impl IndexMut<usize> for Plugins {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.list[index]
    }
}