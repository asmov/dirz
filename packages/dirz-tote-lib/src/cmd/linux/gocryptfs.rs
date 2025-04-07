use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use super::super::EncryptedFS;
use crate::cmd;
use crate::error::*;

pub struct GoCryptFS {
    encrypted_path: PathBuf,
    decrypted_path: PathBuf,
    password: String
}

impl GoCryptFS {
    const CMD_NAME: &'static str = "gocryptfs";
    const CMD_ARG_INIT: &'static str = "-init";
    const CMD_ARG_QUIET: &'static str = "-quiet";
    const ERROR_CMD_FAILED: &'static str = "GoCryptFS command failed.";

    pub fn new(encrypted_path: PathBuf, decrypted_path: PathBuf, password: String) -> Self {
        GoCryptFS {
            encrypted_path,
            decrypted_path,
            password
        }
    }
}

impl EncryptedFS for GoCryptFS {
    fn encrypted_path(&self) -> &PathBuf {
        &self.encrypted_path
    }

    fn decrypted_path(&self) -> &PathBuf {
        &self.decrypted_path
    }

    fn password(&self) -> &String {
        &self.password
    }

    fn create(&self) -> Result<bool> {
        let mut cmd = Command::new(Self::CMD_NAME)
            .args([Self::CMD_ARG_INIT, Self::CMD_ARG_QUIET])
            .args([&self.encrypted_path])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect(GoCryptFS::ERROR_CMD_FAILED);

        cmd.stdin.as_mut().unwrap().write_all(self.password.as_bytes()).unwrap();
        let output = cmd.wait_with_output().chain_err(|| {
                format!("GoCryptFS failed to create filesystem: {}", self.encrypted_path().to_str().unwrap())
            })?;

        if output.status.success() {
            Ok(true)
        } else {
            Err(format!("GoCryptFS failed to create encrypted filesystem: {} :: {}",
                        self.encrypted_path().to_str().unwrap(),
                        String::from_utf8(output.stderr).unwrap()).into())
        }
    }

    fn mount(&self) -> Result<bool> {
        if self.is_mounted() {
            return Ok(false)
        }

        let mut cmd = Command::new(Self::CMD_NAME)
            .args([ &self.encrypted_path, &self.decrypted_path ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect(Self::ERROR_CMD_FAILED);

        cmd.stdin.as_mut().unwrap().write_all(self.password.as_bytes()).unwrap();

        let output = cmd.wait_with_output()
            .expect(Self::ERROR_CMD_FAILED);

        if output.status.success() {
            Ok(true)
        } else {
            let err = String::from_utf8(output.stderr).unwrap().trim().to_string();
            if err.contains("authentication failed") {
                Err(ErrorKind::MountAuthenticationFailed.into())
            } else {
                Err(format!("{} :: {}", Self::ERROR_CMD_FAILED, err.replace("\n", " :: ")).into())
            }
        }
    }

    fn change_password(&mut self, new_password: String) -> Result<bool> {
        let mut cmd = Command::new(Self::CMD_NAME)
            .arg("-passwd")
            .arg(&self.encrypted_path)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect(Self::ERROR_CMD_FAILED);

        const ASCII_NEWLINE: u8 = 10;
        let mut stdin = cmd.stdin.take().expect("failed to take stdin");
        stdin.write_all(self.password.as_bytes()).expect("Unable to write to STDIN");
        stdin.write_all(&[ASCII_NEWLINE]).expect("Unable to write to STDIN");
        stdin.write_all(new_password.as_bytes()).expect("Unable to write to STDIN");
        stdin.write_all(&[ASCII_NEWLINE]).expect("Unable to write to STDIN");

        let output = cmd.wait_with_output().expect(Self::ERROR_CMD_FAILED);
        cmd::output_error(&output, Self::ERROR_CMD_FAILED)?;

        self.password = new_password;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use super::*;

    const PASSWORD: &'static str = "p4ssw0rd!";
    const TEST_CONTENT: &'static str = "This is test content.";

    fn setup_tmpdir() -> PathBuf {
        fs::create_dir_all("/tmp/dirz-tote-tests").expect("Unable to create base temp dir");

        let output = Command::new("mktemp")
            .args([ "-dt", "test-XXXX", "-p", "/tmp/dirz-tote-tests" ])
            .output()
            .expect("Unable to create temp directory for tests");

        if output.status.success() {
            PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
        } else {
            panic!("Unable to create temp directory for tests");
        }
    }

    fn teardown_tmpdir(path: PathBuf) {
        fs::remove_dir_all(path).expect("Unable to teardown tmpdir")
    }

    fn create_cryptfs() -> (PathBuf, PathBuf, PathBuf, GoCryptFS) {
        let tmpdir = setup_tmpdir();
        let mut encrypted_dir = tmpdir.clone();
        let mut decrypted_dir = tmpdir.clone();
        encrypted_dir.push("encrypted");
        decrypted_dir.push("decrypted");

        fs::create_dir_all(&encrypted_dir).expect("Unable to create encrypted dir");
        fs::create_dir_all(&decrypted_dir).expect("Unable to create decrypted dir");

        let gocryptfs = GoCryptFS::new(
            encrypted_dir.clone(),
            decrypted_dir.clone(),
            String::from(PASSWORD) );

        gocryptfs.create().unwrap();

        ( tmpdir, encrypted_dir, decrypted_dir, gocryptfs )
    }

    fn create_testfile(decrypted_dir: &Path) -> PathBuf {
        let mut test_file = PathBuf::from(decrypted_dir);
        test_file.push("test.txt");
        fs::write(&test_file, TEST_CONTENT).unwrap();
        test_file
    }

    fn assert_testfile(test_file: &Path, errormsg: &'static str) {
        assert!(test_file.exists(), "{}", errormsg);
        assert_eq!(TEST_CONTENT, fs::read_to_string(&test_file).unwrap(), "{}", errormsg);
    }

    #[test]
    fn new() {
        let ( tmpdir, encrypted_dir, decrypted_dir, gocryptfs ) = create_cryptfs();
        assert_eq!(encrypted_dir.to_str().unwrap(), String::from(gocryptfs.encrypted_path().to_str().unwrap()));
        assert_eq!(decrypted_dir.to_str().unwrap(), String::from(gocryptfs.decrypted_path().to_str().unwrap()));
        assert_eq!(PASSWORD, gocryptfs.password());
        teardown_tmpdir(tmpdir);
    }

    #[test]
    fn create() {
        let ( tmpdir, encrypted_dir, _, _) = create_cryptfs();

        let expected_filenames = vec!["gocryptfs.conf", "gocryptfs.diriv"];
        for filename in expected_filenames {
            let mut filepath = encrypted_dir.clone();
            filepath.push(&filename);
            assert!(filepath.exists(), "Expected filepath does not exist");
        }

        teardown_tmpdir(tmpdir);
    }

    #[test]
    fn mount_unmount_ismounted() {
        let ( tmpdir, _, decrypted_dir, gocryptfs) = create_cryptfs();

        assert!(gocryptfs.mount().unwrap(), "Mount should occur");

        let test_file = create_testfile(&decrypted_dir);
        assert_testfile(&test_file, "Test file should exist");

        assert!(gocryptfs.is_mounted(), "Encrypted filesystem should be mounted");

        gocryptfs.unmount().unwrap();

        assert!(!test_file.exists(), "Test file shouldn't exist after unmount");
        assert!(!gocryptfs.is_mounted(), "Encrypted filesystem shouldn't be mounted");

        gocryptfs.mount().unwrap();
        assert!(!gocryptfs.mount().unwrap(), "Mount shouldn't occur if already mounted");

        assert_testfile(&test_file, "Test file should exist");
       assert!(gocryptfs.is_mounted(), "Encrypted filesystem should re-mounted");

        gocryptfs.unmount().unwrap();
        assert!(!gocryptfs.unmount().unwrap(), "Unmount shouldn't occur if already unmounted");

        teardown_tmpdir(tmpdir);
    }

    #[test]
    fn delete() {
        let ( _, encrypted_dir, decrypted_dir, gocryptfs ) = create_cryptfs();

        assert!(gocryptfs.delete().unwrap(), "Filesystem deletion should occur");
        assert!(!gocryptfs.delete().unwrap(), "Filesystem re-deletion shouldn't occur");
        assert!(!decrypted_dir.exists() && !encrypted_dir.exists(),
                "Neither encrypted or decrypted files should exist after deletion");
    }

    #[test]
    fn change_password() {
        const NEW_PASSWORD: &'static str = "p4ssw0rd?";

        let ( _, encrypted_dir, decrypted_dir, gocryptfs ) = create_cryptfs();

        let mut gocryptfs = gocryptfs;
        gocryptfs.change_password(NEW_PASSWORD.to_string()).unwrap();
        assert_eq!(NEW_PASSWORD, gocryptfs.password, "Password in struct should have changed");

        let oldcryptfs = GoCryptFS::new(
            encrypted_dir,
           decrypted_dir,
            String::from(PASSWORD)
        );

        gocryptfs.mount().unwrap();
        gocryptfs.unmount().unwrap();

        if let Err(e) = oldcryptfs.mount() {
            assert!(match e.kind() { ErrorKind::MountAuthenticationFailed => true, _ => false },
                    "Mounting should have failed only on authentication");
        } else {
            panic!("Password on filesystem should have changed")
        }
    }
}
