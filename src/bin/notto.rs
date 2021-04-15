
use std::{fs, path::{Path, PathBuf}};
use std::sync::{Arc, Mutex};

use console::Term;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;

use clap::{App, Arg, ArgMatches};
use notto::Notto;
use notto::models::note::Note;
use notto::errors::NottoError;
use notto::finder::FindCondition;
use notto::finder::NoteFindMessage;

fn main() {
    let matches = App::new("notto")
        .author("Nico")
        .subcommand(App::new("new")
            .about("Creates a new note, you can add the note name in the note hierarchy. Examples:\n`notto new`\n`notto new meeting_minutes`\n`notto new work/resources`")
            .arg(Arg::new("name")
                .about("Name of the note file, no need to add extension, you can use the note/subnote/notename to nest notes")
                .index(1))
            .arg(Arg::new("journal")
                .about("Add an entry under a Y/M/D directory structure")
                .short('j')
                .long("journal")
                .required(false)
                .takes_value(false))
            )
        .subcommand(App::new("open")
            .about("Opens a note"))
        .subcommand(App::new("find")
            .about("Finds a note")
            .arg(Arg::new("text")
                .about("Text to find in the note")
                .index(1)
                .required(true)))
        .get_matches();

    match matches.subcommand() {
        Some(("new", matches)) => new(matches),
        Some(("open", matches)) => { open(matches); },
        Some(("find", matches)) => { find(matches); },
        Some(_) => {}
        None => {}
    };
}

fn new(matches: &ArgMatches) {
    let notto = Notto::new().unwrap();

    let note_name = matches.value_of("name");

    if matches.is_present("journal") {
        match notto.create_journal_entry(note_name) {
            Ok(path) => println!("Saved note at {}", path.to_string_lossy()),
            Err(e) => println!("Error creating note: {}", e)
        }
    } else {
        match notto.create_or_open_note_at(note_name) {
            Ok(path) => println!("Saved note at {}", path.to_string_lossy()),
            Err(e) => println!("Error creating note: {}", e)
        }
    }
}

fn open(matches: &ArgMatches) -> Result<(), NottoError> {
    let notto = Notto::new()?;
    let path_to_list = notto.config.get_notes_dir()?;

    if let Some(note_path) = display_selection_for_path(path_to_list, PathBuf::new())? {
        notto.open_by_path(note_path)?;
    }

    Ok(())
}

fn find(matches: &ArgMatches) -> Result<(), NottoError> {
    let notto = Notto::new()?;

    let find_text = matches.value_of("text");

    if let Some(text) = find_text {
        let mut conditions = vec![];
        conditions.push(FindCondition::Text(text.to_string()));
        let rx = notto.find(conditions)?;

        let theme = ColorfulTheme::default();
        let mut selection = Select::with_theme(&theme);

        match rx.try_recv() {
            Ok(msg) => {
                if let NoteFindMessage::Result(note_result) = msg {
                    selection.item(PathEntry {
                        name: note_result.note.get_title(),
                        path: note_result.path,
                        is_dir: false});
                }
            },
            Err(e) => {}
        }

        selection.interact_on_opt(&Term::stderr())?; 
    }

    Ok(())
}

fn display_selection_for_path<P, A>(base_path: P, path: A) -> Result<Option<PathBuf>, NottoError> where P: AsRef<Path>, A: AsRef<Path> {
    let full_path = base_path.as_ref().join(&path);
    let items = get_selections_for_path(full_path)?;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(path.as_ref().to_string_lossy())
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())?;
    
    if let Some(select) = selection {
        if let Some(item) = items.get(select) {
            let path = &item.path;
            let p = path.strip_prefix(&base_path).unwrap();
            if path.is_dir() {
                return display_selection_for_path(base_path, p)
            } else {
                return Ok(Some(PathBuf::from(p)));
            }
        }
    }

    Ok(None)
}

fn get_selections_for_path<P>(path: P) -> Result<Vec<PathEntry>, NottoError> where P: AsRef<Path> {
    let mut result = vec![];
    for path in fs::read_dir(path)? {
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
                        result.push(PathEntry { name, path, is_dir: true });
                    }
                } else {
                    if let Ok(note_text) = fs::read_to_string(&path) {
                        let note = Note::from_text(note_text);
                        let mut name = note.get_title();

                        if let Some(file_name) = path.file_name() {
                            name = format!("{} [{}]", name, file_name.to_string_lossy());
                        }
                        result.push(PathEntry { name, path, is_dir: false });
                    }
                }
            }
        };
    }

    result.sort();

    Ok(result)
}

#[derive(Debug, Eq, PartialEq, Ord)]
struct PathEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
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
        let self_dir = self.path.is_dir();
        let other_dir = other.path.is_dir();

        if self_dir == other_dir {
            if let (Ok(s), Ok(o)) = (self.name.parse::<i32>(), other.name.parse::<i32>()) {
                s.partial_cmp(&o)
            } else {
                self.name.partial_cmp(&other.name)
            }
        } else {
            if self_dir { Some(std::cmp::Ordering::Less ) } else { Some(std::cmp::Ordering::Greater ) }
        }
    }
}