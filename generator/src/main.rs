use serde_json;
use std::{
    collections::HashMap,
    io::{Write, stdin, stdout},
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use steam_vdf_parser::parse_text;

#[derive(Deserialize, Serialize, Default, Debug)]
struct Config {
    path: String,
    keys: DecryptionKey,
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct DecryptionKey {
    content: HashMap<u64, String>,
}

#[derive(Default, Debug)]
struct LuaGenerator {
    id: u64,
    source: String,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    steam_path: Option<String>,

    #[arg(short, long)]
    appid: Option<u64>,

    #[arg(short, long, default_value_t = false)]
    get_keys: bool,
}

impl LuaGenerator {
    fn begin(&mut self, id: u64) {
        self.id = id;
        self.source.push_str(&format!("addappid({id})\n"));
    }

    fn addgame(&mut self, id: u64, key: String) {
        self.source.push_str("addappid(");
        self.source.push_str(&id.to_string());
        self.source.push_str(", 1, ");
        self.source.push('"');
        self.source.push_str(&key);
        self.source.push('"');
        self.source.push_str(")\n");
    }

    fn save(&mut self) {
        fs::write(format!("{}.lua", self.id.to_string()), self.source.clone()).unwrap();
        self.source = String::new();
        self.id = 0;
    }
}

const SAVE_PATH: &'static str = "conf.toml";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn save(conf: &Config, path: &str) -> Option<()> {
    fs::write(path, toml::to_string(&conf).ok()?).ok()?;
    Some(())
}

fn read(path: &str) -> Option<Config> {
    let s = fs::read_to_string(path).ok()?;
    let conf: Config = toml::from_str(&s).ok()?;
    Some(conf)
}

fn get_decryption_keys_cli(keys: HashMap<u64, String>) {
    if let Ok(json) = serde_json::to_string(&keys) {
        println!("{}", json);
    } else {
        exit(1);
    }
}

fn main() {
    let args = Args::parse();
    let cli = args.steam_path.is_some() && args.appid.is_some();

    if !cli {
        println!("-- Lua Generator V{VERSION} --");
    }

    let mut config = Config::default();

    let mut pb = PathBuf::new();

    if !cli {
        if !Path::new("conf.toml").exists() {
            let mut path = String::new();
            write!(stdout(), "Steam path: ").unwrap();
            stdout().flush().unwrap();

            stdin().read_line(&mut path).unwrap();
            pb.push(path.trim_end());
            pb.push("config");

            if !pb.exists() {
                eprintln!("steamepath '{}/config' doesnt exist.", path.trim());
                return;
            }

            config.path = pb.display().to_string();
            save(&config, SAVE_PATH);
        } else {
            config = read("conf.toml").unwrap();
        }
    }

    if cli {
        pb.push(args.steam_path.unwrap());
        pb.push("config");
        pb.push("config.vdf");
    } else {
        pb.push(&config.path);
        pb.push("config.vdf");
    }

    let input = fs::read_to_string(&pb).unwrap();
    pb.pop();
    let vdf = parse_text(&input).unwrap();

    let depots = vdf
        .get_path(&["Software", "Valve", "Steam", "depots"])
        .and_then(|value| value.as_obj())
        .ok_or("depots object not found")
        .unwrap();

    let mut depots_map = HashMap::<u64, String>::new();

    for (depot_id, depot_value) in depots.iter() {
        let Ok(depot_id) = depot_id.parse::<u64>() else {
            continue;
        };

        let key = depot_value
            .get("DecryptionKey")
            .and_then(|value| value.as_str());

        if let Some(key) = key {
            depots_map.insert(depot_id, key.to_owned());
        }
    }

    config.keys.content = depots_map;
    if args.get_keys {
        get_decryption_keys_cli(config.keys.content);
        return;
    }

    if !cli {
        save(&config, SAVE_PATH);
    }

    let mut buf = String::new();

    if cli {
        buf = args.appid.unwrap().to_string();
    } else {
        stdout().write(b"Appid: ").unwrap();
        stdout().flush().unwrap();
        stdin().read_line(&mut buf).unwrap();
    }

    let id = if let Ok(id) = buf.trim().parse::<u64>() {
        id
    } else {
        println!("Make sure that the appid is numeric!");
        return;
    };

    let mut luagen = LuaGenerator::default();
    luagen.begin(id);

    let v = config
        .keys
        .content
        .keys()
        .filter(|x| {
            x.to_string()
                .starts_with(&buf.trim()[..buf.trim().len() - 1])
        })
        .collect::<Vec<_>>();

    if v.is_empty() {
        println!(
            "No keys have been found.\nMake sure you have the game installed and the appid is correct!"
        );
        return;
    }

    for key in v {
        luagen.addgame(*key, config.keys.content.get(key).unwrap().to_string());
    }

    luagen.save();

    if !cli {
        println!("Successfully created '{id}.lua'.\nHappy Sharing/Gaming");
    }
}
