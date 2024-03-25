use dirs;
use serde;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    path: path::PathBuf,
    last_update_check: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataContent {
    pub last_update_check: u64,
    pub last_checked_version: String,
}

impl Default for DataContent {
    fn default() -> Self {
        Self {
            last_update_check: 0,
            last_checked_version: String::from("v0.0.0"),
        }
    }
}

impl Data {
    pub fn new() -> Self {
        let path = dirs::data_dir()
            .expect("unable to find data directory")
            .join("bwenv")
            .join("bwenv.yaml");

        Self {
            last_update_check: 0,
            path,
        }
    }

    fn read(&self) -> Result<DataContent, Box<dyn std::error::Error>> {
        if !self.path.exists() {
            let file = fs::File::create(&self.path)?;
            let default_data = DataContent::default();
            serde_yaml::to_writer(file, &default_data)?;
            return Ok(default_data);
        }

        let file = fs::File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        let data = serde_yaml::from_reader::<io::BufReader<fs::File>, DataContent>(reader)
            .unwrap_or_else(|_| DataContent::default());
        Ok(data)
    }

    fn write(&self, content: &DataContent) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(&self.path, serde_yaml::to_string(content).unwrap())?;
        Ok(())
    }

    pub fn get_content(&self) -> Result<DataContent, Box<dyn std::error::Error>> {
        Ok(self.read()?)
    }

    pub fn set_content(
        &self,
        last_update_check: u64,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_data = self.read()?;
        current_data.last_update_check = last_update_check;
        current_data.last_checked_version = version;
        Ok(self.write(&current_data)?)
    }
}
