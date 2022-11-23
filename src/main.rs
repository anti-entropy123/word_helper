use inquire::{InquireError, Select};
use lazy_static::lazy_static;
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin::Mutex;
use std::{
    fs::{self, File},
    io::Write,
    process::exit,
};

const WORD_LIST_PATH: &str = "/home/yjn/rust_project/english-tool/word_list.txt";
const SAVE_FILE: &str = "/home/yjn/rust_project/english-tool/data.txt";

lazy_static! {
    static ref CONTEXT: Mutex<ProcessData> = Mutex::from(load_process());
}

#[derive(Serialize, Deserialize, Clone)]
struct Word {
    pub word: String,
    pub translated: String,
}

#[derive(Serialize, Deserialize, Default)]
struct ProcessData {
    pub process: usize,
    pub later: Vec<Word>,
    pub removed: Vec<String>,
}

impl ProcessData {
    pub fn to_json(&self) -> String {
        json!(self).to_string()
    }
}

fn init_word_data() -> Vec<Word> {
    let context = fs::read_to_string(WORD_LIST_PATH).expect("word list file not found.");

    let lines = context.split("\n");
    let mut words = Vec::new();
    for line in lines {
        let mut parts = line.split(" ");
        let word = match (parts.next(), parts.next()) {
            (Some(_), Some(word)) => word,
            _ => panic!("bad word line? line={}", line),
        };
        // println!("the word is {}", word);
        let translated = &line[line.find(word).unwrap() + word.len()..];
        // println!("the translated is {}", translated);
        words.push(Word {
            word: word.to_owned(),
            translated: translated.to_owned(),
        })
    }
    words
}

fn save_process() {
    File::create("data.txt")
        .expect("create failed")
        .write_all(CONTEXT.lock().to_json().as_bytes())
        .expect("failed to save process");

    println!("process data is saved");
    exit(0)
}

fn load_process() -> ProcessData {
    match fs::read_to_string(SAVE_FILE) {
        Ok(val) => serde_json::from_str(&val).unwrap_or(ProcessData::default()),
        _ => ProcessData::default(),
    }
}

fn sub_select(word: &Word) {
    clearscreen::clear().expect("failed to clear screen");
    let sub_options: Vec<String> = vec!["next".to_owned(), "later".to_owned()];
    let ans: Result<String, InquireError> = Select::new(
        &format!("{} -> {}\n\n\n", word.word, word.translated),
        sub_options.clone(),
    )
    .prompt();
    match ans {
        Ok(ans) => match sub_options.iter().enumerate().find(|(_, v)| **v == ans) {
            Some((idx, _)) => match idx {
                0 => {}
                1 => CONTEXT.lock().later.push(word.clone()),
                _ => panic!(),
            },
            None => {}
        },
        _ => {
            save_process();
        }
    }
}

fn learn_a_word(word: &Word) {
    let options: Vec<String> = vec!["show".to_owned(), "next".to_owned(), "remove".to_owned()];
    loop {
        let ans = Select::new(&format!("{}\n\n", &word.word), options.clone()).prompt();
        match ans {
            Ok(ans) => match options.iter().enumerate().find(|(_, v)| **v == ans) {
                Some((idx, _)) => match idx {
                    0 => {
                        sub_select(&word);
                        break;
                    }
                    1 => {
                        break;
                    }
                    2 => {
                        CONTEXT.lock().removed.push(word.word.clone());
                        break;
                    }
                    _ => {
                        panic!()
                    }
                },
                None => {}
            },
            _ => {
                save_process();
            }
        }
    }
}

fn clear_later_word(rng: &mut ThreadRng) {
    let cur_len = CONTEXT.lock().later.len() as f64;
    if cur_len * 0.1 < rng.gen::<f64>() {
        return;
    }
    loop {
        clearscreen::clear().unwrap();
        let word = {
            let ctx = CONTEXT.lock();
            if ctx.later.is_empty() {
                break;
            }
            ctx.later.get(0).unwrap().clone()
        };
        learn_a_word(&word);
        CONTEXT.lock().later.remove(0);
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let words = init_word_data();
    let mut process = CONTEXT.lock().process;

    loop {
        clearscreen::clear().unwrap();
        clear_later_word(&mut rng);
        process = process % words.len();
        CONTEXT.lock().process = process;
        let word = words[process].clone();
        if CONTEXT.lock().removed.contains(&word.word) {
            continue;
        }
        learn_a_word(&word);
        process += 1
    }

    // Text::new("都背完了，你真棒！").prompt().unwrap();
}
