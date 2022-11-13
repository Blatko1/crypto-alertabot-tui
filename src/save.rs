use crate::error::Result;
use std::{fs, io};

const SAVE: &str = "bot_data.json";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SaveData {
    zones: Vec<PriceLevelData>,
}

impl SaveData {
    fn empty() -> Self {
        Self { zones: Vec::new() }
    }

    pub fn data(self) -> Vec<Zone> {
        let mut zones = Vec::with_capacity(self.zones.len());
        for z in self.zones {
            zones.push(z.into());
        }
        zones
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PriceLevelData(f64);

/// Searches for the save file.
///
/// Possibilities:
/// - If the save file is **found** the function returns the data.
/// - If the save file is **not found** the function creates a new one.
pub fn load_create_save_file() -> Result<SaveData> {
    let path = &format!(
        "{}/{}",
        std::env::current_exe()?
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            // If the program is run from the source it would have these char prefixes
            .trim_start_matches("\\\\?\\"),
        SAVE
    );

    match fs::read(path) {
        Ok(data) =>  {
            println!("Save file found! Loading ...");
            Ok(serde_json::from_slice(&data)?)
        },
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                println!("Save file NOT found! Creating a new one ...");
                let form = serde_json::to_string(&SaveData::empty())?;
                fs::write(path, form)?;
                Ok(SaveData::empty())
            },
            _ => Err(err.into()),
        }
    }
}

/// Saves `data` into the save file. 
/// 
/// If the save file is not found it creates a new file with memorized data.
pub fn save_data(data: &SaveData) -> Result<()> {
    let path = &format!(
        "{}/{}",
        std::env::current_exe()?
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            // If the program is run from the source it would have these char prefixes
            .trim_start_matches("\\\\?\\"),
        SAVE
    );
    let serialized = serde_json::to_string(data)?;

    // Error check in case file is missing
    if let Err(e) = fs::File::open(path) {
        match e.kind() {
            io::ErrorKind::NotFound => println!(
                "The save file is missing! \
                    Creating a new one with memorized data."
            ),
            _ => return Err(e.into()),
        }
    }

    fs::write(SAVE, serialized)?;

    Ok(())
}