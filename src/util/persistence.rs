use crate::fx::post_process::FxPersistenceType;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};

pub struct Persistence;

pub struct ImportError {
    pub msg: String,
}

#[derive(Serialize, Deserialize)]
pub struct ExportAnimation {
    #[serde(rename = "type")]
    pub animation_type: String,
    pub animation: serde_json::Value,
}

pub enum ExportType {
    PostFx,
    EmitterStates,
}

impl Display for ExportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportType::PostFx => f.write_str("post_fx.json"),
            ExportType::EmitterStates => f.write_str("emitters.json"),
        }
    }
}

impl Persistence {
    pub fn write_to_file(to_export: impl Serialize, file_type: ExportType) {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push(format!("export/{}", file_type));

        let path = dir.to_str().expect("not correct");
        let file = File::create(path).expect("Path for export doesn't exist");

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &to_export).expect("Can't write export");
    }

    pub fn fetch_post_fx() -> Result<Vec<FxPersistenceType>, ImportError> {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push(format!("export/{}", ExportType::PostFx));

        let path = dir.to_str().expect("Path is not correct");

        let file_res = fs::read_to_string(path);
        let error_msg;

        match file_res {
            Ok(val) => match serde_json::from_str::<Vec<FxPersistenceType>>(&val) {
                Ok(val) => {
                    println!("Import {:?}", &val);
                    return Ok(val);
                }
                Err(err) => {
                    error_msg = format!("Wrong syntaxed JSON: {}", err);
                }
            },
            Err(err) => {
                error_msg = format!("No post fx export: {}", err);
            }
        }

        Err(ImportError { msg: error_msg })
    }
}
