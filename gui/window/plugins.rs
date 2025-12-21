use std::{fs, ops::{Index, IndexMut}, path::Path, sync::{Arc, Mutex}, thread};

use eframe::egui::{self, Context, FontId, Label, RichText};
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use steamtools::st::{run_lua_file, start_file, stop_file};
use crate::{App, window::WindowPopup};

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

impl WindowPopup for Plugins {
    fn view(app: &mut App, ctx: &Context) {
        egui::Window::new("Plugins").default_size([0.0, 0.0]).open(&mut app.plugins.active).show(ctx, |ui| {
            if !app.plugins.fetched {
                if !Path::new("./plugins").exists() {fs::create_dir("./plugins").unwrap()}
                let dirs = fs::read_dir("./plugins").unwrap();
                for dir in dirs {
                    match dir {
                        Ok(d) => {
                            app.plugins.list.push(Plugin {
                                code: fs::read_to_string(d.path()).unwrap(),
                                name: Arc::new(Mutex::new(d.path().file_stem().unwrap().to_string_lossy().to_string())),
                                name_buffer: d.path().file_stem().unwrap().to_string_lossy().to_string(),
                                ..Default::default()
                            });
                        },
                        Err(_) => () 
                    }
                    app.plugins.fetched = true;
                }
            }

            if app.plugins.list.is_empty() {
                ui.centered_and_justified(|ui| {
                    if ui.button("Add new plugin").clicked() {
                        let mut dir = std::env::current_dir().unwrap();
                        dir.push("plugins");
                        let plugin = rfd::FileDialog::default()
                            .set_directory(dir)
                            .set_file_name("plugin.lua")
                            .add_filter("Lua File", &[".lua"])
                            .save_file();

                        if let Some(f) = plugin {
                            fs::File::create(f).unwrap();
                        }

                        app.plugins.fetched = false;
                    }
                });
                return;
            } 

            ui.vertical_centered(|ui| {
                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    for (i, plugin) in app.plugins.list.iter().enumerate() {
                        ui.group(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.add(Label::new(RichText::new(plugin.name.lock().unwrap().as_str()).font(FontId::proportional(20.0)).strong()).wrap_mode(egui::TextWrapMode::Truncate));
                            });
                            
                            if ui.button("View/Edit").clicked() {
                                app.plugins.selected_plugin = Some(i);
                                app.plugins.ceditor = true;
                            }

                            if ui.button("Start").clicked() {
                                // Resetting the static flag to 0 (internally)
                                start_file();
                                let pl = plugin.name.clone();
                                thread::spawn(move || {
                                    let pl_guard = pl.lock().unwrap();
                                    run_lua_file(format!("./plugins/{}.lua", *pl_guard));
                                });
                            }

                            if ui.button("Stop").clicked() {
                                stop_file();
                            }
                        });
                    }
                });
            });
        });

    }
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

    pub fn ceditor(app: &mut App, ctx: &Context) {
        if app.plugins.ceditor {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("immediate_code_editor"),
                egui::ViewportBuilder::default()
                    .with_title("Code Editor")
                    .with_inner_size([320.0, 150.0])
                    .with_min_inner_size([320.0, 150.0]),
                |ctx, _class| {
                egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let len = app.plugins.list.len();
                        if ui.button("\u{1F5D1}").clicked() {
                            if let Err(e) = fs::remove_file(format!("./plugins/{}.lua", app.plugins.list.remove(app.plugins.get().unwrap()).name.lock().unwrap())) {
                                eprintln!("{}", e);
                            }
                            if len == 1 {
                                app.plugins.ceditor = false;
                                return;
                            };
                        }

                        if ui.button("Save").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
                            fs::rename(format!("./plugins/{}.lua", &app.plugins.get_selected().unwrap().name.lock().unwrap()), format!("./plugins/{}.lua",&app.plugins.get_selected().unwrap().name_buffer)).ok();
                            fs::write(format!("./plugins/{}.lua", &app.plugins.get_selected().unwrap().name_buffer), app.plugins.get_selected().unwrap().code.as_bytes()).unwrap();
                            *app.plugins.list.get_mut(app.plugins.selected_plugin.unwrap()).unwrap().name.lock().unwrap() = app.plugins.get_selected().unwrap().name_buffer.clone();
                        }

                        ui.text_edit_singleline(&mut app.plugins.get_selected().unwrap().name_buffer);
                    });
                });

                if !app.plugins.ceditor {
                    return;
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                    let viewport_size = ctx.available_rect().size();
                    ui.vertical(|ui| {
                        let syntax: Syntax = Syntax::lua();
                        CodeEditor::default()
                            .with_syntax(syntax)
                            .with_theme(ColorTheme::GITHUB_DARK)
                            .desired_width(viewport_size.x)
                            .with_rows((viewport_size.y / 14.0) as usize)
                            .show(ui, &mut app.plugins.list[app.plugins.selected_plugin.unwrap()].code);
                    });
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    app.plugins.ceditor = false;
                }
            });
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