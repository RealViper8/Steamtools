#[cfg_attr(not(debug_assertions), windows_subsystem="windows")]

use std::process;
use std::{collections::HashMap, fs::{self, File}, io::{self, BufReader, BufWriter, Read, Write}, path::{Path, PathBuf}, sync::{Arc, Mutex}, thread};

use egui_extras::install_image_loaders;
use serde::{Serialize, Deserialize};
use eframe::egui::{self, FontId, RichText};
use steamtools::*;

mod window;
use window::{ModsPopup, ViewPopup, ViewState};

#[derive(Deserialize, Serialize, Debug, Default)]
enum State {
    #[default]    
    Setup,
    MainMenu,
    Settings,
}

const STEAM_BINARY_PATH: &str = "steam.bin";

// mod settings;
// use settings::Settings;

#[derive(Default)]
struct App {
    st: Steam,
    state: State,
    games: Arc<Mutex<Vec<Game>>>,
    cached_games: GameMap,
    loaded: bool,
    view: ViewPopup,
    mods: ModsPopup,
    // settings: Settings,
}

#[derive(Default)]
struct GameMap(pub HashMap<u32, Game>);

fn write_string(file: &mut impl Write, s: &str) -> io::Result<()> {
    file.write_all(&(s.len() as u32).to_le_bytes())?;
    file.write_all(s.as_bytes())?;
    Ok(())
}

fn read_string(reader: &mut impl Read, len_buf: &mut [u8; 4], buf: &mut Vec<u8>) -> io::Result<String> {
    reader.read_exact(len_buf)?;
    let len = u32::from_le_bytes(*len_buf) as usize;

    if buf.len() < len {
        buf.resize(len, 0);
    }

    reader.read_exact(&mut buf[..len])?;
    let s = String::from_utf8(buf[..len].to_vec()).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;

    Ok(s)
}

impl GameMap {
    fn write_to(file: &mut impl Write, map: HashMap<u32, &Game>) -> io::Result<()> {
        let mut writer = BufWriter::new(file);

        writer.write_all(&(map.len() as u32).to_le_bytes())?; // Length
        for (appid, game) in map {
            // APPID, Key
            writer.write_all(&appid.to_le_bytes())?;
            writer.write_all(&(game.installed as u8).to_le_bytes())?;

            // AppData
            write_string(&mut writer, &game.details.app_type)?;
            write_string(&mut writer, &game.details.name)?;
            write_string(&mut writer, &game.details.header_image)?;

            writer.write_all(&(game.details.is_free as u8).to_le_bytes())?;
            write_string(&mut writer, &game.path)?;

            writer.write_all(&(game.details.pc_requirements.len() as u32).to_le_bytes())?;
            for (key, s) in &game.details.pc_requirements {
                write_string(&mut writer, key)?;
                write_string(&mut writer, s)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn read_from(file: &mut impl Read) -> io::Result<HashMap<u32, Game>> {
        let mut reader = BufReader::new(file);
        let mut buf = [0u8; 4];
        let mut res = Vec::<u8>::with_capacity(512);
        reader.read_exact(&mut buf)?;
        let count = u32::from_le_bytes(buf); // Length
        
        let mut games = HashMap::<u32, Game>::new();
        for _ in 0..count {
            // Appid, Key
            reader.read_exact(&mut buf)?;
            let appid = u32::from_le_bytes(buf);

            reader.read_exact(&mut buf[0..1])?;
            let installed = buf[0] != 0;

            // AppData
            let app_type = read_string(&mut reader, &mut buf, &mut res)?;
            let name = read_string(&mut reader, &mut buf, &mut res)?;
            let header_image = read_string(&mut reader, &mut buf, &mut res)?;

            reader.read_exact(&mut buf[0..1])?;
            let is_free = buf[0] != 0;

            let path = read_string(&mut reader, &mut buf, &mut res)?;

            reader.read_exact(&mut buf)?;
            let count = u32::from_le_bytes(buf);
            let mut pc_requirements = HashMap::<String, String>::new();
            for _ in 0..count {
                let key = read_string(&mut reader, &mut buf,&mut res)?;
                let s = read_string(&mut reader, &mut buf,&mut res)?;
                pc_requirements.insert(key, s);
            }

            games.insert(appid, Game {
                appid: appid,
                installed,
                path,
                details: AppData { app_type, name, is_free, header_image, pc_requirements }
            });
        }

        Ok(games)
    }
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let mut app = App::default();
        if let Some(storage_ref) = cc.storage {
            storage_ref.get_string("steam" ).map(|s| {
                let mut p = PathBuf::new();
                app.st = serde_json::from_str::<Steam>(&s).unwrap();
                if app.st.cfg.to_string_lossy().is_empty() {
                    p.push(&app.st.path);
                    p.push("config");
                    p.push("stplug-in");
                    app.st.cfg = p;
                }
            });

            storage_ref.get_string("state" ).map(|state| {
                app.state = serde_json::from_str::<State>(&state).unwrap();
            });

            storage_ref.get_string("games" ).map(|games| {
                app.games = serde_json::from_str::<Arc<Mutex<Vec<Game>>>>(&games).unwrap();
                app.loaded = true;
            });
        }

        app
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("steam", serde_json::to_string(&self.st).unwrap());
        storage.set_string("state", serde_json::to_string(&self.state).unwrap());
        storage.set_string("games", serde_json::to_string(&self.games).unwrap());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.state {
            State::Setup => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Setup").font(FontId::proportional(20.0)));
                    });

                    ui.vertical_centered_justified(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Select Steam Path: ");
                            if ui.text_edit_singleline(&mut self.st.path).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.st.path = path.to_string_lossy().to_string();
                                }
                            }
                        });

                        ui.add_space(15.0);

                        if !self.st.path.is_empty() && ui.button(RichText::new("Validate").font(FontId::proportional(18.0))).clicked() {
                            let mut pt_bf = PathBuf::from(self.st.path.clone());
                            pt_bf.push("config");
                            pt_bf.push("stplug-in");
                            dbg!(&pt_bf);
                            if !pt_bf.exists() {
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Info)
                                    .set_title("Error")
                                    .set_description(format!("Steam or Steamtools is not installed in {}. Please choose a path where you installed Steam with steamtools.", self.st.path))
                                    .show();
                            } else {
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Info)
                                    .set_title("Info")
                                    .set_description(format!("Path for Steamtools successfully set to {}", self.st.path))
                                    .show();
                                self.state = State::MainMenu;
                            }
                        }
                    });
                });
            }
            State::Settings => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Settings").font(FontId::proportional(20.0)));
                    });
                });

                egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(5.0);
                        if ui.button(RichText::new("↩ Back").font(FontId::proportional(15.0))).clicked() {
                            self.state = State::MainMenu;
                        }
                        ui.add_space(5.0);
                    });
                });
            }
            State::MainMenu => {
                egui::TopBottomPanel::bottom("status_panel").max_height(30.0).show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Made by");
                        ui.hyperlink_to("RealViper", "https://github.com/RealViper8/Steamtools");
                        });
                    });
                });

                egui::Window::new("View").default_size([0.0, 0.0]).open(&mut self.view.active).show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("\u{1F3E0} Home").clicked() { self.view.state = ViewState::Main };
                            if ui.button("Requirements").clicked() { self.view.state = ViewState::MinimumRequirements };
                            // ui.button("Plugins");
                        });
                    });
                    match self.view.state {
                        ViewState::Main => {
                            ui.vertical_centered(|ui| {
                                ui.label(format!("APPID: {}", self.view.game_id));
                                ui.label(format!("is_free: {}", self.games.lock().unwrap().get(self.view.current_game).unwrap().details.is_free))
                            });
                        }

                        ViewState::MinimumRequirements => {
                            let document = scraper::Html::parse_fragment(&self.view.requirements);
                            let text = document.root_element().text().collect::<Vec<_>>().join("\n");
                            ui.add(egui::Label::new(RichText::new(&text)).wrap());
                        }
                    }
                });


                egui::Window::new("Mods").default_size([0.0, 0.0]).open(&mut self.mods.active).show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("APPID:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.st.mod_id)
                                .hint_text("Enter appid")
                                .char_limit(10),
                        );

                        if ui.button("Get").clicked() {
                            if self.cached_games.0.is_empty() && !Path::new(STEAM_BINARY_PATH).exists() {
                                let games_guard = self.games.lock().unwrap();
                                let mut stfile = File::create(STEAM_BINARY_PATH).unwrap();
                                dbg!(&games_guard.iter().map(|g| (g.appid, g)).collect::<HashMap<u32, &Game>>());
                                GameMap::write_to(&mut stfile, games_guard.iter().map(|g| (g.appid, g)).collect::<HashMap<u32, &Game>>()).unwrap();
                            } else if self.cached_games.0.is_empty() && Path::new(STEAM_BINARY_PATH).exists() {
                                let mut stfile = File::open(STEAM_BINARY_PATH).unwrap();
                                self.cached_games.0 = GameMap::read_from(&mut stfile).unwrap();
                            }

                            if let Err(_) = self.st.mod_id.parse::<u32>() {
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Warning)
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .set_title("Warning")
                                    .set_description("APPID should be numeric")
                                    .show();
                            } else {
                                if let Some(s) = self.cached_games.0.get(&self.st.mod_id.parse::<u32>().unwrap()) {
                                    install_melonloader(&s.path, self.st.melon_loader);
                                } else {
                                    rfd::MessageDialog::new()
                                        .set_title("Info")    
                                        .set_level(rfd::MessageLevel::Info)
                                        .set_buttons(rfd::MessageButtons::Ok)
                                        .set_description(format!("You dont have {}.", self.st.mod_id))
                                        .show();
                                }
                            }

                            // TODO: Add Game serialization to binary? 
                            // todo!("Cant get path without saving Game struct");
                        };
                    });
                });

                egui::TopBottomPanel::top("top").show(ctx, |ui| {
                    let width = ui.available_width();
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Steam Tools").font(FontId::proportional(20.0)));
                            
                            ui.add_space(15.0);

                            // egui::MenuBar::new().config(MenuConfig::new()).ui(ui, |ui| {
                            //     if ui.button("About").clicked() {
                            //         rfd::MessageDialog::new()
                            //             .set_title("About")
                            //             .set_description("So this tool is used with the other Steam Tool to give you more power.")
                            //             .set_buttons(rfd::MessageButtons::Ok)
                            //             .set_level(rfd::MessageLevel::Info).show();
                            //     }
                            // });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("⚙").strong().font(FontId::proportional(20.0))).clicked() {
                                    self.state = State::Settings;
                                }

                                if ui.button("\u{1F502} Mods").on_hover_text("Explore Mods").clicked() {
                                    self.mods.active = !self.mods.active;
                                }

                                if ui.button("\u{1F502} Fetch").on_hover_text("Fetch manually in case it doesnt Update the List automatically").clicked() {
                                    self.loaded = false;
                                }
                            });
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_sized([width * 0.4, 25.0], egui::Label::new("Header Image"));
                            ui.add_sized([width * 0.27, 25.0], egui::Label::new("Game"));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_sized([width * 0.3, 25.0], egui::Label::new("Tools"));
                            });
                        });
                    });
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical(|ui| {
                        let width = ui.available_width();
                        let height = ui.available_height();
                        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                            egui::Grid::new("games").striped(false).show(ui, |ui| {
                                for (i, game) in self.games.lock().unwrap().iter().enumerate() {
                                   ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| { 
                                        ui.add(
                                            egui::Image::new(format!("file://icons/{}.jpg", game.appid))
                                                .fit_to_exact_size(egui::vec2(width * 0.4, height * 0.4))
                                        );
                                        ui.add_sized([width * 0.3, height * 0.3], egui::Label::new(RichText::new(&game.details.name).strong()).wrap());

                                        ui.vertical(|ui| {
                                            let height = ui.available_height();
                                            if ui.add_sized([width * 0.3, height * 0.3], egui::Button::new(RichText::new("Remove").strong())).on_hover_text("Removes the game from your Account").clicked() {
                                                let mut p = PathBuf::from(&self.st.path);
                                                p.push("config");
                                                p.push("stplug-in");
                                                p.push(format!("{}.lua", game.appid));
                                                if fs::remove_file(p).is_err() {
                                                    rfd::MessageDialog::new()
                                                        .set_title("Error")
                                                        .set_description("Failed to delete")
                                                        .set_buttons(rfd::MessageButtons::Ok);
                                                }

                                                self.loaded = false;
                                            }

                                            if game.installed {
                                                if ui.add_sized([width * 0.3, height * 0.3], egui::Button::new(RichText::new("\u{1F5D1} Uninstall").strong().raised())).clicked() {
                                                    #[cfg(target_os="windows")]
                                                    process::Command::new("cmd").args(["/C", &format!("start steam://uninstall/{}", game.appid)]).spawn().expect("Failed to uninstall");
                                                }
                                            } else {
                                                if ui.add_sized([width * 0.3, height * 0.3], egui::Button::new(RichText::new("\u{2795} Install").strong().raised())).clicked() {
                                                    #[cfg(target_os="windows")]
                                                    process::Command::new("cmd").args(["/C", &format!("start steam://install/{}", game.appid)]).spawn().expect("Failed to install");
                                                }
                                            }

                                            if ui.add_sized([width * 0.3, height * 0.3], egui::Button::new(RichText::new("\u{1F50D} View").strong())).clicked() {
                                                self.view.requirements = game.details.pc_requirements.get("minimum").unwrap().to_string();
                                                self.view.game_id = game.appid;
                                                self.view.current_game = i;
                                                self.view.active = true;
                                            }
                                        });
                                        ui.add_space(55.0);
                                    });

                                    ui.end_row();
                                }
                            });
                        });
                    });
                });


                if !self.loaded {
                    let s = self.st.path.clone();
                    let games_arc = self.games.clone();
                    thread::spawn(move || {
                        let result = get_games(&s);
                        // dbg!(&result);
                        let mut games_lock = games_arc.lock().unwrap();
                        *games_lock = result;
                    });
                    self.loaded = true;
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default().with_taskbar(true).with_inner_size([520.0, 320.0]).with_min_inner_size([520.0, 320.0]).with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../icon.png")).unwrap()),
        ..Default::default()
    };

    eframe::run_native(
        "steamtools",
        options,
        Box::new(|cc| {
            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(cc)))
        })
    )
}