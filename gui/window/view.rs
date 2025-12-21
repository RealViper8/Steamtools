use eframe::egui::{self, Context};
use crate::{App, window::WindowPopup};

#[derive(Debug, Default)]
pub enum ViewState {
    #[default]
    Main,
}

#[derive(Default)]
pub struct ViewPopup {
    pub active: bool,
    pub game_id: u32,
    pub current_game: usize,
    pub state: ViewState
}

impl WindowPopup for ViewPopup {
    fn view(app: &mut App, ctx: &Context) {
        egui::Window::new("View").default_size([0.0, 0.0]).open(&mut app.view.active).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("\u{1F3E0} Home").clicked() { app.view.state = ViewState::Main };
                });
            });
            match app.view.state {
                ViewState::Main => {
                    ui.vertical_centered(|ui| {
                        ui.label(format!("APPID: {}", app.view.game_id));
                        ui.horizontal(|ui| {
                            ui.label(format!("free: {}", app.games.lock().unwrap().get(app.view.current_game).unwrap().details.is_free));
                        })
                    });
                }
            }
        });
    }
}