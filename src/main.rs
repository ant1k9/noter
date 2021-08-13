use std::path::Path;
use std::fs;
use std::fs::{ File, create_dir_all };
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command};

use regex::Regex;
use chrono::{Utc};
use clap::{ App };
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{ Deserialize, Serialize };
use tempfile::NamedTempFile;

const METADATA_FILE: &str = ".noter/metadata/metadata.json";
const DATA_FILE: &str = ".noter/notes/data.json";

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    text: String,
    date: String,
    labels: Vec<String>,
}

fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

fn init() -> std::io::Result<()> {
    let home = dirs::home_dir().unwrap();
    let path = home.join(Path::new(METADATA_FILE));
    if path.exists() {
        return Ok(());
    }

    let metadata = Metadata {
        hash: rand_string(),
    };
    let s = serde_json::to_string_pretty(&metadata).unwrap();

    create_dir_all(home.join(Path::new("./.noter/metadata")))?;
    create_dir_all(home.join(Path::new("./.noter/notes")))?;

    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())?;
    return Ok(());
}

fn add_note() -> std::io::Result<()> {
    let home = dirs::home_dir().unwrap();
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

    let note = Note {
        title: title_re.captures(&content).unwrap().get(1).unwrap().as_str().trim().to_owned(),
        text: text_re.captures(&content).unwrap().get(1).unwrap().as_str().trim().to_owned(),
        date: date_re.captures(&content).unwrap().get(1).unwrap().as_str().trim().to_owned(),
        labels: labels,
    };

    let rf = File::open(path.clone())?;
    let reader = BufReader::new(rf);
    let mut notes: Vec<Note> = serde_json::from_reader(reader).unwrap();
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
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("edit") {
        println!("TODO: edit");
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("init") {
        init()?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("list") {
        println!("TODO: list");
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("sync") {
        println!("TODO: sync");
        return Ok(());
    }

    add_note()
}
