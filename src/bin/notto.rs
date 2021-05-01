mod ui;

use std::{fs, path::{Path, PathBuf}};
use std::sync::{Arc, Mutex};

use console::Term;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;

use clap::{App, Arg, ArgMatches};
use notto::{Notto, io::browser::NottoPath};
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

    if let Some(note_path) = display_selection_for_path(&notto, &NottoPath::new())? {
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
                    /*selection.item(PathEntry {
                        name: note_result.note.get_title(),
                        path: note_result.path,
                        is_dir: false});*/
                }
            },
            Err(e) => {}
        }

        selection.interact_on_opt(&Term::stderr())?; 
    }

    Ok(())
}

fn display_selection_for_path(notto: &Notto, path: &NottoPath) -> Result<Option<NottoPath>, NottoError> {
    let path_string: String = path.into();
    let items = notto.browse(path)?;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(path_string)
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())?;
    
    if let Some(select) = selection {
        if let Some(item) = items.get(select) {
            let path = &item.path;
            if item.is_dir() {
                return display_selection_for_path(notto, path)
            } else {
                return Ok(Some(path.clone()));
            }
        }
    }

    Ok(None)
}
