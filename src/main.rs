use std::cmp::Ordering;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use clap::{App, Arg};
use tempfile::NamedTempFile;
use terminal_size::{terminal_size, Height, Width};

const METADATA_FILE: &str = ".noter/metadata/metadata.json";
const DATA_FILE: &str = ".noter/notes/data.json";
const REMOTE_DATA_FILE: &str = ".noter/notes/remote.data.json";

const DEFAULT_LIST_LIMIT: usize = 10;

fn add_note(metadata: noter::Metadata) -> std::io::Result<()> {
    let path = noter::home_path().join(Path::new(DATA_FILE));
    if !path.exists() {
        let mut f = File::create(path)?;
        f.write_all(b"[]")?;
    }

    edit_and_save(None, metadata)
}

fn compact() -> std::io::Result<()> {
    let notes: Vec<noter::Note> = noter::read_data(DATA_FILE);
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
    noter::save_data(DATA_FILE, compacted)
}

fn merge(mut metadata: noter::Metadata) -> std::io::Result<()> {
    let notes: Vec<noter::Note> = noter::read_data(DATA_FILE);
    let remote_notes: Vec<noter::Note> = noter::read_data(REMOTE_DATA_FILE);
    let last_snapshot = metadata.get_last_snapshot();

    let mut merged: Vec<noter::Note> = notes.to_vec();
    merged.sort_by_key(|x| x.get_date().to_owned());
    remote_notes
        .iter()
        .filter(|x| x.get_date().cmp(last_snapshot) == Ordering::Greater)
        .for_each(|x| merged.push(x.clone()));

    let mut remote_merged: Vec<noter::Note> = remote_notes.to_vec();
    remote_merged.sort_by_key(|x| x.get_date().to_owned());
    notes
        .iter()
        .filter(|x| x.get_date().cmp(last_snapshot) == Ordering::Greater)
        .for_each(|x| remote_merged.push(x.clone()));

    if !notes.is_empty() {
        merged.sort_by_key(|x| x.get_date().to_owned());
        metadata.set_last_snapshot(merged[merged.len() - 1].get_date());
        noter::save_data(METADATA_FILE, metadata)?;
    }

    noter::save_data(DATA_FILE, merged)?;
    noter::save_data(REMOTE_DATA_FILE, remote_merged)?;
    compact()
}

fn edit(metadata: noter::Metadata) -> std::io::Result<()> {
    let id: String = env::args().nth(2).unwrap();

    let saved_notes: Vec<noter::Note> = noter::read_data(DATA_FILE);
    for note in saved_notes.iter().rev() {
        if note.get_id() == id {
            return edit_and_save(Some(note), metadata);
        }
    }

    Ok(())
}

fn edit_and_save(opt: Option<&noter::Note>, metadata: noter::Metadata) -> std::io::Result<()> {
    let mut tmp = NamedTempFile::new()?;

    match opt {
        Some(note) => noter::show_existed_note(&mut tmp, note)?,
        None => noter::initial_note(&mut tmp, metadata.get_hash())?,
    }

    Command::new("vim")
        .arg(tmp.path())
        .status()
        .expect("editor failed to start");

    let content = fs::read_to_string(tmp.path())?;
    noter::update_notes_with_content(DATA_FILE, &content, metadata.get_hash())
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

    let path = home.join(Path::new(DATA_FILE));
    f = File::create(path)?;
    f.write_all("[]".as_bytes())?;

    Ok(())
}

fn get_tags() -> std::io::Result<()> {
    let mut listed: Vec<String> = Vec::new();

    let saved_notes: Vec<noter::Note> = noter::read_data(DATA_FILE);
    for note in saved_notes.iter().rev() {
        for tag in note.get_tags() {
            if listed.contains(tag) {
                continue;
            }
            listed.push(tag.to_owned());
        }
    }

    listed.sort();
    for tag in listed {
        println!("{}", tag);
    }

    Ok(())
}

fn list(tag: &str, with_colors: bool) -> std::io::Result<()> {
    let mut n: usize = DEFAULT_LIST_LIMIT;
    let mut terminal_width: i32 = 100;
    let mut max_lines: i32 = 10000;

    if env::args().len() > 1 {
        n = env::args()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap_or(DEFAULT_LIST_LIMIT);
    } else if let Some((Width(w), Height(h))) = terminal_size() {
        terminal_width = w as i32;
        max_lines = h as i32;
    }

    let mut listed: HashSet<String> = HashSet::new();
    let mut notes: Vec<String> = Vec::new();

    let saved_notes: Vec<noter::Note> = noter::read_data(DATA_FILE);
    for note in saved_notes.iter().rev() {
        if listed.contains(note.get_id()) {
            continue;
        }
        if !tag.is_empty() && !note.has_tag(tag) {
            continue;
        }
        notes.push(note.format(with_colors).to_string());
        listed.insert(note.get_id().to_string());
    }

    notes.sort_by(|a, b| b.cmp(a));
    for note in notes.iter().take(n) {
        if max_lines <= 0 {
            break;
        }
        // 2 lines for header and footer break
        max_lines -= 2;
        for line in note.split('\n') {
            max_lines -= (line.len() + terminal_width as usize - 1) as i32 / terminal_width;
        }
        println!("{}", note);
    }

    Ok(())
}

fn remove() -> std::io::Result<()> {
    let id: String = env::args().nth(2).unwrap();

    let saved_notes: Vec<noter::Note> = noter::read_data(DATA_FILE);
    let notes = saved_notes
        .into_iter()
        .filter(|note| note.get_id() != id)
        .collect::<Vec<noter::Note>>();

    noter::save_data(DATA_FILE, notes)
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
        .subcommand(
            App::new("rm")
                .about("remove a note (needs a hash as an argument)")
                .arg(Arg::new("").takes_value(true)),
        )
        .subcommand(App::new("sync").about("sync with remote file"))
        .subcommand(App::new("tags").about("show present tags in notes"))
        .arg(Arg::new("").takes_value(true))
        .arg(
            Arg::new("tag")
                .long("--tag")
                .long_help("filter notes by given tag")
                .takes_value(true),
        )
        .arg(
            Arg::new("no-colors")
                .long("--no-colors")
                .long_help("show notes without colorizing"),
        )
        .get_matches();

    if !noter::path_exists(METADATA_FILE) {
        init()?;
    }

    let metadata: noter::Metadata = noter::read_data(METADATA_FILE);

    if matches.subcommand_matches("compact").is_some() {
        compact()?;
    } else if matches.subcommand_matches("edit").is_some() {
        edit(metadata)?;
    } else if matches.subcommand_matches("add").is_some() {
        add_note(metadata)?;
    } else if matches.subcommand_matches("rm").is_some() {
        remove()?;
    } else if matches.subcommand_matches("sync").is_some() {
        merge(metadata)?;
    } else if matches.subcommand_matches("tags").is_some() {
        get_tags()?;
    } else {
        list(
            matches.value_of("tag").unwrap_or(""),
            !matches.is_present("no-colors"),
        )?;
    }

    Ok(())
}
