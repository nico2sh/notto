use std::{fs::{self, OpenOptions}, io::Write, path::{Path, PathBuf}};

use uuid::Uuid;

use crate::{errors::NottoError, models::note::Note};

const FILE_NAME_EXTENSION: &str = "md";
pub const DIR_ROOT_NOTE_NAME: &str = "index.md";
const FILE_NAME_LENGTH: usize = 32;

#[derive(Debug, PartialEq, Eq)]
pub enum NoteFileType {
    File(String),
    Directory(String)
}

pub struct Writer {
    base_path: PathBuf
}

impl Writer {
    pub fn new(base_path: PathBuf) -> Self { Self { base_path } }

    pub fn save_note_at<P, S>(&self, note: Note, path: P, file_name: S, overwrite: bool) -> Result<PathBuf, NottoError> where P: AsRef<Path>, S: AsRef<str> {
        if !self.exists(&path) {
            self.create_dir_all(&path)?;
        };

        // No extension in the file
        let dotted_extension = format!(".{}", FILE_NAME_EXTENSION);
        let file_name = if file_name.as_ref().ends_with(&dotted_extension) {
                file_name.as_ref()[..file_name.as_ref().len() - dotted_extension.len()].to_string()
            } else {
                file_name.as_ref().to_string()
            };

        let save_path = match self.note_file_exists(&path, &file_name) {
            Some(note_file_type) => {
                if !overwrite {
                    return Err(NottoError::NoteExists { note_name: file_name } )
                } else {
                    let mut destination_path = PathBuf::from(path.as_ref());
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
                let mut destination_path = PathBuf::from(path.as_ref());
                destination_path.push(format!("{}.{}", file_name, FILE_NAME_EXTENSION));
                destination_path
            }
        };

        let mut file = self.get_note_file(&save_path)?;
        file.write(note.to_text().as_bytes())?;

        Ok(save_path)
    }

    pub fn convert_note_to_parent_note<P>(&self, note_path: P) -> Result<(), NottoError> where P: AsRef<Path> {
        let note_path = note_path.as_ref();
        if !self.exists(&note_path) {
            return Err(NottoError::FileError{ message: format!("Expected note at `{}` but not found", note_path.to_string_lossy() ) });
        }
        if !self.is_file(&note_path) {
            return Err(NottoError::FileError{ message: format!("Expected file at `{}` but it's not", note_path.to_string_lossy() ) });
        }

        if let Some(dir_name) = note_path.file_stem() {
            // We define the destination directory
            let dest_directory = note_path.with_file_name(&dir_name);
            // We create a temp file name
            let temp_file_name = Uuid::new_v4().to_simple().to_string();
            let temp_file_path = note_path.with_file_name(&temp_file_name);
            // We move the file to its new temp file
            self.rename_note_file(&note_path, &temp_file_path)?;
            // We create the directory
            self.create_dir_all(&dest_directory)?;
            // We move the file into the directory with the `index.md` name
            let final_path = dest_directory.join(DIR_ROOT_NOTE_NAME);
            self.rename_note_file(&temp_file_path, &final_path)?;

            Ok(())
        } else {
            Err(NottoError::FileError{ message: format!("Path {} doesn't contain a file", note_path.to_string_lossy() ) })
        }
    }

    pub fn get_file_name_from_note(&self, note: &Note) -> String {
        let special_chars = [ ' ', ':', '.', ',', '/', '\\', '<', '>', '"', '|', '?', '*', '^', '\'' ];
        let mut file_name = note.get_title();
        file_name.retain(|c| !special_chars.contains(&c));
        file_name = file_name.trim().to_string();
        if file_name.len() > FILE_NAME_LENGTH {
            file_name = file_name[..FILE_NAME_LENGTH].to_string();
        }
        
        if file_name.is_empty() {
            file_name = note.front_matter.id[..FILE_NAME_LENGTH].to_string();
        }

        format!("{}.{}", file_name, FILE_NAME_EXTENSION)
    }

    pub fn note_file_exists<P, S>(&self, path: P, file_name: S) -> Option<NoteFileType> where P: AsRef<Path>, S: AsRef<str> {
        let file_name = String::from(file_name.as_ref());
        // We shouldn't get the file name with extension
        let dotted_extension = format!(".{}", FILE_NAME_EXTENSION);
        let (note_file_name, note_dir_name) = if file_name.ends_with(&dotted_extension) {
            (file_name.clone(), file_name[..file_name.len() - dotted_extension.len()].to_string())
        } else {
            (format!("{}.{}", file_name, FILE_NAME_EXTENSION), file_name)
        };

        let mut file_path = PathBuf::from(path.as_ref());
        // Test file
        file_path.push(&note_file_name);
        if self.exists(&file_path) && self.is_file(&file_path) {
            return Some(NoteFileType::File(note_file_name))
        }

        // Test dir
        file_path.pop();
        file_path.push(&note_dir_name);
        if self.exists(&file_path) && self.exists(&file_path) {
            file_path.push(DIR_ROOT_NOTE_NAME);
            if self.exists(&file_path) && self.exists(&file_path) {
                return Some(NoteFileType::Directory(note_dir_name));
            }
        }

        None
    }

    pub fn get_full_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.base_path.join(path)
    }

    /// Operations in the filesystem
    /// Using the base_path as a reference

    fn is_dir<P>(&self, path: P) -> bool where P: AsRef<Path> {
        self.base_path.join(path).is_dir()
    }

    fn is_file<P>(&self, path: P) -> bool where P: AsRef<Path> {
        self.base_path.join(path).is_file()
    }

    fn exists<P>(&self, path: P) -> bool where P: AsRef<Path> {
        self.base_path.join(path).exists()
    }

    pub fn create_dir_all<P>(&self, path: P) -> Result<(), NottoError> where P: AsRef<Path> {
        let components = path.as_ref().components();

        let mut path_tree = PathBuf::new();
        for comp in components {
            match comp {
                std::path::Component::Normal(name) => {
                    path_tree.push(name);
                    self.create_dir_if_doesnt_exist(&path_tree)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn create_dir_if_doesnt_exist<P>(&self, path: P) -> Result<(), NottoError> where P: AsRef<Path> {
        if self.exists(&path) {
            if self.is_file(&path) {
                self.convert_note_to_parent_note(path)
            } else {
                Ok(())
            }
        } else {
            let mut possible_file = PathBuf::from(path.as_ref());
            possible_file.set_extension(FILE_NAME_EXTENSION);
            if self.exists(&possible_file) {
                self.convert_note_to_parent_note(possible_file)
            } else {
                fs::create_dir_all(self.base_path.join(path))?;
                Ok(())
            }
        }
    }

    fn rename_note_file<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<(), NottoError> {
        fs::rename(self.base_path.join(from), self.base_path.join(to))?;

        Ok(())
    }

    fn get_note_file<P: AsRef<Path>>(&self, path: P) -> Result<fs::File, NottoError> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.base_path.join(&path))?;

        Ok(file)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{errors::NottoError, models::{front_matter::FrontMatter, note::Note}};

    use super::Writer;
    use super::NoteFileType;
    use super::FILE_NAME_EXTENSION;
    use super::DIR_ROOT_NOTE_NAME;

    const BASE_PATH: &str = "test_notes_dir"; 

    #[test]
    fn create_note() -> Result<(), NottoError> {
        // Setup ================
        let base_path = PathBuf::from(BASE_PATH);
        if base_path.exists() {
            std::fs::remove_dir_all(&base_path)?;
        }
        std::fs::create_dir(&base_path)?;
        let writer = Writer::new(base_path.clone());
        // Setup ================

        let note_1 = Note::new(FrontMatter::default(), "This is a test note");
        let note_1_path = PathBuf::new();
        writer.save_note_at(note_1, &note_1_path, "test", true)?;

        let test_path_1 = base_path.join(format!("test.{}", FILE_NAME_EXTENSION));
        // Note 1 written
        assert!(test_path_1.exists());
        assert!(test_path_1.is_file());
        assert_eq!(writer.note_file_exists(&note_1_path, "test"), Some(NoteFileType::File("test.md".to_string())));

        let note_2 = Note::new(FrontMatter::default(), "This is another note, on the root");
        writer.save_note_at(note_2, &note_1_path, "test_2", true)?;

        let test_path_2 = base_path.join(format!("test_2.{}", FILE_NAME_EXTENSION));
        // Note 2 written
        assert!(test_path_2.exists());
        assert!(test_path_2.is_file());

        // We create a subnote for test 1
        let note_3 = Note::new(FrontMatter::default(), "This is a subnote from the first one");
        let note_3_path = PathBuf::from("test");
        writer.save_note_at(note_3, note_3_path, "subnote", true)?;

        // Test 1 path shouldn't exist anymore
        assert!(!test_path_1.exists());
        let new_test_path_1 = base_path.join("test").join(DIR_ROOT_NOTE_NAME);
        assert!(new_test_path_1.exists());
        assert_eq!(writer.note_file_exists(&note_1_path, "test"), Some(NoteFileType::Directory("test".to_string())));

        // Note 3 written
        let test_path_3 = base_path.join("test").join(format!("subnote.{}", FILE_NAME_EXTENSION));
        assert!(test_path_3.exists());

        let note_4 = Note::new(FrontMatter::default(), "This is a note deep inside directories.");
        let note_4_path = PathBuf::from("test").join("subnote").join("subpath").join("deep");
        writer.save_note_at(note_4, &note_4_path, "deep", true)?;



        Ok(())
    }
}