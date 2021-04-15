use std::{fs::{self}, io::{Read}, path::{Path, PathBuf}, process::{self, ExitStatus}};

use chrono::{Datelike, Utc};
use crossbeam_channel::Receiver;
use errors::NottoError;
use finder::{FindCondition, Finder, NoteFindMessage};
use models::{config::{Config}, front_matter::FrontMatter, note::Note};
use uuid::Uuid;
use io::ReaderWriter;

pub mod models;
mod io;
pub mod finder;
pub mod errors;

const BASE_CONFIG_DIR: &str = ".notto";
const PATH_SEPARATOR: &str = "/";

pub struct Notto {
    pub config: Config,
}

impl Notto {
    pub fn new() -> Result<Self, NottoError> {
        env_logger::init();

        let config = Config::load_config(Notto::get_home_dir()?)?;

        Ok(Self { config })
    }

    pub fn open_by_path<P>(&self, note_path: P) -> Result<(), NottoError> where P: AsRef<Path> {
        let path = self.config.get_notes_dir()?.join(note_path);

        self.open_editor_with_path(&path)?;

        Ok(())
    }

    /// Returns a receiver with the find results
    pub fn find(&self, conditions: Vec<FindCondition>) -> Result<Receiver<NoteFindMessage>, NottoError> {
        let finder = Finder::new(self.config.get_notes_dir()?);
        let rx = finder.find(PathBuf::new(), conditions)?;

        Ok(rx)
    }

    pub fn create_or_open_note_at<S: AsRef<str>>(&self, dest_path: Option<S>) -> Result<PathBuf, NottoError> {
        let writer = ReaderWriter::new(self.config.get_notes_dir()?);

        if let Some(dest_path) = dest_path {
            let path_segments = dest_path.as_ref().split(PATH_SEPARATOR).into_iter().collect::<Vec<_>>();
            let mut path = PathBuf::new();
            let segments = path_segments.len();

            if segments > 1 {
                for i in 0..(segments - 1) {
                    path.push(path_segments[i]);
                }
                writer.create_dir_all(&path)?;
            }
            let file_name = path_segments[segments - 1].to_string();
            let result_path = match writer.note_file_exists(&path, &file_name) {
                Some(note_type) => match note_type {
                    io::NoteFileType::File(file_name) => path.join(file_name),
                    io::NoteFileType::Directory(dir_name) => path.join(dir_name).join(io::DIR_ROOT_NOTE_NAME)
                },
                None => {
                    let front_matter = FrontMatter::default();
                    let note = Note { front_matter, content: String::new() };
                    writer.save_note_at(note, &path, &file_name, false)?
                }
            };

            let status = self.open_editor_with_path(writer.get_full_path(&result_path))?;

            if status.success() {
                Ok(result_path)
            } else {
                Err(NottoError::CreateNoteError { message: format!("Error saving note, exit code: {}", status) })
            }

        } else {
            // A note without name
            let note_text = self.get_text_from_editor()?;
            if !note_text.is_empty() {
                let note = Note::from_text(&note_text);
                let file_name = writer.get_file_name_from_note(&note);
                writer.save_note_at(note, PathBuf::new(), file_name, false)
            } else {
                Err(NottoError::CreateNoteError { message: format!("No content in the note, not saving") })
            }
        }
    }

    pub fn create_journal_entry<S: AsRef<str>>(&self, name: Option<S>) -> Result<PathBuf, NottoError> {
        let date = Utc::now().naive_local().date();
        let note_name = match name {
            Some(n) => {
                let name_segments = n.as_ref().split(PATH_SEPARATOR);
                match name_segments.last() {
                    Some(last) => String::from(last),
                    None => String::from(io::DIR_ROOT_NOTE_NAME)
                }
            }
            None => String::from(io::DIR_ROOT_NOTE_NAME)
        };
        let path = format!("{}/{}/{}/{}", date.year().to_string(), date.month().to_string(), date.day().to_string(), note_name);
        self.create_or_open_note_at(Some(path))
    }

    pub fn get_text_from_editor(&self) -> Result<String, NottoError> {
        let file_path = Notto::get_temp_file()?;

        let status = self.open_editor_with_path(&file_path)?;

        if status.success() {
            let mut editable = String::new();
            fs::File::open(file_path)
                .expect("Can't open file")
                .read_to_string(&mut editable)?;
            
            Ok(editable)
        } else {
            Err(NottoError::CreateNoteError { message: format!("Error saving note, exit code: {}", status) })
        }
    }

    fn open_editor_with_path<P>(&self, path: P) -> Result<ExitStatus, NottoError> where P: AsRef<Path> {
        let editor = self.config.get_editor()?;
        let status = process::Command::new(editor)
            .arg(path.as_ref())
            .status()?;

        Ok(status)
    }

    fn get_temp_file() -> Result<PathBuf, NottoError> {
        let mut file_path = Notto::get_temp_dir()?;
        let uuid = Uuid::new_v4().to_simple();
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
