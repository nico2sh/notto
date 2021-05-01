use std::path::{Path, PathBuf};
use std::fs;

use crate::errors::NottoError;
use crate::models::note::Note;

const PATH_SEPARATOR: &str = "/";

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct NottoPath {
    path: String,
}

impl NottoPath {
    pub fn new() -> Self {
        Self { path: String::new() }
    }

    pub fn push<S: AsRef<str>>(&mut self, path: S) {
        let segments = path.as_ref().split(PATH_SEPARATOR).into_iter().collect::<Vec<_>>();
        for seg in segments {
            if !self.path.is_empty() {
                self.path.push_str(PATH_SEPARATOR);
            }
            self.path.push_str(seg);
        }
    }
}

impl From<String> for NottoPath {
    fn from(p: String) -> Self {
        let mut path = NottoPath::new();
        path.push(p);
        path
    }
}

impl From<NottoPath> for String {
    fn from(p: NottoPath) -> Self {
        p.path
    }
}

impl From<&NottoPath> for String {
    fn from(p: &NottoPath) -> Self {
        p.path.clone()
    }
}

impl AsRef<Path> for NottoPath {
    fn as_ref(&self) -> &Path {
        &Path::new(&self.path)
    }
}

#[derive(Debug, Eq, PartialEq, Ord)]
pub struct PathEntry {
    name: String,
    pub path: NottoPath,
    is_dir: bool,
}

impl PathEntry {
    pub fn string_to_pathbuf(path: &NottoPath) -> PathBuf {
        let segments = path.path.split(PATH_SEPARATOR).into_iter().collect::<Vec<_>>();
        let mut path = PathBuf::new();

        for seg in segments {
            path.push(seg);
        }

        path
    }

    pub fn pathbuf_to_string<P: AsRef<Path>>(path: P) -> NottoPath {
        let mut result = NottoPath::new();
        let p = path.as_ref();
        for comp in p.components() {
            if let std::path::Component::Normal(name) = comp {
                result.push(&name.to_string_lossy());
            }
        }

        result
    }

    pub fn get_full_path<P: AsRef<Path>>(&self, base_path: P) -> PathBuf {
        base_path.as_ref().join(PathEntry::string_to_pathbuf(&self.path))
    }

    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
}

impl ToString for PathEntry {
    fn to_string(&self) -> String {
        if self.is_dir {
            format!("[{}]", self.name.clone())
        } else {
            self.name.clone()
        }
    }
}

impl PartialOrd for PathEntry {
    // We want directories come first
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.is_dir == other.is_dir {
            if let (Ok(s), Ok(o)) = (self.name.parse::<i32>(), other.name.parse::<i32>()) {
                s.partial_cmp(&o)
            } else {
                self.name.partial_cmp(&other.name)
            }
        } else {
            if self.is_dir { Some(std::cmp::Ordering::Less ) } else { Some(std::cmp::Ordering::Greater ) }
        }
    }
}

pub struct NoteBrowser {
    base_path: PathBuf
}

/*impl Into<NottoPath> for String {
    fn into(self) -> NottoPath {
        let mut path = NottoPath::new();
        path.push(self);
        path
    }
}*/

impl NoteBrowser {
    pub fn new(base_path: PathBuf) -> Self { Self { base_path } }

    fn get_rel_path_string<P: AsRef<Path>>(&self, path: P) -> NottoPath {
        match path.as_ref().strip_prefix(&self.base_path) {
            Ok(p) => {
                PathEntry::pathbuf_to_string(p)
            }
            Err(_) => {
                PathEntry::pathbuf_to_string(path)
            }
        }
    }

    pub fn get_selections_for_path(&self, path: &NottoPath) -> Result<Vec<PathEntry>, NottoError> {
        let full_path = self.base_path.join(path);
        let mut result = vec![];
        for path in fs::read_dir(full_path)? {
            if let Ok(dir_entry) = path {
                let path = dir_entry.path();
                let hidden = if let Some(file_name) = path.file_name() {
                    file_name.to_string_lossy().starts_with(".")
                } else {
                    false
                };

                if !hidden {
                    if path.is_dir() {
                        if let Some(name) = path.file_name() {
                            let name = name.to_string_lossy().to_string();
                            result.push(PathEntry { name, path: self.get_rel_path_string(path), is_dir: true });
                        }
                    } else {
                        if let Ok(note_text) = fs::read_to_string(&path) {
                            let note = Note::from_text(note_text);
                            let mut name = note.get_title();

                            if let Some(file_name) = path.file_name() {
                                name = format!("{} [{}]", name, file_name.to_string_lossy());
                            }
                            result.push(PathEntry { name, path: self.get_rel_path_string(path), is_dir: false });
                        }
                    }
                }
            };
        }

        result.sort();

        Ok(result)
    }

}
