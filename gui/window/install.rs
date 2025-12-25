use crate::{stack_buffer, utils::stack::StackBuffer, window::WindowPopup};
use std::{fmt::Write, fs::File, io::Write as ioWrite, path::{MAIN_SEPARATOR, Path}};
use eframe::{egui::Window};

const MANIFESTS_URL: &str = "https://raw.githubusercontent.com/SteamAutoCracks/ManifestHub/refs/heads";

struct Handler(fn(&String, i32));

pub struct InstallPopup {
    pub active: bool,
    pub appid: String,
    handler: Handler
}

fn install(path: &String, appid: i32) {
    let mut sb = stack_buffer!(100); // Lua string can always be (max of) 100 bytes
    write!(sb, "{}/{}/{}.lua", &MANIFESTS_URL, appid, appid).unwrap();
    if let Ok(resp) = reqwest::blocking::get(sb.as_str()) {
        if !resp.status().is_success() {
            return;
        }

        // if Path::new(path)

        let mut sb = stack_buffer!(516);
        write!(sb, "{}{MAIN_SEPARATOR}config{MAIN_SEPARATOR}stplug-in{MAIN_SEPARATOR}{}.lua", path, appid).unwrap();
        if Path::new(sb.as_str()).exists() {
            rfd::MessageDialog::new()
                .set_title("Info")
                .set_description("You already have the game/app !")
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            return;
        }


        let mut file = File::create_new(sb.as_str()).unwrap();
        file.write_all(&resp.bytes().unwrap()).unwrap();
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
                    (app.install.handler.0)(&app.st.path, app.install.appid.parse::<i32>().unwrap())
                }

            });
        });
    }
}