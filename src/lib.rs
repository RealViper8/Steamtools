use log::{debug, error, info};
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, DirBuilder, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
// use crate::st::{Lua, init_lua};

// importing x32 mod
pub mod st;

// Can get ip timeouted if user requests too much !!!
pub const STEAM_URL: &str = "https://store.steampowered.com/api/appdetails?appids=";

// const STEAM_HEADER_URL: &str = "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/";

// Requires api key
pub const STEAM_APPLIST_URL: &str =
    "https://api.steampowered.com/IStoreService/GetAppList/v1/?key=";

#[cfg(target_os = "windows")]
const MELONLOADER_URL: &str = "https://github.com/LavaGang/MelonLoader/releases/download/v0.7.1/";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameDetails {
    success: bool,
    data: Option<AppData>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppData {
    #[serde(rename = "type")]
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

#[must_use]
pub fn install_melonloader(path: &str, melon_loader: bool) -> Option<()> {
    if melon_loader {
        if Path::new("MelonLoader").exists() {
            if Command::new("cmd")
                .args(["/C", ".\\MelonLoader\\Loader.exe"])
                .spawn()
                .and_then(|mut child| child.wait())
                .is_err()
            {
                error!("Starting melon loader failed.");
            }
        } else {
            if let Err(e) = DirBuilder::new().create("MelonLoader") {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_description(e.to_string())
                    .set_title("Error")
                    .show();
                return None;
            }
            #[cfg(target_os = "windows")]
            {
                std::thread::spawn(|| {
                    let Ok(bytes) =
                        blocking::get(format!("{MELONLOADER_URL}MelonLoader.Installer.exe"))
                            .and_then(blocking::Response::bytes)
                    else {
                        return;
                    };
                    let Ok(mut file) = File::create("MelonLoader/Loader.exe") else {
                        error!("Failed to install melon loader!");
                        return;
                    };
                    _ = file.write_all(&bytes);
                    _ = file.flush();
                    if Command::new("cmd")
                        .args(["/C", ".\\MelonLoader\\Loader.exe"])
                        .spawn()
                        .and_then(|mut child| child.wait())
                        .is_err()
                    {
                        error!("Starting melon loader failed.");
                    }
                });
            };
        }
    }

    let mods_path = format!("{path}\\Mods");
    DirBuilder::new().recursive(true).create(&mods_path).ok()?;

    let Ok(m) = fs::read_dir("mods") else {
        rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Error")
                .set_description("For now only local mods are supported create a folder in steamtools named mods and drop your MelonLoader (.dll) into! Example: GameName.dll")
                .show();
        return None;
    };

    for m in m {
        let entry = match m {
            Ok(e) => e,
            Err(e) => {
                println!("ERROR: {e}");
                continue;
            }
        };

        let pathb = entry.path();
        if !pathb.is_file() {
            continue;
        }

        dbg!(&PathBuf::from(path).file_name()?.to_str()?);
        if pathb.file_stem()?.to_str()? == PathBuf::from(path).file_name()?.to_str()? {
            fs::copy(
                pathb.file_name()?,
                format!("{}\\{}", mods_path, pathb.file_name()?.display()),
            )
            .ok()?;
        }
    }

    Some(())
}

fn setup_icons(icons: &mut Option<HashMap<u32, PathBuf>>) {
    if Path::new("icons").exists() {
        *icons = Some(HashMap::new());

        if let Ok(entries) = fs::read_dir("icons") {
            for (key, path) in entries.filter_map(Result::ok).filter_map(|entry| {
                let path = entry.path();
                let key = path.file_stem()?.to_str()?.parse::<u32>().ok()?;
                Some((key, path))
            }) {
                if let Some(i) = icons.as_mut() {
                    i.insert(key, path);
                }
            }
        }
    }
}

#[allow(clippy::too_many_lines, clippy::implicit_hasher)]
/// # Errors
/// Can fail for many reasons
pub fn get_games(
    path: impl Into<PathBuf> + Copy,
    current_games: HashMap<u32, Game>,
) -> io::Result<HashMap<u32, Game>> {
    let mut p = path.into();
    p.push("config");
    p.push("stplug-in");

    let mut gp = path.into();
    gp.push("steamapps");

    if !gp.exists() {
        fs::create_dir(&gp)?;
    }

    let mut games: HashMap<u32, Game> = current_games;

    let entries = match fs::read_dir(p) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Directory doesnt exist. {e}");
            return Ok(games);
        }
    };

    let mut name: HashMap<u32, String> = HashMap::new();

    let installed: HashMap<u32, String> = match fs::read_dir(&gp) {
        Ok(entries) => entries,
        Err(e) => {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .set_title("Error")
                .set_description(e.to_string())
                .show();
            return Ok(games);
        }
    }
    .filter_map(Result::ok)
    .filter(|f| f.path().is_file())
    .filter_map(|entry| {
        let fname = entry.file_name().into_string().ok()?;
        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        if fname.starts_with("appmanifest_") && fname.ends_with(".acf") {
            debug!("Game found: {fname}");
            let id_part =
                fname.get("appmanifest_".len()..fname.len().checked_sub(".acf".len())?)?;
            let id = id_part.parse::<u32>().ok()?;
            let mut file_ptbuf = gp.clone();
            file_ptbuf.push(&fname);

            let text = fs::read_to_string(file_ptbuf).ok()?;

            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("\"name\"")
                    && let Some((_, value)) = line.split_once('"')
                    && let Some((_, value)) = value.split_once('"')
                {
                    name.insert(
                        id,
                        format!(
                            "{}\\steamapps\\common\\{}",
                            Into::<PathBuf>::into(path).display(),
                            value.trim().get(1..value.len().checked_sub(3)?)?
                        ),
                    );
                }
            }
            Some((id, fname))
        } else {
            None
        }
    })
    .collect();

    debug!("Installed Games: {installed:#?}");

    let mut icons: Option<HashMap<u32, PathBuf>> = None;
    setup_icons(&mut icons);

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[ERROR] {e}");
                continue;
            }
        };

        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let appid = path.file_stem().unwrap_or_default();
        let Ok(appid_i) = appid.to_string_lossy().parse::<u32>() else {
            rfd::MessageDialog::new()
                .set_description(format!(
                    "Failed to parse {} please use appid. Skipping entry",
                    appid.to_string_lossy()
                ))
                .set_buttons(rfd::MessageButtons::Ok);
            continue;
        };

        if let Some(ic) = icons.as_ref()
            && ic.contains_key(&appid_i)
        {
            continue;
        }

        let url = format!("{}{}", STEAM_URL, appid.display());
        info!("Fetching {}", appid.display());
        debug!("Fetching image: {url}");
        let Ok(r) = blocking::get(&url).inspect_err(|e| error!("GET {url}: {e}")) else {
            continue;
        };

        let Ok(resp) = r
            .json::<HashMap<String, GameDetails>>()
            .inspect_err(|e| error!("JSON {url}: {e}"))
        else {
            continue;
        };

        let installed_val: bool = installed.contains_key(&appid_i);

        let g = Game::new()
            .with_appid(
                appid
                    .to_str()
                    .ok_or(io::ErrorKind::Other)?
                    .parse::<u32>()
                    .unwrap_or_default(),
            )
            .with_details(&resp, appid_i)
            .with_path_installed(installed_val, |i, g| {
                if i {
                    g.path
                        .clone_from(name.get(&appid_i).ok_or(io::ErrorKind::Other)?);
                }
                Ok(())
            })?;

        games.insert(appid_i, g);
    }

    Ok(games)
}

impl Game {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// # Errors
    /// Will return `Err` if the index is bigger than the lenght of the hashmap
    pub fn with_path_installed(
        mut self,
        installed: bool,
        f: impl FnOnce(bool, &mut Self) -> io::Result<()>,
    ) -> io::Result<Self> {
        f(installed, &mut self)?;
        self.installed = installed;
        Ok(self)
    }

    #[must_use]
    pub fn with_details(mut self, resp: &HashMap<String, GameDetails>, appid: u32) -> Self {
        self.details = if let Some(r) = resp.get(&appid.to_string())
            && let Some(data) = &r.data
        {
            if !Path::new(&format!("icons/{appid}.jpg")).exists() {
                debug!("Image Asset: {} done", data.name);
                DirBuilder::new().recursive(true).create("icons").ok();

                if let Ok(s) = blocking::get(&data.header_image)
                    && let Ok(bytes) = s.bytes()
                    && let Ok(mut file) = File::create(format!("icons/{appid}.jpg"))
                    && file.write_all(&bytes).is_err()
                {
                    _ = file.flush();
                }
            }
            data.clone()
        } else {
            AppData::default()
        };

        self
    }

    #[must_use]
    pub const fn with_appid(mut self, appid: u32) -> Self {
        self.appid = appid;
        self
    }
}

impl Steam {
    #[must_use]
    pub fn new(path: Option<impl Into<String> + AsRef<str>>) -> Self {
        let mut steam = Self {
            path: String::new(),
            cfg: PathBuf::new(),
            ..Default::default()
        };
        if let Some(p) = path {
            steam.path = p.into();
        } else {
            #[cfg(target_os = "windows")]
            {
                steam.path = String::from("C:\\Program Files (x86)\\Steam");
            }
            #[cfg(target_os = "macos")]
            {
                steam.path = String::from("~/Library/Application Support/Steam");
            }
            #[cfg(target_os = "linux")]
            {
                steam.path = String::from("~/.local/share/Steam");
            }
        }

        steam
    }
}
