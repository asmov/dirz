#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod record;
pub mod schema;
pub mod git;

use std::path::{PathBuf, Path};
use std::{env, fs};
use std::io;
use toml;
use crate::record::{RecordName};

#[macro_export]
macro_rules! path {
    [ $($segment:expr),+ ] => {{
        let mut path = ::std::path::PathBuf::new();
        $(path.push($segment);)*
        path
    }}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unspecified error")]
    Unspecified,
    #[error("Unable to read/write schema file")]
    SchemaIO,
    #[error("Invalid schema format found when parsing")]
    SchemaParsing,
}

const ROOT_CONFIG_FILENAME: &'static str = ".cabinet.root.toml";
const CONFIG_FILENAME: &'static str = ".cabinet.toml";

pub fn find_record_names(path: &Path) -> Vec<RecordName> {
    let mut record_names: Vec<RecordName> = Vec::new();

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue
        }

        let record_name = RecordName::parse(entry.file_name().into_string().unwrap()).unwrap();
        record_names.push(record_name);
    }

    record_names.sort();
    record_names
}

pub fn cmd_list_directory(path: &Path) {
    for record_name in find_record_names(path) {
        println!("{}", record_name.to_string());
    }
}

pub fn is_cabinet_root(dir: &Path) -> bool {
    return path!(dir, ROOT_CONFIG_FILENAME).exists();
}

pub fn is_cabinet(dir: &Path) -> bool {
    return path!(dir, CONFIG_FILENAME).exists();
}

pub fn find_cabinet_root(dir: &Path) -> Result<PathBuf, io::Error>{
    let mut dir_path = dir;

    loop {
        if is_cabinet_root(dir_path) {
            return Ok(PathBuf::from(dir_path));
        } else if let Some(parent_dir) = dir_path.parent() {
            dir_path = parent_dir;
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, dir_path.to_str().unwrap()))
        }
    }
}

pub fn find_cabinet(dir: &Path) -> Result<PathBuf, io::Error>{
    let mut dir_path = dir;

    loop {
        if is_cabinet(dir_path) {
            return Ok(PathBuf::from(dir_path));
        } else if let Some(parent_dir) = dir_path.parent() {
            dir_path = parent_dir;
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, dir_path.to_str().unwrap()))
        }
    }
}

pub fn cabinet_root_config(dir: &Path) -> Result<toml::Value, io::Error> {
    match find_cabinet_root(dir) {
        Ok(root_dir) => {
            let config_filepath = path!(root_dir, ROOT_CONFIG_FILENAME);
            let cfg = fs::read_to_string(config_filepath).unwrap();
            let value = cfg.parse::<toml::Value>().unwrap();
            Ok(value)
        },
        Err(_) => Err(io::Error::new(io::ErrorKind::NotFound, "No cabinet selected"))
    }
}

pub fn cabinet_config(dir: &Path) -> Result<toml::Value, io::Error> {
    match find_cabinet(dir) {
        Ok(root_dir) => {
            let config_filepath = path!(root_dir, CONFIG_FILENAME);
            let cfg = fs::read_to_string(config_filepath).unwrap();
            let value = cfg.parse::<toml::Value>().unwrap();
            Ok(value)
        },
        Err(_) => Err(io::Error::new(io::ErrorKind::NotFound, "No cabinet selected"))
    }
}

pub fn cmd_cabinet_root_name() {
    match find_cabinet_root(&env::current_dir().unwrap()) {
        Ok(root_dir) => {
            match cabinet_root_config(&root_dir) {
                Ok(value) => {
                    println!("{} :: {}", value["name"].as_str().unwrap(),value["label"].as_str().unwrap());
                },
                Err(err) => {
                    eprintln!("{}", err.to_string());
                }
            }
        },
        Err(_) => {
            eprintln!("No cabinet selected.");
        }
    }
}

pub fn cmd_cabinet_name() {
    let cwd = env::current_dir().unwrap();
    let root_config = match cabinet_root_config(&cwd) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return;
        }
    };

    match find_cabinet(&cwd) {
        Ok(dir) => {
            match cabinet_config(&dir) {
                Ok(config) => {
                    println!("{} // {}", root_config["label"].as_str().unwrap(), config["label"].as_str().unwrap());
                 },
                Err(err) => {
                    eprintln!("{}", err.to_string());
                }
            }
        },
        Err(_) => {
            eprintln!("No cabinet selected.");
        }
    }
}

#[cfg(test)]
mod testing {
    use std::path::{PathBuf, Path};
    use std::fs;
    use std::process;

    pub struct Testing {
        base_tmp_dir: PathBuf,
        tmp_dir: PathBuf
    }

    impl Testing {
        pub fn setup(base_tmp_dir_path: &str) -> Testing {
            let base_tmp_dir = PathBuf::from(base_tmp_dir_path);
            let tmp_dir = Self::setup_tmpdir(&base_tmp_dir);

            Testing {
                base_tmp_dir,
                tmp_dir
            }
        }

        fn setup_tmpdir(tmp_dir: &Path) -> PathBuf {
            const TEST_DIR_ERROR: &'static str = "Unable to create testing tmpdir: ";

            if tmp_dir.exists() {
                fs::remove_dir_all(tmp_dir)
                    .expect(&format!("Unable to wipe previous testing tmpdir: {}", &tmp_dir.to_str().unwrap()));
            }

            fs::create_dir_all(tmp_dir)
                .expect(&format!("{TEST_DIR_ERROR}{}", tmp_dir.to_str().unwrap()));

            Self::mktemp(tmp_dir)
        }

        fn mktemp(subdir: &Path) -> PathBuf {
            const TEST_DIR_ERROR: &'static str = "Unable to create testing tmpdir via mktemp: ";

            let output = process::Command::new("mktemp")
            .args(["-dt", "test-XXXX", "-p", subdir.to_str().unwrap()])
            .output()
            .expect(&*format!("{TEST_DIR_ERROR}{}", subdir.to_str().unwrap()));

            if !output.status.success() {
                panic!("{TEST_DIR_ERROR}{}", subdir.to_str().unwrap());
            }

            PathBuf::from(String::from_utf8(output.stdout).unwrap().trim()).canonicalize().unwrap()
        }

        pub fn create_subtest_dir(&self, path_prefix: &str) -> PathBuf {
            const TEST_DIR_ERROR: &'static str = "Unable to create sub-test tmp dir: ";

            let subtest_dir = path!(&self.tmp_dir, path_prefix);
            fs::create_dir_all(&subtest_dir)
                .expect(&*format!("{TEST_DIR_ERROR}{}", subtest_dir.to_str().unwrap()));

            Self::mktemp(&subtest_dir)
        }

        pub fn create_subtest_dir_copy(&self, copy_from_dir: &Path, path_prefix: &str) -> PathBuf {
            let subtest_dir = self.create_subtest_dir(path_prefix);
            let options = fs_extra::dir::CopyOptions::new().content_only(true);
            fs_extra::dir::copy(&copy_from_dir, &subtest_dir, &options).unwrap();
            subtest_dir
        }

        pub fn tmp_dir(&self) -> &Path {
            return &self.tmp_dir;
        }

        pub fn teardown(&self) {
            self.teardown_tmp_dir();
        }

        fn teardown_tmp_dir(&self) {
            fs::remove_dir_all(&self.base_tmp_dir)
                .expect(&*format!("Unable to delete testing tmp dir: {}", self.tmp_dir.to_str().unwrap()));
        }
    }

    impl Drop for Testing {
        fn drop(&mut self) {
            self.teardown();
        }
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use std::path::{PathBuf};
    use crate::testing::Testing;
    use super::*;

    const SIMPLE_TRANSACTION_FIXTURE_RECORD_NAMES: [&str;3] = [
        "TXN 221001 Purchased office supplies from Alpha Inc",
        "TXN 221002 Service fees from Bravo LLC for 2022-01",
        "TXN 221003 Taxi fare for Jane Smith from HQ to Gotham Convention Center",
    ];

    const SIMPLE_TRANSACTION_FIXTURE_DIR: &'static str = "tests/fixtures/transactions-simple/zulucorp/account/transactions";

    lazy_static! {
        static ref TESTING: Testing = Testing::setup("/tmp/dirz-lib-test/lib");
    }

    #[test]
    fn test_find_record_names() {
        let record_names = find_record_names(&PathBuf::from(SIMPLE_TRANSACTION_FIXTURE_DIR));
        let record_names_str: Vec<String> = record_names.iter().map(|r| r.to_string()).collect();
        let record_names_expected: Vec<String> = SIMPLE_TRANSACTION_FIXTURE_RECORD_NAMES.iter().map(|r| r.to_string()).collect();
        for r in &record_names_str {
            println!("{}", r);
        };

        assert_eq!(record_names_expected, record_names_str);
    }
}
