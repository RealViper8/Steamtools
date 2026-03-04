use crate::{stack_buffer, utils::stack::StackBuffer, window::WindowPopup};
use std::{fmt::Write, fs::File, io::{self, Error, ErrorKind, Write as ioWrite}, path::{MAIN_SEPARATOR, Path}};
use eframe::{egui::Window};
use log::debug;

const MANIFESTS_URL: &str = "https://raw.githubusercontent.com/SteamAutoCracks/ManifestHub/refs/heads";

struct Handler(fn(&String, i32) -> io::Result<()>);

pub struct InstallPopup {
    pub active: bool,
    pub appid: String,
    handler: Handler
}

fn install(path: &String, appid: i32) -> io::Result<()> {
    let mut sb = stack_buffer!(100); // Lua string can always be (max of) 100 bytes
    write!(sb, "{}/{}/{}.lua", &MANIFESTS_URL, appid, appid).unwrap();

    let resp = reqwest::blocking::get(sb.as_str()).map_err(|_| Error::new(ErrorKind::ConnectionAborted, "Invalid url"))?;
    if !resp.status().is_success() {
        return Err(io::Error::new(io::ErrorKind::ConnectionRefused, "Response was an error"));
    }

    let mut sb = stack_buffer!(516);
    write!(sb, "{}{MAIN_SEPARATOR}config{MAIN_SEPARATOR}stplug-in{MAIN_SEPARATOR}{}.lua", path, appid).unwrap();
    if Path::new(sb.as_str()).exists() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Game already exists!"));
    }

    let mut file = File::create_new(sb.as_str())?;
    file.write_all(&resp.bytes().unwrap())?;

    debug!("Downloaded lua file: {}", sb.as_str());
    rfd::MessageDialog::new()
        .set_title("Success")
        .set_description("Lua file founded and downloaded!")
        .show();

    Ok(())
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

                if ui.button("Download").clicked() && !app.install.appid.is_empty() {
                    if let Err(error) = (app.install.handler.0)(&app.st.path, app.install.appid.parse::<i32>().unwrap()) {
                        match error.kind() {
                            ErrorKind::AlreadyExists => {
                                rfd::MessageDialog::new()
                                    .set_title("Info")
                                    .set_level(rfd::MessageLevel::Info)
                                    .set_description("You already have the game/app !")
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show();
                            },
                            ErrorKind::ConnectionRefused => {
                                rfd::MessageDialog::new()
                                    .set_title("Error")
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_description("Couldnt find app in database / invalid appid")
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show();
                            },
                            e => { dbg!(&e); },
                        }
                    } else {
                        // if its successfull trigger a "reload"
                        app.loaded = false;
                    }
                }

            });
        });
    }
}