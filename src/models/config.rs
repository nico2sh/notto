use std::{collections::HashMap, env, fs::{self, File}, io::{BufReader, BufWriter}, path::{Path, PathBuf}};

use crate::{Notto, errors::NottoError};

const DEFAULT_CONTEXT: &str = "default";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    context: String,
    #[serde(default = "default_contexts")]
    contexts: HashMap<String, ConfigContext>
}

impl Config {
    /// Loads the config from the default file location
    ///
    /// If it doesn't find any file there, creates a new one
    pub fn load_config<P>(base_path: P) -> Result<Config, NottoError> where P: AsRef<Path> {
        let mut config_file_path = PathBuf::from(base_path.as_ref());
        config_file_path.push("config");
        if config_file_path.exists() && !config_file_path.is_file() {
            fs::remove_dir_all(&config_file_path)?;
        }

        if !config_file_path.exists() {
            let config = Config::default();
            Config::save_config_file(&config, &config_file_path)?;
            Ok(config)
        } else {
            let config_file = File::open(config_file_path)?;
            let reader = BufReader::new(config_file);
            match serde_yaml::from_reader(reader) {
                Ok(conf) => Ok(conf),
                Err(e) => {
                    Err(NottoError::LoadConfigError { message: format!("{}", e) } )
                }
            }
        }
    }

    /// Saves the config file to disk
    ///
    /// Fails on IO operations
    pub fn save_config_file(&self, config_file_path: &PathBuf) -> Result<(), NottoError> {
        let file = File::create(config_file_path)?;
        let writer = BufWriter::new(file);
        match serde_yaml::to_writer(writer, &self) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(NottoError::LoadConfigError { message: format!("{}", e) } )
            }
        }
    }

    /// Gets the editor for the current context
    ///
    /// Defaults to the context default
    pub fn get_editor(&self) -> Result<String, NottoError> {
        let context = self.get_context()?;
        self.get_editor_from(context)
    }

    fn get_editor_from<S>(&self, context: S) -> Result<String, NottoError> where S: AsRef<str> {
        match self.get_config_context(&context)?.editor {
            Some(editor) => Ok(editor),
            None => {
                if context.as_ref() == DEFAULT_CONTEXT {
                    Config::default_editor()   
                } else {
                    self.get_editor_from(DEFAULT_CONTEXT)
                }
            }
        }
    }

    /// Gets the notes directory for the current context
    ///
    /// Defaults to the default directory
    pub fn get_notes_dir(&self) -> Result<PathBuf, NottoError> {
        let context = self.get_context()?;
        self.get_notes_dir_from(context)
    }

    pub fn get_notes_dir_from<S>(&self, context: S) -> Result<PathBuf, NottoError> where S: AsRef<str> {
        match &self.get_config_context(&context)?.base_dir {
            Some(base_dir) => Ok(base_dir.clone()),
            None => {
                if context.as_ref() == DEFAULT_CONTEXT {
                    Config::default_directory()   
                } else {
                    self.get_notes_dir_from(DEFAULT_CONTEXT)
                }
            }
        }
    }

    /// Gets the current context
    ///
    /// Tries to get it first form the env vatiable, then in the config file, finally defaults to `default`
    fn get_context(&self) -> Result<String, NottoError> {
        match env::var("NOTTO_CONFIG") {
            Ok(context) => Ok(context),
            Err(e) => {
                match e {
                    env::VarError::NotPresent => if self.context.is_empty() { Ok(DEFAULT_CONTEXT.to_string()) } else { Ok(self.context.clone()) },
                    env::VarError::NotUnicode(_) => {
                        Err(e.into())
                    }
                }
            }
        }
    }

    fn get_config_context<S>(&self, context: S) -> Result<ConfigContext, NottoError> where S: AsRef<str> {
        if let Some(context) = self.contexts.get(context.as_ref()) {
            Ok(context.clone())
        } else {
            Err(NottoError::ContextNotFound { context: context.as_ref().to_owned() } )
        }
    }

    fn default_editor() -> Result<String, NottoError> {
        let editor = env::var("EDITOR")?;
        Ok(editor)
    }

    fn default_directory() -> Result<PathBuf, NottoError> {
        let mut home_dir = Notto::get_home_dir()?;
        home_dir.push("notes");
        let home_path = home_dir.as_path();
        if !home_path.exists() {
            fs::create_dir_all(home_path)?;
        } else {
            if !home_path.is_dir() {
                fs::remove_file(home_path)?;
            }
        }

        Ok(home_dir.to_path_buf())
    }
}

impl Default for Config {
    fn default() -> Self {
        let contexts = default_contexts();
        Self {
            context: DEFAULT_CONTEXT.to_string(),
            contexts: contexts
        }
    }
}

fn default_contexts() -> HashMap<String, ConfigContext> {
    let mut contexts = HashMap::new();
    let context = ConfigContext::default();
    contexts.insert(DEFAULT_CONTEXT.to_string(), context);

    contexts
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct ConfigContext {
    editor: Option<String>,
    base_dir: Option<PathBuf>
}

impl Default for ConfigContext {
    fn default() -> Self {
        Self {
            editor: None, 
            base_dir: None
        }
    }
}
