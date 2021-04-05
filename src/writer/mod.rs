use std::{fs::{self, OpenOptions}, io::Write, path::{Path, PathBuf}};

use crate::{errors::NottoError, models::note::Note};

const DIR_ROOT_NOTE_NAME: &str = "index.md";
const FILE_NAME_LENGTH: usize = 25;

enum NoteFileType {
    File(String),
    Directory(String)
}

pub struct Writer {
    base_path: PathBuf
}

impl Writer {
    pub fn new(base_path: PathBuf) -> Self { Self { base_path } }

    pub fn save_note_at<P, S>(&self, note: Note, path: P, file_name: S, overwrite: bool) -> Result<(), NottoError> where P: AsRef<Path>, S: AsRef<str> {
        let mut destination_path = PathBuf::from(&self.base_path);
        destination_path.push(&path);

        if !destination_path.exists() {
            fs::create_dir_all(&destination_path)?;
        };

        let save_path = match self.note_file_exists(&path, &file_name) {
            Some(note_file_type) => {
                if !overwrite {
                    return Err(NottoError::NoteExists { note_name: file_name.as_ref().to_string() } )
                } else {
                    match note_file_type {
                        NoteFileType::File(note_file_name) => {
                            destination_path.push(note_file_name);
                        }
                        NoteFileType::Directory(directory_name) => {
                            destination_path.push(directory_name);
                            destination_path.push(DIR_ROOT_NOTE_NAME);
                        }
                    }
                    destination_path
                }
            }
            None => {
                destination_path.push(file_name.as_ref());
                destination_path
            }
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(save_path)?;

        file.write(note.to_text().as_bytes())?;

        Ok(())
    }

    pub fn get_file_name_from_note(&self, note: &Note) -> String {
        let special_chars = [ ' ', ':', '.', '/', '\\', '<', '>', '"', '|', '?', '*', '^', '\'' ];
        let mut file_name = note.front_matter.title.clone();
        file_name.retain(|c| special_chars.contains(&c));
        if file_name.len() > FILE_NAME_LENGTH {
            file_name = file_name[..FILE_NAME_LENGTH].to_string();
        }
        
        if !file_name.is_empty() {
            file_name
        } else {
            note.front_matter.id[..FILE_NAME_LENGTH].to_string()
        }
    }

    fn note_file_exists<P, S>(&self, path: P, file_name: S) -> Option<NoteFileType> where P: AsRef<Path>, S: AsRef<str> {
        let file_name = String::from(file_name.as_ref());
        // We shouldn't get the file name with extension
        let (note_file_name, note_dir_name) = if file_name.ends_with(".md") {
            (file_name.clone(), file_name[..file_name.len() - 3].to_string())
        } else {
            (format!("{}.md", file_name), file_name)
        };

        let mut file_path = PathBuf::from(&self.base_path);
        file_path.push(path);

        // Test file
        file_path.push(&note_file_name);
        if file_path.exists() && file_path.is_file() {
            return Some(NoteFileType::File(note_file_name))
        }

        file_path.pop();
        file_path.push(&note_dir_name);

        if file_path.exists() && file_path.is_dir() {
            file_path.push(DIR_ROOT_NOTE_NAME);
            if file_path.exists() && file_path.is_file() {
                return Some(NoteFileType::Directory(note_dir_name));
            }
        }

        None
    }
}