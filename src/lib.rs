use std::path::PathBuf;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    hash: String,
}

impl Metadata {
    pub fn new(hash: String) -> Metadata {
        Metadata {
            hash: hash,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    id: String,
    title: String,
    text: String,
    date: String,
    labels: Vec<String>,
}

impl Note {
    pub fn new(title: String, text: String, date: String, labels: Vec<String>) -> Note {
        Note {
            id: rand_string(),
            title: title,
            text: text,
            date: date,
            labels: labels,
        }
    }
}

pub fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

pub fn home_path() -> PathBuf {
    dirs::home_dir().unwrap()
}
