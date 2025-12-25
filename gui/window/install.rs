use crate::{stack_buffer, utils::stack::StackBuffer, window::WindowPopup};
use std::fmt::Write;
use eframe::egui::Window;

const MANIFESTS_URL: &str = "https://raw.githubusercontent.com/SteamAutoCracks/ManifestHub/refs/heads";

struct Handler(fn(i32));

pub struct InstallPopup {
    pub active: bool,
    pub appid: String,
    handler: Handler
}

fn install(appid: i32) {
    let mut sb = stack_buffer!(256);
    write!(sb, "{}/{}/{}.lua", &MANIFESTS_URL, appid, appid).unwrap();
    if let Ok(resp) = reqwest::blocking::get(sb.as_str()) {
        let b = resp.bytes().unwrap();
        let s = std::str::from_utf8(&b).unwrap_or("")
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();

        dbg!(s);
        dbg!(b);
    }
    dbg!(sb.as_str());
}

impl Default for InstallPopup {
    fn default() -> Self {
        Self {
            active: false,
            appid: String::new(),
            handler: Handler(install)
        }
    }
}

impl WindowPopup for InstallPopup {
    fn view(app: &mut crate::App, ctx: &eframe::egui::Context) {
        Window::new("Install").default_size([0.0, 0.0]).open(&mut app.install.active).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    // ui.label(RichText::new("Install").font(FontId::proportional(20.0)));
                    ui.label("Appid: ");
                    let response = ui.text_edit_singleline(&mut app.install.appid);
                    if response.changed() && !app.install.appid.chars().all(|c| c.is_ascii_digit()) {
                        app.install.appid.retain(|c| c.is_ascii_digit());
                    }
                });


                if ui.button("Download").clicked() {
                    (app.install.handler.0)(app.install.appid.parse::<i32>().unwrap())
                }

            });
        });
    }
}