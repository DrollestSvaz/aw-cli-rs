use std::io::Write;
use std::process::{Command, Stdio};
use crate::State;

pub(crate) async fn post_play(mut url:String, index_ep: usize, indirizzi: Vec<String>) -> State {
    let opzioni = vec!["Prossimo", "Precedente", "Riguarda", "Cambia anime", "Esci"];
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(opzioni.join("\n").as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    let scelta = String::from_utf8(output.stdout).unwrap().trim().to_string();

    match scelta.as_str() {
        "Prossimo" => {
            if (index_ep + 1) == indirizzi.len() {
                println!("{}", index_ep);
                State::Playing(url, index_ep, indirizzi.clone())
            } else {
                println!("{}", index_ep);
                url = indirizzi[index_ep + 1].clone();
                State::Playing(url, index_ep + 1, indirizzi)
            }

        },
        "Riguarda" => {
            url = indirizzi[index_ep].clone();
            State::Playing(url, index_ep, indirizzi)
        },
        "Precedente" => {
            if index_ep == 0 {
                State::Playing(url, index_ep, indirizzi.clone())
            } else {
                url = indirizzi[index_ep - 1].clone();
                State::Playing(url, index_ep - 1, indirizzi)
            }

        },
        "Esci" => std::process::exit(0),
        "Cambia anime" => State::SearchInput,
        _ => std::process::exit(0)
    }
}