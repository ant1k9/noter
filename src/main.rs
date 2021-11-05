use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use clap::{App, Arg};
use tempfile::NamedTempFile;

const METADATA_FILE: &str = ".noter/metadata/metadata.json";
const DATA_FILE: &str = ".noter/notes/data.json";
const REMOTE_DATA_FILE: &str = ".noter/notes/remote.data.json";

const DEFAULT_LIST_LIMIT: usize = 10;

fn add_note() -> std::io::Result<()> {
    let path = noter::home_path().join(Path::new(DATA_FILE));
    if !path.exists() {
        let mut f = File::create(path.to_owned())?;
        f.write_all(b"[]")?;
    }

    edit_and_save(None)
}

fn compact() -> std::io::Result<()> {
    let notes = noter::read_notes(DATA_FILE);
    let mut compacted: Vec<noter::Note> = Vec::new();

    for note in notes.iter().rev() {
        let mut found = false;
        for selected in compacted.iter().rev() {
            if note.get_id() == selected.get_id() {
                found = true;
                break;
            }
        }
        if !found {
            compacted.push(note.to_owned());
        }
    }

    compacted.reverse();
    return noter::save_notes(DATA_FILE, compacted);
}

fn merge() -> std::io::Result<()> {
    let notes = noter::read_notes(DATA_FILE);
    let remote_notes = noter::read_notes(REMOTE_DATA_FILE);
    let mut merged: Vec<noter::Note> = Vec::new();

    let mut i: usize = 0;
    let mut j: usize = 0;
    let k = notes.len() + remote_notes.len();

    while i + j < k {
        if i == notes.len()
            || (j < remote_notes.len() && remote_notes[j].get_date() < notes[i].get_date())
        {
            merged.push(remote_notes[j].to_owned());
            j += 1;
        } else {
            merged.push(notes[i].to_owned());
            i += 1;
        }
    }

    noter::save_notes(DATA_FILE, merged)?;
    return compact();
}

fn edit() -> std::io::Result<()> {
    let id: String = env::args().nth(2).unwrap();

    for note in noter::read_notes(DATA_FILE).iter().rev() {
        if note.get_id() == id {
            return edit_and_save(Some(note));
        }
    }

    Ok(())
}

fn edit_and_save(opt: Option<&noter::Note>) -> std::io::Result<()> {
    let mut tmp = NamedTempFile::new()?;

    match opt {
        Some(note) => noter::show_existed_note(&mut tmp, &note)?,
        None => noter::initial_note(&mut tmp)?,
    }

    Command::new("vim")
        .arg(tmp.path())
        .status()
        .expect("editor failed to start");

    let content = fs::read_to_string(tmp.path().to_owned())?;
    noter::update_notes_with_content(DATA_FILE, content)
}

fn init() -> std::io::Result<()> {
    let home = noter::home_path();
    let path = home.join(Path::new(METADATA_FILE));
    if path.exists() {
        return Ok(());
    }

    let metadata = noter::Metadata::new();
    let s = serde_json::to_string_pretty(&metadata).unwrap();

    create_dir_all(home.join(Path::new("./.noter/metadata")))?;
    create_dir_all(home.join(Path::new("./.noter/notes")))?;

    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())?;
    return Ok(());
}

fn list(tag: &str, with_colors: bool) -> std::io::Result<()> {
    let mut n: usize = DEFAULT_LIST_LIMIT;
    if env::args().len() > 1 {
        n = env::args()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap_or(DEFAULT_LIST_LIMIT);
    }

    let mut listed: HashSet<String> = HashSet::new();
    for note in noter::read_notes(DATA_FILE).iter().rev() {
        if n == 0 {
            break;
        }
        if listed.contains(note.get_id()) {
            continue;
        }
        if tag != "" && !note.has_tag(tag) {
            continue;
        }

        println!("{}", note.format(with_colors));
        listed.insert(note.get_id().to_string());
        n -= 1;
    }

    Ok(())
}

fn remove() -> std::io::Result<()> {
    let id: String = env::args().nth(2).unwrap();

    let notes = noter::read_notes(DATA_FILE)
        .into_iter()
        .filter(|note| note.get_id() != id)
        .collect::<Vec<noter::Note>>();

    return noter::save_notes(DATA_FILE, notes);
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Noter")
        .subcommand(App::new("add").about("opens a vim editor to create a new node"))
        .subcommand(App::new("compact").about("remove staled versions and edits"))
        .subcommand(
            App::new("edit")
                .about("edit a note (needs a hash as an argument)")
                .arg(Arg::new("").takes_value(true)),
        )
        .subcommand(App::new("init").about("initialize folders and directories for noter"))
        .subcommand(
            App::new("remove")
                .about("remove a note (needs a hash as an argument)")
                .arg(Arg::new("").takes_value(true)),
        )
        .subcommand(App::new("sync").about("sync with remote file"))
        .arg(Arg::new("").takes_value(true))
        .arg(
            Arg::new("tag")
                .about("filter notes by given tag")
                .long("--tag")
                .takes_value(true),
        )
        .arg(
            Arg::new("no-colors")
                .about("show notes without colorizing")
                .long("--no-colors"),
        )
        .get_matches();

    if let Some(_) = matches.subcommand_matches("compact") {
        compact()?;
    } else if let Some(_) = matches.subcommand_matches("edit") {
        edit()?;
    } else if let Some(_) = matches.subcommand_matches("init") {
        init()?;
    } else if let Some(_) = matches.subcommand_matches("add") {
        add_note()?;
    } else if let Some(_) = matches.subcommand_matches("remove") {
        remove()?;
    } else if let Some(_) = matches.subcommand_matches("sync") {
        merge()?;
    } else {
        list(
            matches.value_of("tag").unwrap_or(""),
            !matches.is_present("no-colors"),
        )?;
    }

    Ok(())
}
