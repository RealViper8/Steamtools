use std::fmt::Write;

use crate::{App, window::WindowPopup};
use eframe::egui;

#[derive(Debug, Default)]
pub enum ViewState {
    #[default]
    Main,
}

#[derive(Default)]
pub struct ViewPopup {
    pub active: bool,
    pub current_game: u32,
    pub state: ViewState,
}

impl WindowPopup for ViewPopup {
    fn view(app: &mut App, ui: &mut egui::Ui) {
        egui::Window::new("View")
            .default_size([0.0, 0.0])
            .open(&mut app.view.active)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("\u{1F3E0} Home").clicked() {
                            app.view.state = ViewState::Main
                        };
                    });
                });
                match app.view.state {
                    ViewState::Main => {
                        ui.vertical_centered(|ui| {
                            app.buffer.clear();
                            write!(&mut app.buffer, "APPID: {}", app.view.current_game).unwrap();
                            ui.label(&app.buffer);
                            app.buffer.clear();
                        });
                    }
                }
            });
    }
}
