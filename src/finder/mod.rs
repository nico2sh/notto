use std::{fs, path::{Path, PathBuf}, sync::Arc, thread};
use chrono::{NaiveDate, NaiveTime};
use crossbeam_utils::sync::WaitGroup;
use log::error;
use crossbeam_channel::{Receiver, Sender};

use crate::{errors::NottoError, models::note::Note};

pub enum NoteFindMessage {
    Result(NoteFindResult),
    Finish
}

#[derive(Debug)]
pub struct NoteFindResult {
    note: Note,
    path: PathBuf
}

#[derive(Debug, Clone)]
pub enum TimeFind {
    Before,
    After,
    At
}

#[derive(Debug, Clone)]
pub enum FindCondition {
    Text(String),
    Tag(String),
    Date(TimeFind, NaiveDate),
    Time(TimeFind, NaiveTime),
}

pub struct Finder {
    base_path: PathBuf
}

type Fc = Box<dyn Fn(&Note) -> bool + Send + Sync>;

impl Finder {
    pub fn new(base_path: PathBuf) -> Self { Self { base_path } }

    pub fn find<P>(&self, path: P, conditions: Vec<FindCondition>) -> Result<Receiver<NoteFindMessage>, NottoError> where P: AsRef<Path> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let check_conds = Box::new(move |note: &Note| {
            for cond in &conditions {
                let matched = match cond {
                    FindCondition::Text(text) => {
                        note.content.to_uppercase().find(&text.to_uppercase()).is_some()
                    }
                    FindCondition::Tag(tag) => { false }
                    FindCondition::Date(when, date) => { false }
                    FindCondition::Time(when, time) => { false }
                };

                if !matched {
                    return false;
                }
            }

            true
        });

        let wg = WaitGroup::new();
        Finder::read_dir(self.base_path.join(path), tx.clone(), wg.clone(), Arc::new(check_conds))?;

        wg.wait();
        tx.send(NoteFindMessage::Finish)?;

        Ok(rx)
    }

    fn read_dir<P>(path: P, sender: Sender<NoteFindMessage>, wg: WaitGroup, f: Arc<Fc>) -> Result<(), NottoError> where P: AsRef<Path> {
        for entry in fs::read_dir(path)? {
            let p = entry?.path();
            let f = Arc::clone(&f);
            if p.is_dir() {
                if let Err(e) = Finder::read_dir(p, sender.clone(), wg.clone(), f) { error!("{}", e); }
            } else {
                let tx = sender.clone();
                let wg_cloned = wg.clone();
                thread::spawn(move || {
                    let note_content = fs::read_to_string(&p).unwrap();
                    let note = Note::from_text(note_content);
                    if f(&note) {
                        let note_find_result = NoteFindResult { note, path: p };
                        if let Err(e) = tx.send(NoteFindMessage::Result(note_find_result)) { error!("{}", e); }
                    };

                    drop(wg_cloned);
                });
            }
        };

        drop(wg);

        Ok(())
    }
}