use std::path::PathBuf;
use log::error;
use serde::{Serialize, Deserialize};

use crate::writer::Writer;

use super::{config::Config, note::Note};

#[derive(Debug, Serialize, Deserialize)]
pub struct BucketItem {
    context: String,
    file: String,
    file_name: Option<String>,
    dest_path: PathBuf
}

/// The Bucket is a file with the pending tasks
/// for copying note files from the temp directory
/// in case an editor crashed but left the temp file
#[derive(Debug, Serialize, Deserialize)]
pub struct Bucket {
    items: Vec<BucketItem>
}

impl Bucket {
    pub fn process(&self, config: Config, temp_dir: PathBuf) {
        for item in &self.items {
            match config.get_base_dir_from(&item.context) {
                Ok(base_path) => {
                    let temp_note_path = temp_dir.join(&item.file);
                    let written = match std::fs::read_to_string(&temp_note_path) {
                        Ok(text) => {
                            let writer = Writer::new(base_path);
                            let note = Note::from_text(text);
                            let file_name = item.file_name.as_ref().map(|f| f.clone()).unwrap_or_else(|| writer.get_file_name_from_note(&note));
                            writer.save_note_at(note, &item.dest_path, file_name, false).and_then(|_| {
                                Ok(std::fs::remove_file(&temp_note_path)?)
                            })
                        }
                        Err(e) => {
                            Err(e.into())
                        }
                    };

                    if let Err(e) = written {
                        error!("{}", e);
                    }
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
}

