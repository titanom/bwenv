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

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DataContent {
    pub last_update_check: u64,
    pub last_checked_version: Option<String>,
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
            let _ = fs::create_dir_all(&self.path);
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
        std::fs::write(&self.path, serde_yaml::to_string(content)?)?;
        Ok(())
    }

    pub fn get_content(&self) -> DataContent {
        self.read().unwrap_or_default()
    }

    pub fn set_content(
        &self,
        last_update_check: u64,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_data = self.read()?;
        current_data.last_update_check = last_update_check;
        current_data.last_checked_version = Some(version);
        self.write(&current_data)
    }
}
