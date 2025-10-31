use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Default)]
pub struct Settings {
    pub mod_experimental: bool,
}