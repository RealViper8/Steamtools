use std::collections::HashMap;
use std::fs::{self, DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use reqwest::blocking;
use serde::{Serialize, Deserialize};
use log::{debug, info, error};
// use crate::st::{Lua, init_lua};

pub mod st;

// Can get ip timeouted if user requests too much !!!
pub const STEAM_URL: &str = "https://store.steampowered.com/api/appdetails?appids=";

// const STEAM_HEADER_URL: &str = "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/";

// Requires api key
pub const STEAM_APPLIST_URL: &str = "https://api.steampowered.com/IStoreService/GetAppList/v1/?key=";

const MELONLOADER_URL: &str = "https://github.com/LavaGang/MelonLoader/releases/download/v0.7.1/";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameDetails {
   success: bool,
   data: Option<AppData>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppData {
    #[serde(rename="type")]
   pub app_type: String, 
   pub name: String, 
   pub header_image: String,
//    pub pc_requirements: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Game {
    pub appid: u32,
    pub details: AppData,
    pub installed: bool,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Steam {
    pub path: String,
    pub mod_id: String,
    pub cfg: PathBuf,
    pub melon_loader: bool,
}

pub fn install_melonloader(path: &str, melon_loader: bool) -> Option<()> {
    if melon_loader {
        if !Path::new("MelonLoader").exists() {
            if let Err(e) = DirBuilder::new().create("MelonLoader") {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_description(e.to_string())
                    .set_title("Error")
                    .show();
                return None;
            }
            #[cfg(target_os="windows")]
            {
                std::thread::spawn(|| {
                    let bytes = blocking::get(format!("{}MelonLoader.Installer.exe", MELONLOADER_URL)).ok().unwrap().bytes().unwrap_or_default();
                    let mut file = File::create("MelonLoader/Loader.exe").unwrap();
                    file.write_all(&bytes).unwrap();
                    file.flush().unwrap();
                    Command::new("cmd").args(["/C", ".\\MelonLoader\\Loader.exe"]).spawn().expect("Failed to open MelonLoader.");
                });
            };
        } else {
            Command::new("cmd").args(["/C", ".\\MelonLoader\\Loader.exe"]).spawn().expect("Failed to open MelonLoader.");
        }
    }

    let mods_path = format!("{}\\Mods", path);
    DirBuilder::new()
        .recursive(true)
        .create(&mods_path)
        .unwrap();

    let m = match fs::read_dir("mods") {
        Ok(entries) => entries,
        Err(_) => {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Error")
                .set_description("For now only local mods are supported create a folder in steamtools named mods and drop your MelonLoader (.dll) into! Example: GameName.dll")
                .show();
            return None;
        }
    };

    for m in m {
        let entry = match m {
            Ok(e) => e,
            Err(e) => {
                println!("ERROR: {}",e );
                continue;
            }
        };

        let pathb = entry.path();
        if !pathb.is_file() {
            continue;
        }

        dbg!(&PathBuf::from(path).file_name().unwrap().to_str().unwrap());
        if pathb.file_stem().unwrap().to_str().unwrap() == PathBuf::from(path).file_name().unwrap().to_str().unwrap() {
            fs::copy(pathb.file_name().unwrap(), format!("{}\\{}", &mods_path, pathb.file_name().unwrap().display())).ok()?;
        }
    }

    Some(())
}

#[must_use]
pub fn get_games(path: impl Into<PathBuf> + Copy, current_games: HashMap<u32, Game>) -> HashMap<u32, Game> {
    let mut p = path.into();
    p.push("config");
    p.push("stplug-in");

    let mut gp = path.into();
    gp.push("steamapps");

    let mut games: HashMap<u32, Game> = current_games;

    let entries = match fs::read_dir(p) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Directory doesnt exist. {}", e);
            return games;
        }
    };

    let mut name: HashMap<u32, String> = HashMap::new();

    let installed: HashMap<u32, String> = match fs::read_dir(&gp) {
        Ok(entries) => entries,
        Err(e) => {
            rfd::MessageDialog::new().set_level(rfd::MessageLevel::Error).set_buttons(rfd::MessageButtons::Ok).set_title("Error").set_description(e.to_string()).show();
            return games;
        }
    }.filter_map(|res| res.ok())
    .filter(|f| f.path().is_file())
    .filter_map(|entry| {
        let fname = entry.file_name().into_string().ok()?;
        if fname.starts_with("appmanifest_") && fname.ends_with(".acf") {
            debug!("Game found: {}", &fname);
            let id_part = &fname["appmanifest_".len()..fname.len() - ".acf".len()];
            let id = id_part.parse::<u32>().unwrap();
            let mut file_ptbuf = gp.to_path_buf();
            file_ptbuf.push(&fname);

            let text = fs::read_to_string(file_ptbuf).unwrap();

            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("\"name\"") {
                    if let Some((_, value)) = line.split_once('"') {
                        if let Some((_, value)) = value.split_once('"') {
                            name.insert(id, format!("{}\\steamapps\\common\\{}", Into::<PathBuf>::into(path).display(), value.trim()[1..value.len() - 3].to_string()));
                        }
                    }
                }
            }
            Some((id, fname))
        } else {
            None
        }
    })
    .collect();

    debug!("Installed Games: {:#?}", installed);

    let mut icons: Option<HashMap<u32, PathBuf>> = None;
    if Path::new("icons").exists() {
        icons = Some(HashMap::new());

        for entry in fs::read_dir("icons").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            icons.as_mut().unwrap().insert(path.file_stem().unwrap().to_str().unwrap().parse::<u32>().unwrap(), path);
        }
    }

    'entries: for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.is_file() {
            let appid = path.file_stem().unwrap();
            let appid_i = match appid.to_string_lossy().parse::<u32>() {
                Ok(i) => i,
                Err(_) => {
                    rfd::MessageDialog::new().set_description(format!("Failed to parse {} please use appid. Skipping entry", appid.to_string_lossy()))
                        .set_buttons(rfd::MessageButtons::Ok);
                    continue 'entries;
                }
            };

            if let Some(ic) = icons.as_ref() {
                if ic.contains_key(&appid_i) {
                    continue 'entries;
                }
            }

            let url = format!("{}{}", STEAM_URL, appid.display());
            info!("Fetching {}", appid.display());
            debug!("Fetching image: {}", &url);
            let resp: HashMap<String, GameDetails> = match blocking::get(url).ok().unwrap().json() {
                Ok(r) => r,
                Err(e) => {
                    error!("Fetching: {e}");
                    continue;
                }
            };

            let installed_val: bool = installed.contains_key(&appid_i);

            games.insert(appid_i, Game {
                appid: appid.to_string_lossy().to_string().parse::<u32>().unwrap(),
                details: if let Some(r) = resp.get(&appid.to_string_lossy().to_string()) && let Some(data) = &r.data{
                        if !Path::new(&format!("icons/{}.jpg", appid.display())).exists() {
                            debug!("Image Asset: {} done", data.name);
                            DirBuilder::new().recursive(true).create("icons").unwrap();
                            let bytes = blocking::get(&data.header_image).unwrap().bytes().unwrap_or_default();
                            let mut file = File::create(format!("icons/{}.jpg", appid.display())).unwrap();
                            file.write_all(&bytes).unwrap();
                        }
                        data.clone()
                    } else {
                        AppData::default()
                    },
                path: if installed_val {
                    name.get(&appid_i).unwrap().to_string()
                } else {
                    String::new()
                },
                installed: installed_val,
                ..Default::default() 
            });
        }
    }

    games
}

impl Game {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Steam {
    #[must_use]
    pub fn new(path: Option<impl Into<String> + AsRef<str>>) -> Self {
        let mut steam = Self { path: String::new(), cfg: PathBuf::new(), ..Default::default() };
        if let Some(p) = path {
            steam.path = p.into();
        } else {
            #[cfg(target_os="windows")] { steam.path = String::from("C:\\Program Files (x86)\\Steam"); }
            #[cfg(target_os="macos")] { steam.path = String::from("~/Library/Application Support/Steam"); }
            #[cfg(target_os="linux")] { steam.path = String::from("~/.local/share/Steam"); }
        }

        steam
    }
}