use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use steamtools::{Game, install_melonloader};

use crate::STEAM_BINARY_PATH;
use crate::utils::bserializer::GameMap;
use crate::window::WindowPopup;
use crate::egui::{Window, TextEdit};

#[derive(Default)]
pub struct ModsPopup {
    pub active: bool,
}

impl WindowPopup for ModsPopup {
    fn view(app: &mut crate::App, ctx: &eframe::egui::Context) {
        Window::new("Mods").default_size([0.0, 0.0]).open(&mut app.mods.active).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
                ui.label("APPID:");
                ui.add(
                    TextEdit::singleline(&mut app.st.mod_id)
                        .hint_text("Enter appid")
                        .char_limit(10),
                );

                if ui.button("Get").clicked() {
                    if app.cached_games.0.is_empty() && !Path::new(STEAM_BINARY_PATH).exists() {
                        let games_guard = app.games.lock().unwrap();
                        let mut stfile = File::create(STEAM_BINARY_PATH).unwrap();
                        dbg!(&games_guard.iter().map(|g| (g.appid, g)).collect::<HashMap<u32, &Game>>());
                        GameMap::write_to(&mut stfile, games_guard.iter().map(|g| (g.appid, g)).collect::<HashMap<u32, &Game>>()).unwrap();
                    } else if app.cached_games.0.is_empty() && Path::new(STEAM_BINARY_PATH).exists() {
                        let mut stfile = File::open(STEAM_BINARY_PATH).unwrap();
                        app.cached_games.0 = GameMap::read_from(&mut stfile).unwrap();
                    }

                    if let Err(_) = app.st.mod_id.parse::<u32>() {
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Warning)
                            .set_buttons(rfd::MessageButtons::Ok)
                            .set_title("Warning")
                            .set_description("APPID should be numeric")
                            .show();
                    } else {
                        if let Some(s) = app.cached_games.0.get(&app.st.mod_id.parse::<u32>().unwrap()) {
                            install_melonloader(&s.path, app.st.melon_loader);
                        } else {
                            rfd::MessageDialog::new()
                                .set_title("Info")    
                                .set_level(rfd::MessageLevel::Info)
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_description(format!("You dont have {}.", app.st.mod_id))
                                .show();
                        }
                    }

                    // TODO: Add Game serialization to binary? 
                    // todo!("Cant get path without saving Game struct");
                };
            });
        });
    }
}