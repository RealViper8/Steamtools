use eframe::egui::{self, Context, RichText};

use crate::App;


#[derive(Debug, Default)]
pub enum ViewState {
    #[default]
    Main,
    MinimumRequirements,
}

#[derive(Default)]
pub struct ViewPopup {
    pub active: bool,
    pub requirements: String,
    pub game_id: u32,
    pub current_game: usize,
    pub state: ViewState
}

impl ViewPopup {
    pub fn window_view(app: &mut App, ctx: &Context) {
        egui::Window::new("View").default_size([0.0, 0.0]).open(&mut app.view.active).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("\u{1F3E0} Home").clicked() { app.view.state = ViewState::Main };
                    if ui.button("Requirements").clicked() { app.view.state = ViewState::MinimumRequirements };
                });
            });
            match app.view.state {
                ViewState::Main => {
                    ui.vertical_centered(|ui| {
                        ui.label(format!("APPID: {}", app.view.game_id));
                        ui.label(format!("is_free: {}", app.games.lock().unwrap().get(app.view.current_game).unwrap().details.is_free))
                    });
                }

                ViewState::MinimumRequirements => {
                    let document = scraper::Html::parse_fragment(&app.view.requirements);
                    let text = document.root_element().text().collect::<Vec<_>>().join("\n");
                    ui.add(egui::Label::new(RichText::new(&text)).wrap());
                }
            }
        });
    }
}