use std::{path::{Path, PathBuf}, sync::{Arc, Mutex}, thread};

use egui_extras::install_image_loaders;
use serde::{Serialize, Deserialize};
use eframe::egui::{self, FontId, RichText};
use steamtools::*;


// #[cfg(feature="gui")]
// fn main() {
//     println!("Burger");
// }

// #[cfg(not(feature="gui"))]

#[derive(Deserialize, Serialize, Debug, Default)]
enum State {
    #[default]    
    Setup,
    MainMenu,
    Settings,
}

#[derive(Default)]
struct App {
    st: Steam,
    state: State,
    games: Arc<Mutex<Vec<Game>>>,
    loaded: bool,
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
                app.st = serde_json::from_str::<Steam>(&s).unwrap();
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
                egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if !self.loaded {
                            ui.label("[ASSETS] Downloading...");
                        }
                    });
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    // ui.vertical(|ui| {
                    //     ui.vertical_centered(|ui| {
                    //         ui.horizontal_top(|ui| {
                    //             ui.button("Burg");
                    //             ui.label(RichText::new("SteamTools").font(FontId::proportional(20.0)));
                    //         });
                    //     });

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Steam Tools").font(FontId::proportional(20.0)));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("⚙").font(FontId::proportional(20.0))).clicked() {
                                    self.state = State::Settings;
                                }

                                if ui.button("Fetch").on_hover_text("Fetch manually in case it doesnt Update the List automatically").clicked() {
                                    self.loaded = false;
                                }
                            });
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label("Header Image");

                            ui.add_space(35.0);
                            ui.label("Game");


                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label("Tools");
                            });
                        });

                        ui.separator();

                        let width = ui.available_width();
                        let height = ui.available_height();
                        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                            // egui::Grid::new("games").striped(false).show(ui, |ui| {
                            //     for game in self.games.lock().unwrap().iter() {
                            //         // dbg!(format!("./icons/{}.jpg", game.appid));
                            //         // ui.image("./icons/242760.jpg");
                            //         ui.horizontal(|ui| {
                            //             ui.add_sized([width * 0.4, height * 0.5],
                            //                 egui::Image::new(format!("file://icons/{}.jpg", game.appid))
                            //                     // .fit_to_exact_size(egui::vec2(150.0, 150.0))
                            //             );
                            //             // ui.label(&game.details.name);
                            //             ui.add_sized([width * 0.3, height * 0.3], egui::Label::new(&game.details.name));
                            //             ui.add_sized([width * 0.3, 20.0], egui::Button::new("Delete"));
                            //         });


                            //         ui.end_row();
                            //     }
                            // });
                            for game in self.games.lock().unwrap().iter() {
                                ui.horizontal(|ui| {
                                    ui.add_sized([width * 0.4, height * 0.8],
                                        egui::Image::new(format!("file://icons/{}.jpg", game.appid))
                                    );

                                    ui.add_sized(
                                        [width * 0.2, height * 0.2],
                                        egui::Label::new(&game.details.name).wrap()
                                    );

                                    ui.button("Delete");
                                });

                                ui.separator();
                            }
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
        viewport: egui::ViewportBuilder::default().with_taskbar(true).with_always_on_top().with_inner_size([520.0, 320.0]).with_min_inner_size([520.0, 320.0]),
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