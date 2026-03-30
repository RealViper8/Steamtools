#![cfg_attr(not(debug_assertions), windows_subsystem="windows")]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Write;
use std::process;
use std::{fs, path::PathBuf, sync::{Arc, Mutex}, thread};

use egui_extras::install_image_loaders;
use log::{warn, info, debug, error, trace};
use serde::{Serialize, Deserialize};
use eframe::egui::{self, FontData, FontDefinitions, FontId, RichText, Sense, UiBuilder, vec2};
use steamtools::{Game, Steam, get_games};

mod window;
use window::{ModsPopup, ViewPopup, InstallPopup, Settings, Plugins};

mod utils;
use utils::bserializer::GameMap;
use utils::filter::Filter;

use crate::window::WindowPopup;

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
enum State {
    #[default]    
    Setup,
    MainMenu,
    Settings,
}

const HOOK_DLL: &[u8] = include_bytes!("../deps/xinput1_4.dll");
const STEAM_BINARY_PATH: &str = "steam.bin";


#[derive(Default)]
struct App {
    st: Steam,
    settings: Settings,
    state: State,
    games: Arc<Mutex<HashMap<u32, Game>>>,
    cached_games: GameMap,
    loaded: bool,
    view: ViewPopup,
    install: InstallPopup,
    mods: ModsPopup,
    plugins: Plugins,
    unlock: bool,
    version: String,
    buffer: String,
    searchbar: RefCell<String>,
    filter: Filter,
    selected_game: Cell<u32>,
    delete_request: Option<u32>,
    // settings: Settings,
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "dejavu".to_owned(),
            Arc::new(FontData::from_static(include_bytes!("../fonts/DejaVuSans.ttf")))
        );
        fonts.families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "dejavu".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let mut app = App {
            buffer: String::with_capacity(512),
            ..Default::default()
        };

        if let Some(storage_ref) = cc.storage {
            storage_ref.get_string("version" ).map(|version| {
                app.version = serde_json::from_str::<String>(&version).unwrap();
                if VERSION != &app.version && let Some(dir) = eframe::storage_dir("steamtools") {
                    fs::remove_dir_all(dir).unwrap();
                    fs::remove_dir_all("icons").ok();
                    #[cfg(not(target_os = "windows"))]
                    rfd::MessageDialog::new()
                        .set_title("Info")
                        .set_buttons(rfd::MessageButtons::Ok)
                        .set_description("Restart Steamtools !")
                        .show();
                    #[cfg(target_os = "windows")] {
                        // let exe = std::env::current_exe().unwrap();
                        // #[cfg(debug_assertions)] {
                        //     std::process::Command::new(exe)
                        //         // .creation_flags(0x00000008) // DETACHED PROCESS
                        //         .spawn()
                        //         .ok();
                        // }

                        // #[cfg(not(debug_assertions))] {
                        //     use std::os::windows::process::CommandExt;
                        //     std::process::Command::new(exe)
                        //         .creation_flags(0x00000008) // DETACHED PROCESS
                        //         .spawn()
                        //         .ok();
                        // }
                        app.state = State::Setup;
                        rfd::MessageDialog::new()
                            .set_title("Info")
                            .set_buttons(rfd::MessageButtons::Ok)
                            .set_description(&format!("Updated to version: {VERSION}\nPlease setup the path for steam again !"))
                            .show();
                    }
                    // exit(0);
                } else {
                    storage_ref.get_string("state" ).map(|state| {
                        app.state = serde_json::from_str::<State>(&state).unwrap();
                    });
                }
            });

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


            storage_ref.get_string("games" ).map(|games| {
                app.games = serde_json::from_str::<Arc<Mutex<HashMap<u32, Game>>>>(&games).unwrap();
                app.loaded = true;
            });

            storage_ref.get_string("settings" ).map(|settings| {
                app.settings = serde_json::from_str(&settings).unwrap();
            });

            storage_ref.get_string("unlock" ).map(|unlock| {
                app.unlock = serde_json::from_str(&unlock).unwrap();
            });
        }

        #[cfg(target_os = "windows")]
        if app.st.path.is_empty() {
            app.st.path = windows_registry::LOCAL_MACHINE.open("software\\WOW6432Node\\Valve\\Steam").unwrap().get_string("InstallPath").unwrap();
        }

        app.version = VERSION.to_string();
        app
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("steam", serde_json::to_string(&self.st).unwrap());
        storage.set_string("version", serde_json::to_string(VERSION).unwrap());
        if self.state != State::Setup {
            storage.set_string("state", serde_json::to_string(&State::MainMenu).unwrap());
        } else {
            storage.set_string("state", serde_json::to_string(&self.state).unwrap());
        }
        storage.set_string("games", serde_json::to_string(&self.games).unwrap());
        storage.set_string("settings", serde_json::to_string(&self.settings).unwrap());
        storage.set_string("unlock", serde_json::to_string(&self.unlock).unwrap());
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
                            // pt_bf.push("config");
                            // pt_bf.push("stplug-in");
                            pt_bf.push("steam.exe");
                            info!("Steam path set to {}", &pt_bf.display());
                            if !pt_bf.exists() {
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Info)
                                    .set_title("Error")
                                    .set_description(format!("Steam is not installed in {}. Please choose a path where you installed Steam.", self.st.path))
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
                    ui.vertical(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Settings").font(FontId::proportional(20.0)));
                        });
                        ui.checkbox(&mut self.settings.mod_experimental, "Mods feature (Experimental)");
                        ui.checkbox(&mut self.settings.plugins_experimental, "Plugins feature (Experimental)")
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
                            ui.add_space((ui.available_width()/2.0)+85.0);
                            ui.label(format!("Version: {}", VERSION));
                        });

                    });
                });

                ViewPopup::view(self, ctx); 
                Plugins::view(self, ctx);
                Plugins::ceditor(self, ctx);
                ModsPopup::view(self, ctx);
                InstallPopup::view(self, ctx);

                egui::TopBottomPanel::top("top").show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(5.0);
                            ui.label(RichText::new("Steam Tools").font(FontId::proportional(24.0)));
                            ui.add_space(15.0);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("⚙").strong().font(FontId::proportional(20.0))).on_hover_text("Explore Settings").clicked() {
                                    self.state = State::Settings;
                                }

                                if self.settings.mod_experimental && ui.button("\u{1F502} Mods").on_hover_text("Explore Mods").clicked() {
                                    self.mods.active = !self.mods.active;
                                }

                                if self.settings.plugins_experimental && ui.button("\u{1F50C} Plugins").on_hover_text("Explore Plugins").clicked() {
                                    self.plugins.active = !self.plugins.active;
                                }

                                if ui.button("\u{2193} Install").on_hover_text("Downloads a lua file to use with st").clicked() {
                                    self.install.active = !self.install.active;
                                }

                                if ui.button("\u{1F502} Fetch").on_hover_text("Fetch manually in case it doesnt Update the List automatically").clicked() {
                                    self.plugins.fetched = false;
                                    self.loaded = false;
                                }

                                if ui.button("Load lua").on_hover_text("Loads a .lua file for game/dlcs").clicked() {
                                    // Lua files
                                    let files =
                                        rfd::FileDialog::default()
                                            .add_filter("lua", &["lua"])
                                            .set_title("Steamtools Lua")
                                            .pick_files();

                                    let mut path: PathBuf = PathBuf::new();
                                    path.push(&self.st.path);
                                    path.push("config\\stplug-in");
                                    match files {
                                        Some(ref files) => {
                                            files.iter().for_each(|file| {
                                                path.push(&file.file_stem().unwrap());
                                                fs::copy(file.as_path(),format!("{}.lua", &path.to_string_lossy())).unwrap();
                                            });
                                        },
                                        None => ()
                                    }

                                    if files.is_none() {
                                        return;
                                    }

                                    // for file in files.unwrap() {
                                    //     let games_lock = self.games.clone();
                                    //         match file.file_stem().unwrap().to_str().unwrap().parse::<u32>().ok() {
                                    //             Some(appid) => {
                                    //                 let appid = appid;
                                    //                 thread::spawn(move || {
                                    //                     let url = format!("{}{}", STEAM_URL, appid);
                                    //                     println!("[FETCHING] {}", appid);
                                    //                     dbg!(&url);
                                    //                     let resp: HashMap<String, GameDetails> = match blocking::get(url).ok().unwrap().json() {
                                    //                         Ok(r) => {
                                    //                             r
                                    //                         },
                                    //                         Err(e) => {
                                    //                             eprintln!("[FETCHING ERROR] {}", e);
                                    //                             return
                                    //                         }
                                    //                     };


                                    //                 });
                                    //             },
                                    //             None => println!("error: failed to read .lua make sure the filename is the appid !"),
                                    //         };
                                    //     }
                                    self.loaded = false;
                                }

                                if ui.checkbox(&mut self.unlock, "Unlock").changed() {
                                    if self.unlock {
                                        fs::write(format!("{}\\xinput1_4.dll", self.st.path), HOOK_DLL).unwrap();
                                    } else {
                                        fs::remove_file(format!("{}\\xinput1_4.dll", self.st.path)).unwrap();
                                    }
                                }
                            });
                        });

                        ui.add_space(3.0);

                        ui.horizontal(|ui| {
                            let searchbar = ui.add(egui::TextEdit::singleline(self.searchbar.get_mut())
                                .hint_text("Search")
                                .char_limit(50)
                                .background_color(egui::Color32::from_hex("#2a363a").unwrap())
                            );

                            if ui.button("\u{2715}").clicked() {
                                self.searchbar.get_mut().clear();
                                self.filter = Filter::None;
                                return;
                            }

                            if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                                searchbar.request_focus();
                            }

                            if searchbar.changed() {
                                if self.searchbar.borrow().is_empty() {
                                    self.filter = Filter::None;
                                    return;
                                }

                                match &self.searchbar.borrow().parse::<u32>() {
                                    Ok(id) => self.filter = Filter::Id(*id),
                                    Err(_) => {
                                        // Since it errors it will be a string !
                                        self.filter = Filter::Name(self.searchbar.borrow().clone());
                                    }
                                }
                            }
                        });
                        
                        ui.add_space(4.0);
                    });
                });

                egui::SidePanel::right("game_stats").exact_width(140.0).show_separator_line(true).resizable(false).show(ctx, |ui| {
                    let width = ui.available_width();
                    let height = ui.available_height();
                    ui.vertical_centered(|ui| {
                        let selected_game = self.selected_game.get();
                        if selected_game != 0 {
                            let game = self.games.lock().unwrap();
                            let game = game.get(&selected_game).unwrap();
                            ui.label(RichText::new(&game.details.name).font(FontId::new(18.0, egui::FontFamily::Proportional)));
                            ui.add_space(5.0);
                            ui.label(&format!("APPID: {}", game.appid));
                            ui.add_space(8.0);
                            ui.vertical_centered_justified(|ui| {
                                if game.installed {
                                    if ui.add_sized(vec2(50.0,25.0), egui::Button::new(RichText::new("\u{1F5D1} Uninstall").strong().raised())).on_hover_text("Prompts steam to uninstall the game").clicked() {
                                        self.buffer.clear();
                                        write!(&mut self.buffer, "start steam://uninstall/{}", game.appid).unwrap();
                                        #[cfg(target_os="windows")]
                                        process::Command::new("cmd").args(["/C", &self.buffer]).spawn().expect("Failed to uninstall");
                                        self.buffer.clear();
                                    }
                                } else {
                                    if ui.add_sized(vec2(50.0, 25.0), egui::Button::new(RichText::new("\u{2795} Install").strong().raised())).on_hover_text("Prompts steam to install the game").clicked() {
                                        self.buffer.clear();
                                        write!(&mut self.buffer, "start steam://install/{}", game.appid).unwrap();
                                        #[cfg(target_os="windows")]
                                        process::Command::new("cmd").args(["/C", &self.buffer]).spawn().expect("Failed to install");
                                        self.buffer.clear();
                                    }
                                }
                                ui.add_space(2.0);
                                if ui.add_sized([width * 0.3, height * 0.1], egui::Button::new(RichText::new("\u{1F50D} View").strong())).on_hover_text("View information about the game").clicked() {
                                    self.view.current_game = game.appid;
                                    self.view.active = true;
                                }
                                ui.add_space(2.0);
                                if ui.add_sized([width * 0.3, height * 0.1], egui::Button::new(RichText::new("Remove").strong())).on_hover_text("Removes the game from your Account").clicked() {
                                    let mut p = PathBuf::from(&self.st.path);
                                    p.push("config");
                                    p.push("stplug-in");
                                    self.buffer.clear();
                                    write!(&mut self.buffer, "{}.lua", &game.appid).unwrap();
                                    p.push(&self.buffer);
                                    self.buffer.clear();
                                    if fs::remove_file(&p).is_err() {
                                        rfd::MessageDialog::new()
                                            .set_title("Error")
                                            .set_description("Failed to delete")
                                            .set_buttons(rfd::MessageButtons::Ok);
                                    }

                                    p.clear();
                                    p.push("icons");
                                    p.push(format!("{}.jpg", &game.appid));
                                    debug!("Icon deleted: {}", &p.display());

                                    fs::remove_file(&p).ok();
                                    self.delete_request = Some(game.appid);
                                }
                            });
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                ui.wrap_mode();
                                ui.label("Please select a game first by clicking on it !");
                            });
                        }
                    });
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        ui.add_space(5.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                            let game_map = {
                                self.games.lock().unwrap().clone()
                            };

                            if let Some(s) = self.delete_request.take() {
                                self.selected_game.set(0);
                                self.games.lock().unwrap().remove(&s);
                                self.loaded = false;
                            }

                            for (id, _) in game_map.iter().filter(|(gid, g)| {
                                match &self.filter {
                                    Filter::Id(id) => id == *gid,
                                    Filter::Name(name) => g.details.name.to_lowercase().starts_with(&*name.to_lowercase()),
                                    Filter::None => true,
                                }
                            }) {
                                let width = 240.0;
                                let height = 112.0;
                                let (card_rect, card_resp) =
                                    ui.allocate_exact_size(vec2(width, height), Sense::hover());

                                self.buffer.clear();
                                write!(&mut self.buffer, "file://icons/{}.jpg", id).unwrap();

                                ui.scope_builder(
                                    UiBuilder::new().max_rect(card_rect),
                                    |ui| {
                                    ui.add(
                                    egui::Image::new(&self.buffer)
                                        .corner_radius(egui::CornerRadius::same(6))
                                        .fit_to_exact_size(egui::vec2(width, height))
                                    );
                                });

                                if card_resp.hovered() {
                                    ui.painter().rect_stroke(
                                        card_resp.rect.expand(4.0),
                                        10.0,
                                        egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
                                        egui::StrokeKind::Middle
                                    );
                                }

                                if card_resp.interact(Sense::click()).clicked() {
                                    self.selected_game.set(*id);
                                }
                            }
                        });
                        ui.add_space(5.0);
                    });
                });

                if !self.loaded {
                    ctx.request_repaint();
                    let s = self.st.path.clone();
                    let games_arc = self.games.clone();
                    thread::spawn(move || {
                        let mut sbin = fs::File::create(STEAM_BINARY_PATH).unwrap();

                        let current_games = {
                            games_arc.lock().unwrap().clone()
                        };
                        GameMap::write_to(&mut sbin, &current_games).unwrap();

                        let result = get_games(&s, current_games);

                        let mut games = games_arc.lock().unwrap();
                        *games = result;
                    });

                    self.loaded = true;
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    #[cfg(not(debug_assertions))] {
        use env_logger::Env;
        use std::fs::OpenOptions;
        let log_file = OpenOptions::new().create(true).append(true).open("steamtools.log").unwrap();
        env_logger::Builder::from_env(Env::default().default_filter_or("info,stcli=debug,steamtools=debug,wgpu=off"))
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .init();
    }

    #[cfg(debug_assertions)] {
        use env_logger::Env;
        env_logger::Builder::from_env(Env::default().default_filter_or("info,stcli=debug,steamtools=debug,wgpu=off"))
            .init();
    }

    info!("GUI: Initializing");

    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default().with_taskbar(true).with_inner_size([650.0, 370.0]).with_min_inner_size([650.0, 370.0]).with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../icon.png")).unwrap()),
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