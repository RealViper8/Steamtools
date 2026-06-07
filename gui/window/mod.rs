use crate::App;
use eframe::egui;
pub trait WindowPopup {
    fn view(app: &mut App, ui: &mut egui::Ui);
}

mod mods;
pub use mods::ModsPopup;

mod view;
pub use view::ViewPopup;

mod settings;
pub use settings::Settings;

mod plugins;
pub use plugins::Plugins;

mod install;
pub use install::InstallPopup;
