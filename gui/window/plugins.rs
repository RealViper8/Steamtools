
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
    pub list: Vec<Plugin>
}