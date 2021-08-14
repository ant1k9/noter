use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::{ File, create_dir_all };
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use clap::{App, Arg};
use tempfile::NamedTempFile;

const METADATA_FILE: &str = ".noter/metadata/metadata.json";
const DATA_FILE: &str = ".noter/notes/data.json";

fn add_note() -> std::io::Result<()> {
    let path = noter::home_path().join(Path::new(DATA_FILE));
    if !path.exists() {
        let mut f = File::create(path.clone())?;
        f.write_all(b"[]")?;
    }

    edit_and_save(None)
}

fn edit() -> std::io::Result<()> {
    let id: String = env::args().nth(2).unwrap();

    for note in noter::read_notes(DATA_FILE).iter().rev() {
        if note.get_id() == id {
            return edit_and_save(Some(note))
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

    let content = fs::read_to_string(tmp.path().clone())?;
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

fn list() -> std::io::Result<()> {
    let mut n: usize = 100;
    if env::args().len() > 2 {
        n = env::args().nth(2).unwrap().parse().unwrap();
    }

    let mut listed: HashSet<String> = HashSet::new();
    for note in noter::read_notes(DATA_FILE).iter().rev() {
        if n == 0 {
            break;
        }
        if listed.contains(note.get_id()) {
            continue;
        }

        println!("{}", note.format());
        listed.insert(note.get_id().to_string());
        n -= 1;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Noter")
        .subcommand(App::new("edit")
            .arg(Arg::new("")
            .takes_value(true)))
        .subcommand(App::new("init"))
        .subcommand(App::new("list")
            .arg(Arg::new("")
            .takes_value(true)))
        .subcommand(App::new("compact"))
        .subcommand(App::new("sync"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("compact") {
        println!("TODO: compact");
    } else if let Some(_) = matches.subcommand_matches("edit") {
        edit()?;
    } else if let Some(_) = matches.subcommand_matches("init") {
        init()?;
    } else if let Some(_) = matches.subcommand_matches("list") {
        list()?;
    } else if let Some(_) = matches.subcommand_matches("sync") {
        println!("TODO: sync");
    } else {
        add_note()?;
    }

    Ok(())
}
