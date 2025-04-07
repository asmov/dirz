pub mod error;
pub mod config;
pub mod cmd;

use std::fs;
use std::path::PathBuf;
use config::Config;
use cmd::linux::gocryptfs;
use cmd::EncryptedFS;
use error::*;

pub struct Tote {
    config: Config
}

impl Tote {
    pub fn new(config_path: &str) -> Result<Tote> {
        let tote_config = Config::read(config_path)?;
        Ok(Tote {
            config: tote_config
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn create_media(&mut self, media: config::Media) -> Result<bool> {
        match &media.media_type {
            config::MediaType::Directory(location) => {
                self.create_directory_media(&location)?;
            },
            _ => panic!("Unexpected MediaLocation")
        };

        self.config.add_media(media);
        Ok(true)
    }

    fn create_directory_media(&self, directory: &config::DirectoryMedia) -> Result<bool> {
        let media_path = PathBuf::from(&directory.path);
        if !media_path.exists() {
            fs::create_dir_all(&media_path)
                .expect("Unable to create directory.");
        }

        let mut tote_path = PathBuf::from(&media_path);
        tote_path.push(&self.config.name);

        fs::create_dir_all(&tote_path)
            .expect("Unable to create tote directory");

        let mut encrypted_tote_path = PathBuf::from(&media_path);
        encrypted_tote_path.push(format!(".{}.{}", self.config.name, directory.encryption.token()));

        fs::create_dir_all(&encrypted_tote_path)
            .expect("Unable to create tote directory");

        let crypt = gocryptfs::GoCryptFS::new(encrypted_tote_path,
                                   tote_path,
                                   String::from("p4ssw0rd!"));

        crypt.create()?;
        crypt.mount()
    }
}