use std::path::Path;
use std::fs;
use std::fs::{ File, create_dir_all };
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command};

use regex::Regex;
use chrono::{Utc};
use clap::{ App };
use tempfile::NamedTempFile;

const METADATA_FILE: &str = ".noter/metadata/metadata.json";
const DATA_FILE: &str = ".noter/notes/data.json";

fn init() -> std::io::Result<()> {
    let home = noter::home_path();
    let path = home.join(Path::new(METADATA_FILE));
    if path.exists() {
        return Ok(());
    }

    let metadata = noter::Metadata::new (
        noter::rand_string()
    );
    let s = serde_json::to_string_pretty(&metadata).unwrap();

    create_dir_all(home.join(Path::new("./.noter/metadata")))?;
    create_dir_all(home.join(Path::new("./.noter/notes")))?;

    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())?;
    return Ok(());
}

fn add_note() -> std::io::Result<()> {
    let home = noter::home_path();
    let path = home.join(Path::new(DATA_FILE));

    if !path.exists() {
        let mut f = File::create(path.clone())?;
        f.write_all(b"[]")?;
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(("Title:

Text:

Date: ".to_owned() + &now + "

Labels:").as_bytes())?;

    Command::new("vim")
        .arg(tmp.path())
        .status()
        .expect("editor failed to start");

    let content = fs::read_to_string(tmp.path().clone())?;

    let title_re = Regex::new(r"(?m)Title: ?(.*)$").unwrap();
    let text_re = Regex::new(r"(?ms).*Text: ?(.*)Date:").unwrap();
    let date_re = Regex::new(r"(?m).*Date: ?(.*)$").unwrap();
    let labels_re = Regex::new(r"(?m).*Labels: ?.*$").unwrap();

    let labels_str = labels_re.captures(&content).unwrap().get(0).unwrap().as_str();
    let mut labels: Vec<String> = Vec::new();
    for m in Regex::new(r"#(\w+)").unwrap().captures_iter(labels_str) {
        labels.push(m.get(1).unwrap().as_str().to_owned());
    }

    let note = noter::Note::new(
        title_re.captures(&content).unwrap().get(1).unwrap().as_str().trim().to_owned(),
        text_re.captures(&content).unwrap().get(1).unwrap().as_str().trim().to_owned(),
        date_re.captures(&content).unwrap().get(1).unwrap().as_str().trim().to_owned(),
        labels,
    );

    let rf = File::open(path.clone())?;
    let reader = BufReader::new(rf);
    let mut notes: Vec<noter::Note> = serde_json::from_reader(reader).unwrap();
    notes.push(note);

    let notes_str = serde_json::to_string_pretty(&notes).unwrap();
    let mut wf = File::create(path)?;
    wf.write_all(notes_str.as_bytes())?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Noter")
        .subcommand(App::new("edit"))
        .subcommand(App::new("init"))
        .subcommand(App::new("list"))
        .subcommand(App::new("compact"))
        .subcommand(App::new("sync"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("compact") {
        println!("TODO: compact");
    } else if let Some(_) = matches.subcommand_matches("edit") {
        println!("TODO: edit");
    } else if let Some(_) = matches.subcommand_matches("init") {
        init()?;
    } else if let Some(_) = matches.subcommand_matches("list") {
        println!("TODO: list");
    } else if let Some(_) = matches.subcommand_matches("sync") {
        println!("TODO: sync");
    } else {
        add_note()?;
    }

    Ok(())
}
