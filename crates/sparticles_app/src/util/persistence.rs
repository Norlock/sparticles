use crate::model::EmitterUniform;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, BufWriter},
    path::PathBuf,
};

pub struct Persistence;

pub struct ImportError {
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportEmitter {
    pub emitter: EmitterUniform,
    pub is_light: bool,
    pub particle_animations: Vec<DynamicExport>,
    pub emitter_animations: Vec<DynamicExport>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DynamicExport {
    #[serde(rename = "type")]
    pub tag: String,
    pub data: serde_json::Value,
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

    pub fn import_post_fx() -> Result<Vec<DynamicExport>, ImportError> {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push(format!("export/{}", ExportType::PostFx));

        let path = dir.to_str().expect("Path is not correct");

        let file_str = fs::read_to_string(path);
        let error_msg;

        match file_str {
            Ok(file_str) => match serde_json::from_str::<Vec<DynamicExport>>(&file_str) {
                Ok(val) => return Ok(val),
                Err(err) => {
                    let filename = dir.file_name().unwrap().to_str().unwrap();
                    error_msg = format!("Wrong syntaxed JSON for file {}: {}", filename, err);
                }
            },
            Err(err) => {
                error_msg = format!("No post fx export: {}", err);
            }
        }

        Err(ImportError { msg: error_msg })
    }

    pub fn import_emitter_states(path: PathBuf) -> Result<Vec<ExportEmitter>, ImportError> {
        let file_str = fs::read_to_string(path.to_str().expect("Export path is not correct"));

        match file_str {
            Err(err) => println!("{}", err),
            Ok(file_str) => {
                match serde_json::from_str::<Vec<ExportEmitter>>(&file_str) {
                    Ok(val) => return Ok(val),
                    Err(err) => {
                        let filename = path.file_name().unwrap().to_str().unwrap();
                        return Err(ImportError {
                            msg: format!("Wrong syntaxed JSON for file {}: {}", filename, err),
                        });
                    }
                };
            }
        }

        Err(ImportError {
            msg: "file cannot be read".to_owned(),
        })
    }

    pub fn import_textures() -> Result<Vec<PathBuf>, io::Error> {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("src/assets/textures");

        fs::read_dir(dir)?
            .map(|res| {
                res.map(|e| {
                    println!("{:?}", e.path());
                    e.path()
                })
            })
            .collect::<Result<Vec<_>, io::Error>>()
    }
}
