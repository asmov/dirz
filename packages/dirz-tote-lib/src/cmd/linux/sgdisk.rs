use std::path::{PathBuf, Path};
use std::process;
use std::process::Stdio;
use error::*;

pub trait sgdiskCLI {
    fn zap_all();
}

pub struct sgdisk {
    device: PathBuf
}

impl sgdisk {
    const CMD_NAME: &str = "sgdisk";
    pub fn new(device: &Path) -> sgdisk {
        sgdisk {
            device: PathBuf::from(device)
        }
    }

    fn create_cmd(&self) -> process::Command {
        let mut cmd = process::Command::new(Self::CMD_NAME);
        cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }
}

impl sgdiskCLI for sgdisk {
    fn zap_all(&self) {
        let mut cmd = self.create_cmd();
        cmd.arg("--zap-all").arg(self.device);
    }
}

