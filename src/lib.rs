use std::{fs::{self}, io::{Read}, path::{PathBuf}, process};

use chrono::{Datelike, Utc};
use errors::NottoError;
use models::{config::{Config}, note::Note};
use uuid::Uuid;
use writer::Writer;

mod models;
mod writer;
mod errors;

const BASE_CONFIG_DIR: &str = ".notto";

pub struct Notto {
    config: Config,
}

impl Notto {
    pub fn new() -> Result<Self, NottoError> {
        let config = Config::load_config(Notto::get_home_dir()?)?;

        Ok(Self { config })
    }

    pub fn create_journal_entry(&self) -> Result<(), NottoError> {
        let note_text = self.get_text_from_editor()?;
        match note_text {
            Some(note_text) => {
                let note = Note::from_text(note_text);

                let date = Utc::now().naive_local().date();
                let mut path = PathBuf::new();
                path.push(date.year().to_string());
                path.push(date.month().to_string());
                path.push(date.day().to_string());

                let writer = Writer::new(self.config.get_base_dir()?.clone());
                let file_name = writer.get_file_name_from_note(&note);
                writer.save_note_at(note, path, file_name, false)?;
            }
            None => {}
        }
        Ok(())
    }

    pub fn get_text_from_editor(&self) -> Result<Option<String>, NottoError> {
        let file_path = Notto::get_temp_file()?;

        let editor = self.config.get_editor()?;
        let status = process::Command::new(editor)
            .arg(&file_path)
            .status()?;

        if status.success() {
            let mut editable = String::new();
            fs::File::open(file_path)
                .expect("Can't open file")
                .read_to_string(&mut editable)?;
            
            Ok(Some(editable))
        } else {
            Ok(None)
        }
    }

    fn get_temp_file() -> Result<PathBuf, NottoError> {
        let mut file_path = Notto::get_temp_dir()?;
        let uuid = Uuid::new_v4();
        let file_name = format!("{}.md", uuid.to_string());
        file_path.push(file_name);
        fs::File::create(&file_path)?;
        Ok(file_path)
    }

    pub fn get_home_dir() -> Result<PathBuf, NottoError> {
        let mut home = dirs::home_dir();
        match home {
            Some(ref mut home_path) => {
                home_path.push(BASE_CONFIG_DIR);
                if !home_path.exists() {
                    fs::create_dir(&home_path)?;
                } else {
                    if !home_path.is_dir() {
                        fs::remove_file(&home_path)?;
                        fs::create_dir(&home_path)?
                    }
                }
                Ok(home_path.clone())
            }
            None => {
                Err(NottoError::HomeDirectoryNotFound)
            }
        }
    }   

    pub fn get_temp_dir() -> Result<PathBuf, NottoError> {
        let mut temp = Notto::get_home_dir()?;
        temp.push(".temp");
        if !temp.exists() {
            fs::create_dir(&temp)?;
        }

        Ok(temp)
    }
}
