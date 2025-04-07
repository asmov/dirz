
use std::collections::{BTreeMap};
use serde::{Serialize, Deserialize};

pub mod tree;
pub use tree::Tree;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum ShardType {
    #[serde(rename = "none")]
    #[default]
    None,
    #[serde(rename = "path")]
    Path,  // not valid option for directory schema's
    #[serde(rename = "year_quarter")]
    YearQuarter,
}


#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum IDType {
    #[serde(rename = "none")]
    #[default]
    None,
    #[serde(rename = "short_year_quarter_index")]
    ShortYearQuarterIndex
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum StructureType {
    #[serde(rename = "none")]
    #[default]
    None,
    #[serde(rename = "directory")]
    Directory,
    #[serde(rename = "filename")]
    Filename,
    #[serde(rename = "zip")]
    Zip
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CabinetPath {
    name: String,
    label: String,
    #[serde(default)]
    shard_type: ShardType,
    #[serde(default)]
    folders: Vec<String>, // name => relative_path
    #[serde(default)]
    sharded_folders: Vec<String>, // name => relative_path
    #[serde(default, rename = "record")]
    record_schema: Option<Record>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Record {
    name: String,
    label_singular: String,
    label_plural: String,
    shard_type: ShardType,
    id_type: IDType, 
    structure: StructureType,
    format: String,
    #[serde(default)]
    file_extensions: Vec<String>,
    #[serde(rename = "field")]
    fields: BTreeMap<String, FieldSchema>
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum FieldType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "ident")]
    Ident,
    #[serde(rename = "date")]
    Date,
    #[serde(rename = "token")]
    Token,
    #[serde(rename = "summary")]
    Summary
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum FieldTypeAttributes {
    #[default]
    None,
    #[serde(rename = "string")]
    String (StringField),
    #[serde(rename = "ident")]
    Ident (IdentField),
    #[serde(rename = "date")]
    Date (DateField),
    #[serde(rename = "token")]
    Token (TokenField),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct StringField {
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    length: usize,  // 0 ==> std::usize::MAX
    preset: StringFieldPreset,
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum StringFieldPreset {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "summary")]
    Summary
}

impl StringFieldPreset {
    const SUMMARY: Option<StringField> = Some(StringField {
        optional: false,
        length: 128,
        preset: Self::None,
    });

    pub fn preset(&self) -> Option<StringField> {
        match self {
            Self::None => None,
            Self::Summary => Self::SUMMARY
        }
    }
}
    
  
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct TokenField {
    #[serde(default)]
    optional: bool,
    schema: String,
    field: String
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct RecordIdentField {
    schema: String 
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct IdentField {
    #[serde(default)]
    optional: bool,
    schema: String 
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum DatePreset {
    YMD,    // YYYY-MM-DD
    SYQ     // YYQ 
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct DateField {
    #[serde(default)]
    optional: bool,
    preset: DatePreset,
}


#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct FieldSchema {
    name: String,
    #[serde(rename = "type")]
    field_type: FieldType,
    //#[serde(default)]
    //#[serde(flatten)]
    //type_attributes: FieldTypeAttributes,
    #[serde(rename="string")]
    string_attributes: Option<StringField>,
    #[serde(rename="ident")]
    ident_attributes: Option<IdentField>,
    #[serde(rename="date")]
    date_attributes: Option<DateField>,
    #[serde(rename="token")]
    token_attributes: Option<TokenField>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_directory_schema() {
        const FIXTURE: &'static str = r#"
            name = "test"
            label = "Test Label"

            folders = [
                "testone"
            ]
        "#;

        let schema: CabinetPath = toml::from_str(FIXTURE).unwrap();
        assert_eq!("test", schema.name);
        assert_eq!("Test Label", schema.label);
        assert_eq!(1, schema.folders.len());
        assert_eq!("testone", schema.folders.get(0).unwrap());
    }
}