
use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub label: String,
    #[serde(rename = "org")]
    pub organization: Organization,
    pub media: HashMap<String, Media>
}

impl Config {
    pub fn add_media(&mut self, media: Media) {
        self.media.insert(media.label.clone(), media);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub name: String,
    pub label: String,
    #[serde(rename="type")]
    pub media_type: MediaType,
    pub encryption: MediaEncryptionType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MediaType {
    #[serde(rename = "filesystem")]
    Filesystem (FilesystemMedia),
    #[serde(rename = "directory")]
    Directory (DirectoryMedia),
    #[serde(rename = "ssh")]
    SSH (SSHMedia)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectoryMedia {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilesystemMedia {
    pub removable: bool,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SSHMedia {
    pub username: String,
    pub host: String,
    pub port: u32,
    pub path: String,
    pub public_key_filepath: String,
    pub private_key_filepath: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MediaEncryptionType {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "gocryptfs")]
    GoCryptFS,
    #[serde(rename = "cryfs")]
    CryFS
}

impl MediaEncryptionType {
    pub fn token(&self) -> &str {
        match self {
            MediaEncryptionType::GoCryptFS => "gocryptfs",
            MediaEncryptionType::CryFS => "cryfs",
            MediaEncryptionType::None => "",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Organization {
    name: String,
    label: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    name: String,
    label: String,
    description: String,
}

impl Config {
    pub fn read(path: &str) -> Result<Config, String> {
        let file_contents = fs::read_to_string(path)
            .or_else(|e| {
                Err(format!("Failed to read tote config file `{}`. {}.", path, e)) } )?;

        let tote_config: Config = toml::from_str(file_contents.as_str())
            .or_else(|e| {
                Err(format!("Failed to parse tote config file `{}`. {}.", path, e)) } )?;

        Ok(tote_config)
    }
}
