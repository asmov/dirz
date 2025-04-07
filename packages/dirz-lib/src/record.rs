

use regex::Regex;
use lazy_static::lazy_static;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid record name format"), context(suffix(Err)))]
    InvalidRecordName { source_str: String },
    #[snafu(display("Invalid record name format"), context(suffix(Err)))]
    InvalidCabinetID { source_str: String },
    #[snafu(display("Invalid record name format"), context(suffix(Err)))]
    InvalidDirectoryName { source_str: String },
    #[snafu(display("Invalid record name format"), context(suffix(Err)))]
    InvalidIndex { source_str: String },
}

#[derive(Debug)]
pub struct Record {
    name: RecordName,
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordName {
    id: CabinetID,
    title: String,
}

trait RecordNameTokenParser<T = Self> {
    fn parse(str: String) -> Result<(T, String), Error>;
}

impl ToString for RecordName {
    fn to_string(&self) -> String {
        format!("{} {}", self.id.to_string(), self.title)
    }
}

impl RecordName {
    pub fn id(&self) -> &CabinetID { &self.id }
    pub fn title(&self) -> &String { &self.title }

    pub fn parse(str: String) -> Result<Self, Error> {

        let (cabinet_id, str) = CabinetID::parse(str)?;
        Ok(RecordName {
            id: cabinet_id,
            title: str
        })
    }
}

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct CabinetID {
    directory_name: DirectoryName,
    index: QuarterlyIndex, //TODO: this should more generic
}

impl ToString for CabinetID {
    fn to_string(&self) -> String {
        format!("{} {}", self.directory_name.to_string(), self.index.to_string())
    }
}

impl RecordNameTokenParser for CabinetID {
    fn parse(str: String) -> Result<(Self, String), Error> {
        let mut tokens = str.split(' ');
        if let (Some(directory_token), Some(index_token)) = (tokens.next(), tokens.next()) {
            let (directory_name, _) = DirectoryName::parse(directory_token.to_string())?;
            let (index, _) = QuarterlyIndex::parse(index_token.to_string())?;

            let cabinet_id = CabinetID {
                directory_name,
                index,
            };

            let len = directory_token.len() + index_token.len() + 2;

            return Ok((cabinet_id, (&str[len..]).to_string()))
        }

        Err(Error::InvalidCabinetID { source_str: str })
    }
}

pub trait RecordIndex: std::fmt::Display {}

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct QuarterlyIndex {
    year: u16,
    quarter: u8,
    number: u32
}


impl RecordNameTokenParser for QuarterlyIndex {
    fn parse(str: String) -> Result<(Self, String), Error> {
        lazy_static! {
            static ref INDEX_REGEX: Regex = Regex::new(r"^([0-9]{2})([0-9])([0-9]{3})$").unwrap();
        }

        ensure!(INDEX_REGEX.is_match(&str), InvalidIndexErr {source_str: str });
        let captures = INDEX_REGEX.captures(&str).unwrap();

        let year = captures.get(1).unwrap().as_str().to_string();
        let year = year.parse::<u16>().unwrap();
        let quarter = captures.get(2).unwrap().as_str().to_string();
        let quarter = quarter.parse::<u8>().unwrap();
        let index = captures.get(3).unwrap().as_str().to_string();
        let index = index.parse::<u32>().unwrap();

        let quarterly_index = QuarterlyIndex{
            year: 2000 + year,
            quarter,
            number: index
        };

        Ok((quarterly_index, str))
    }
}

impl RecordIndex for QuarterlyIndex {}

impl std::fmt::Display for QuarterlyIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{:<03}", &(self.year.to_string())[2..], self.quarter, self.number)
    }
}

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectoryName {
    name: String,
}

impl ToString for DirectoryName {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl RecordNameTokenParser for DirectoryName {
    fn parse(str: String) -> Result<(Self, String), Error> {
        lazy_static! {
            static ref VALID_REGEX: Regex = Regex::new(r"^[A-Z0-9]+(?:\-?[A-Z0-9]+)*$").unwrap();
        }

        ensure!(VALID_REGEX.is_match(&str), InvalidDirectoryNameErr { source_str: str });
        let directory_name = DirectoryName { name: str };
        Ok((directory_name, String::new()))
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn create_record_name() {
        let directory = DirectoryName {
            name: String::from("TXN"),
        };

        let record_name = RecordName {
            id: CabinetID {
                directory_name: directory,
                index: QuarterlyIndex {
                    year: 2022,
                    quarter: 2,
                    number: 3
                }
            },
            title: String::from("This is a title")
        };

        println!("RECORD NAME: {:?}", record_name);
        println!("record str: {}", record_name.to_string());
    }
}
