#[macro_export]
macro_rules! path {
    [ $($segment:expr),+ ] => {{
        let mut path = ::std::path::PathBuf::new();
        $(path.push($segment);)*
        path
    }}
}

pub mod linux;
pub mod git;

use std::{fs, io};
use std::io::Write;
use std::path::{PathBuf};
use std::process::{Command, Output, Stdio};

use crate::error::*;


pub trait EncryptedFS {
    const CMD_UMOUNT: &'static str ="umount";
    const CMD_FINDMNT: &'static str ="findmnt";
    const ERROR_UMOUNT_FAILED: &'static str = "Command `umount` failed.";
    const ERROR_FINDMNT_FAILED: &'static str = "Command `findmnt` failed.";

    fn encrypted_path(&self) -> &PathBuf;
    fn decrypted_path(&self) -> &PathBuf;
    fn password(&self) -> &String;
    fn create(&self) -> Result<bool>;
    fn mount(&self) -> Result<bool>;
    fn change_password(&mut self, new_password: String) -> Result<bool>;

    fn unmount(&self) -> Result<bool> {
        self.system_umount()
    }

    fn delete(&self) -> Result<bool> {
        if !self.decrypted_path().exists() && !self.encrypted_path().exists() {
            return Ok(false)
        } else if self.is_mounted() {
            self.unmount()?;
        }

        fs::remove_dir_all(self.decrypted_path()).chain_err(|| "Unable to remove dir")?;
        fs::remove_dir_all(self.encrypted_path()).chain_err(|| "Unable to remove dir")?;

        Ok(true)
    }

    fn is_mounted(&self) -> bool {
        self.system_is_mounted()
    }

    fn system_is_mounted(&self) -> bool {
        let output = Command::new(Self::CMD_FINDMNT)
            .args([ self.decrypted_path() ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .expect(Self::ERROR_FINDMNT_FAILED);

        output.status.success()
    }

    fn system_umount(&self) -> Result<bool> {
        if !self.system_is_mounted() {
            return Ok(false);
        }

        let output = Command::new(Self::CMD_UMOUNT)
            .args([ self.decrypted_path() ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .expect(Self::ERROR_UMOUNT_FAILED);

        io::stderr().write_all(&output.stderr).unwrap();

        if output.status.success() {
            Ok(true)
        } else {
            Err(format!("{} {}", Self::ERROR_UMOUNT_FAILED,
                        String::from_utf8(output.stderr).unwrap()).into())
        }
    }
}

fn output_error(output: &Output, error: &'static str) -> Result<bool>{
    if output.status.success() {
        Ok(true)
    } else {
        Err(format!("{} :: {}",
                error,
                String::from_utf8_lossy(&output.stderr).trim()
                    .replace("\n", " :: ")).into())
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;

    pub fn setup_tmpdir() -> PathBuf {
        const TEST_DIR: &'static str = "/tmp/dirz-tote-tests";
        const TESTDIR_ERROR: &'static str = "Unable to create testing tmp dir: ";

        fs::create_dir_all(TEST_DIR).expect(&*format!("{TESTDIR_ERROR}{}", TEST_DIR));

        let output = process::Command::new("mktemp")
            .args(["-dt", "test-XXXX", "-p", TEST_DIR])
            .output()
            .expect(&*format!("{TESTDIR_ERROR}{}", TEST_DIR));

        if output.status.success() {
            PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
        } else {
            panic!("{TESTDIR_ERROR}{}", TEST_DIR);
        }
    }

    #[cfg(test)]
    pub fn teardown_tmpdir(path: &Path) {
        fs::remove_dir_all(path).expect(&*format!("Unable to delete testing tmp dir: {}", path.to_str().unwrap()));
    }
}
