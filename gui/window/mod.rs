use eframe::egui::Context;
use crate::App;
pub trait WindowPopup {
    fn view(app: &mut App, ctx: &Context);
}

mod mods;
pub use mods::ModsPopup;

mod view;
pub use view::ViewPopup;

mod settings;
pub use settings::{Settings};

mod plugins;
pub use plugins::Plugins;

mod install;
pub use install::InstallPopup;