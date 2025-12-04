use std::{collections::HashMap, io::{self, BufReader, BufWriter, Read, Write}};
use steamtools::{AppData, Game};

fn write_string(file: &mut impl Write, s: &str) -> io::Result<()> {
    file.write_all(&(s.len() as u32).to_le_bytes())?;
    file.write_all(s.as_bytes())?;
    Ok(())
}

fn read_string(reader: &mut impl Read, len_buf: &mut [u8; 4], buf: &mut Vec<u8>) -> io::Result<String> {
    reader.read_exact(len_buf)?;
    let len = u32::from_le_bytes(*len_buf) as usize;

    if buf.len() < len {
        buf.resize(len, 0);
    }

    reader.read_exact(&mut buf[..len])?;
    let s = String::from_utf8(buf[..len].to_vec()).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;

    Ok(s)
}

#[derive(Default, Debug)]
pub struct GameMap(pub HashMap<u32, Game>);

impl GameMap {
    pub fn write_to(file: &mut impl Write, map: HashMap<u32, &Game>) -> io::Result<()> {
        let mut writer = BufWriter::new(file);

        writer.write_all(&(map.len() as u32).to_le_bytes())?; // Length
        for (appid, game) in map {
            
            // APPID, Key
            writer.write_all(&appid.to_le_bytes())?;
            writer.write_all(&(game.installed as u8).to_le_bytes())?;

            // AppData
            write_string(&mut writer, &game.details.app_type)?;
            write_string(&mut writer, &game.details.name)?;
            write_string(&mut writer, &game.details.header_image)?;

            writer.write_all(&(game.details.is_free as u8).to_le_bytes())?;
            write_string(&mut writer, &game.path)?;

            // writer.write_all(&(game.details.pc_requirements.len() as u32).to_le_bytes())?;
            // for (key, s) in &game.details.pc_requirements {
            //     write_string(&mut writer, key)?;
            //     write_string(&mut writer, s)?;
            // }
        }

        writer.flush()?;
        Ok(())
    }

    pub fn read_from(file: &mut impl Read) -> io::Result<HashMap<u32, Game>> {
        let mut reader = BufReader::new(file);
        let mut buf = [0u8; 4];
        let mut res = Vec::<u8>::with_capacity(512);
        reader.read_exact(&mut buf)?;
        let count = u32::from_le_bytes(buf); // Length
        
        let mut games = HashMap::<u32, Game>::new();
        for _ in 0..count {
            // Appid, Key
            reader.read_exact(&mut buf)?;
            let appid = u32::from_le_bytes(buf);

            reader.read_exact(&mut buf[0..1])?;
            let installed = buf[0] != 0;

            // AppData
            let app_type = read_string(&mut reader, &mut buf, &mut res)?;
            let name = read_string(&mut reader, &mut buf, &mut res)?;
            let header_image = read_string(&mut reader, &mut buf, &mut res)?;

            reader.read_exact(&mut buf[0..1])?;
            let is_free = buf[0] != 0;

            let path = read_string(&mut reader, &mut buf, &mut res)?;

            reader.read_exact(&mut buf)?;
            let count = u32::from_le_bytes(buf);
            let mut pc_requirements = HashMap::<String, String>::new();
            for _ in 0..count {
                let key = read_string(&mut reader, &mut buf,&mut res)?;
                let s = read_string(&mut reader, &mut buf,&mut res)?;
                pc_requirements.insert(key, s);
            }

            games.insert(appid, Game {
                appid: appid,
                installed,
                path,
                details: AppData {
                    app_type,
                    name,
                    is_free,
                    header_image,
                    //pc_requirements
                }
            });
        }

        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    use steamtools::Game;

    use crate::utils::bserializer::GameMap;

    #[test]
    fn write_read() {
        let mut f = fs::File::create("test.lua").unwrap();
        let mut map = HashMap::new();
        let g = Game::default();
        map.insert(1, &g);
        let gm = GameMap::write_to(&mut f, map);
        gm.unwrap()
    }
}