


use clap::{App, Arg};
use notto::Notto;

fn main() {
    let matches = App::new("notto")
        .author("Nico")
        .arg(Arg::new("journal")
            .about("Add an entry under a Y/M/D directory structure")
            .short('j')
            .long("journal")
            .takes_value(false))
        .get_matches();

    let notto = Notto::new().unwrap();

    if matches.is_present("journal") {
        match notto.create_journal_entry() {
            Ok(_) => {}
            Err(e) => println!("Error creating note: {}", e)
        }
    }
}
