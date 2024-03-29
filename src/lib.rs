use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Metadata {
    hash: String,
    last_snapshot: Option<String>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            hash: rand_string(),
            last_snapshot: None,
        }
    }

    pub fn get_last_snapshot(&self) -> &str {
        match &self.last_snapshot {
            Some(s) => s,
            None => "",
        }
    }

    pub fn get_instance(&self) -> &str {
        &self.hash
    }

    pub fn set_last_snapshot(&mut self, date: &str) {
        self.last_snapshot = Some(date.to_owned());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    id: String,
    instance: String,
    title: String,
    text: String,
    date: String,
    labels: Vec<String>,
}

impl Note {
    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_instance(&self) -> &str {
        &self.instance
    }
    pub fn get_date(&self) -> &str {
        &self.date
    }
    pub fn get_tags(&self) -> &Vec<String> {
        &self.labels
    }
    pub fn has_tag(&self, tag: &str) -> bool {
        self.labels.contains(&tag.to_owned())
    }

    pub fn format(&self, with_colors: bool) -> String {
        if with_colors {
            return format!(
                "\x1B[38;5;6m[{}]\x1B[39m \x1B[1m{} ({}) \x1B[0m\n\t\x1B[38;5;2m{}\x1B[39m \x1B[38;5;8m#{}\n",
                self.date, self.title, self.id,
                self.text.replace('\n', "\n\t"),
                self.labels.join(" #"),
            );
        }
        format!("[{}] {}\n  {}\n", self.date, self.title, self.text,)
    }

    pub fn new(
        id: String,
        instance: String,
        title: String,
        text: String,
        date: String,
        labels: Vec<String>,
    ) -> Note {
        Note {
            id,
            instance,
            title,
            text,
            date,
            labels,
        }
    }

    pub fn new_from_content(content: &str, instance: &str) -> Note {
        let labels_str = capture_string_by_regex(content, r"(?m).*Labels: ?.*$", 0);
        let labels = Regex::new(r"#([\w-]+)")
            .unwrap()
            .captures_iter(&labels_str)
            .map(|m| m.get(1).unwrap().as_str().to_owned())
            .collect::<Vec<String>>();

        Note::new(
            capture_string_by_regex(content, r"(?m)ID: ?(.*)$", 1),
            instance.to_owned(),
            capture_string_by_regex(content, r"(?m)Title: ?(.*)$", 1),
            capture_string_by_regex(content, r"(?ms).*Text: ?(.*)Date:", 1),
            capture_string_by_regex(content, r"(?m).*Date: ?(.*)$", 1),
            labels,
        )
    }
}

pub fn home_path() -> PathBuf {
    dirs::home_dir().unwrap()
}

pub fn path_exists(path: &str) -> bool {
    home_path().join(Path::new(path)).is_file()
}

pub fn initial_note(tmp: &mut NamedTempFile, instance: &str) -> std::io::Result<()> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    show_existed_note(
        tmp,
        &Note::new(
            rand_string(),
            instance.to_owned(),
            "".to_string(),
            "".to_string(),
            now,
            Vec::new(),
        ),
    )
}

pub fn show_existed_note(tmp: &mut NamedTempFile, note: &Note) -> std::io::Result<()> {
    tmp.write_all(
        format!(
            "ID: {}\n\n---\n\nTitle: {}\n\nText: {}\n\nDate: {}\n\nLabels: #{}",
            note.id,
            note.title,
            note.text,
            note.date,
            note.labels.join(", #"),
        )
        .as_bytes(),
    )?;
    Ok(())
}

pub fn read_data<T: DeserializeOwned>(file: &str) -> T {
    let path = home_path().join(Path::new(file));
    let rf = File::open(path).unwrap();
    let reader = BufReader::new(rf);
    let data: T = serde_json::from_reader(reader).unwrap();
    data
}

pub fn update_notes_with_content(file: &str, content: &str, instance: &str) -> std::io::Result<()> {
    let note = Note::new_from_content(content, instance);

    if !note.title.is_empty() && !note.text.is_empty() {
        let mut notes: Vec<Note> = read_data(file);
        notes.push(note);

        save_data(file, notes)?;
    }

    Ok(())
}

pub fn save_data<T: Serialize>(file: &str, data: T) -> std::io::Result<()> {
    let path = home_path().join(Path::new(file));
    let data_str = serde_json::to_string_pretty(&data).unwrap();
    let mut wf = File::create(path)?;
    return wf.write_all(data_str.as_bytes());
}

fn capture_string_by_regex(content: &str, expression: &str, index: usize) -> String {
    let re = Regex::new(expression).unwrap();
    match re.captures(content) {
        Some(group) => group.get(index).unwrap().as_str().trim().to_owned(),
        None => "".to_owned(),
    }
}

fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_metadata() {
        let meta = Metadata::new();
        assert_eq!(meta.hash.len(), 10);
    }

    #[test]
    fn rand_string_is_not_repeated() {
        let first = rand_string();
        let second = rand_string();

        assert_eq!(first.len(), second.len());
        assert_ne!(first, second);
    }

    #[test]
    fn capture_string_by_regex_second_values() {
        let result = capture_string_by_regex("Key = 100", r"(\w+) = (\d+)", 2);
        assert_eq!(result, "100");
    }

    #[test]
    fn make_initial_note() {
        let mut tmp = NamedTempFile::new().unwrap();
        initial_note(&mut tmp, "instance").unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        let note = Note::new_from_content(&content, "instance");

        assert!(!note.get_id().is_empty());
        assert_eq!(
            note.get_date()[..10],
            Utc::now().format("%Y-%m-%d").to_string()
        );
        assert_eq!(note.get_tags().len(), 0);
    }

    #[test]
    fn test_read_notes() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(
            "[
                {
                    \"id\": \"iomYID8t2A\",
                    \"instance\": \"instance\",
                    \"title\": \"Programming thoughts\",
                    \"text\": \"I created a new tool for notes. Think about improvements in it.\",
                    \"date\": \"2021-08-14 11:43:30\",
                    \"labels\": [
                        \"rust\",
                        \"programming\"
                    ]
                }
            ]"
            .as_bytes(),
        )
        .unwrap();

        let notes: Vec<Note> = read_data(tmp.path().to_str().unwrap());
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Programming thoughts");
        assert!(notes[0].has_tag("rust"));

        update_notes_with_content(
            tmp.path().to_str().unwrap(),
            "
            ID: PEg1HdCLos
            ---
            Title: Second entry
            Text: A lot of text...
            Date: 2021-11-28 18:18:03
            Labels: #empty #thoughts
        ",
            "instance",
        )
        .unwrap();

        let notes: Vec<Note> = read_data(tmp.path().to_str().unwrap());
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[1].title, "Second entry");
        assert_eq!(notes[1].instance, "instance");
        assert!(notes[1].has_tag("thoughts"));
    }
}
