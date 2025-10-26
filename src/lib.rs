use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::{self, DirBuilder, File};
use std::io::Write;
use std::fmt::Write as fmtWriter;
use std::path::{Path, PathBuf};
use reqwest::blocking;
use serde::{Serialize, Deserialize};


const STEAM_URL: &str = "https://store.steampowered.com/api/appdetails?appids=";

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
   pub is_free: bool, 
   pub header_image: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Game {
    pub appid: u32,
    pub details: AppData,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Steam {
    pub path: String,
    pub cfg: PathBuf,
}

pub fn get_games(path: impl Into<PathBuf>) -> Vec<Game> {
    let mut p = path.into();
    p.push("config");
    p.push("stplug-in");

    let mut games: Vec<Game> = Vec::new();

    let entries = match fs::read_dir(p) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Directory doesnt exist. {}", e);
            return games;
        }
    };

    for entry in entries {
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

            let url = format!("{}{}", STEAM_URL, appid.display());
            println!("[FETCHING] {}", appid.display());
            let resp: HashMap<String, GameDetails> = blocking::get(url).ok().unwrap().json().unwrap();

            games.push(Game {
                appid: appid.to_string_lossy().to_string().parse::<u32>().unwrap(),
                details: if let Some(r) = resp.get(&appid.to_string_lossy().to_string()) && let Some(data) = &r.data{
                        if !Path::new(&format!("icons/{}.jpg", appid.display())).exists() {
                            println!("[ASSETS] {}", data.name);
                            DirBuilder::new().recursive(true).create("icons").unwrap();
                            let bytes = blocking::get(&data.header_image).unwrap().bytes().unwrap_or_default();
                            let mut file = File::create(format!("icons/{}.jpg", appid.display())).unwrap();
                            file.write_all(&bytes).unwrap();
                        }
                        data.clone()
                    } else {
                        AppData::default()
                    },
                
                ..Default::default() 
            });
        }
    }

    games
}

impl Steam {
    pub fn new(path: Option<impl Into<String> + AsRef<str>>) -> Self {
        let mut steam = Self { path: String::new(), cfg: PathBuf::new() };
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