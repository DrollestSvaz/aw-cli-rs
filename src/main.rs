use text_io::scan;
use scraper::{ Selector, Html };
use std::process::{Command, Stdio};
use std::io::Write;
use colored::*;
enum State {
    SearchInput,
    SearchResults(String),
    EpisodeList(String, String),
    Playing(String, usize, Vec<String>),
    PostPlay(String, usize, Vec<String>),
}

#[tokio::main]
async fn main() {
    let mut state = State::SearchInput;
    print!("\x1B[2J\x1B[1;1H");

    loop {
        state = match state {
            State::SearchInput => search_input().await,
            State::SearchResults(query) => search_results(query).await,
            State::EpisodeList(anime, href) => episode_list(anime, href).await,
            State::Playing(url, index, indirizzi) => play(url, index, indirizzi).await,
            State::PostPlay(url, index_ep , indirizzi) => post_play(url, index_ep, indirizzi).await,
        }
    }
}

async fn search_input() -> State {
    let mut title: String = String::new();
    println!("{}", "Cerca un anime per titolo:".bold().on_white().truecolor(37, 37, 37));
    std::io::stdin() // Get the standard input stream
        .read_line(&mut title) // The read_line function reads data until it reaches a '\n' character
        .expect("Unable to read Stdin"); // In case the read operation fails, it panics with the given message

    State::SearchResults(title.trim().to_string())
}

async fn search_results(query: String) -> State {
    let correct_query = query.to_lowercase().replace(" ", "+").to_string();
    print!("\x1B[2J\x1B[1;1H");

    let url = format!("https://www.animeworld.ac/search?keyword={}", correct_query);
    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
    print!("\x1B[2J\x1B[1;1H");
    let doc = Html::parse_document(&resp);
    let selector = Selector::parse("div.film-list div.item div.inner a.name").unwrap();
    let mut titles: Vec<String> = Vec::new();
    let mut addresses: Vec<String> = Vec::new();
    for element in doc.select(&selector) {
        titles.push(element.inner_html());
        addresses.push(element.attr("href").unwrap().to_string());
    }


    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(titles.join("\n").as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    let scelta = String::from_utf8(output.stdout).unwrap().trim().to_string();

    let index = titles.iter().position(|t| t == &scelta).unwrap();
    let indirizzo = addresses[index].clone();
    State::EpisodeList(query, indirizzo.to_string())
}

async fn episode_list(anime: String, link: String) -> State {
    let url = format!("https://www.animeworld.ac{}", link);
    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
    print!("\x1B[2J\x1B[1;1H");
    let doc = Html::parse_document(&resp);
    let selector = Selector::parse("li.episode a").unwrap();
    let mut episodes: Vec<String> = Vec::new();
    let mut addresses: Vec<String> = Vec::new();
    for element in doc.select(&selector) {
        episodes.push(format!("Episodio: {}", element.inner_html()));
        addresses.push(element.attr("href").unwrap().to_string());
    }


    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(episodes.join("\n").bold().as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    let scelta = String::from_utf8(output.stdout).unwrap().trim().to_string();
    let index = episodes.iter().position(|t| t == &scelta).unwrap();
    let indirizzo = addresses[index].clone();
    State::Playing(indirizzo.to_string(), index, addresses)
}

async fn play(url: String, index: usize, indirizzi: Vec<String>) -> State {
    let url = format!("https://www.animeworld.ac{}", url);
    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url.clone()).await.unwrap().text().await.unwrap();
    print!("\x1B[2J\x1B[1;1H");
    let doc = Html::parse_document(&resp);
    let selector = Selector::parse("a#alternativeDownloadLink").unwrap();
    let video_url = doc.select(&selector)
        .next()
        .unwrap()
        .attr("href")
        .unwrap()
        .to_string();

    Command::new("vlc")
        .arg(&video_url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    State::PostPlay(url, index, indirizzi)
}

async fn post_play(mut url:String, index_ep: usize, indirizzi: Vec<String>) -> State {
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
            url = indirizzi[index_ep + 1].clone();
            State::Playing(url, index_ep + 1, indirizzi)
        },
        "Riguarda" => {
            url = indirizzi[index_ep].clone();
            State::Playing(url, index_ep, indirizzi)
        },
        "Precedente" => {
            url = indirizzi[index_ep - 1].clone();
            State::Playing(url, index_ep - 1, indirizzi)
        },
        "Esci" => std::process::exit(0),
        "Cambia anime" => State::SearchInput,
        _ => std::process::exit(0)
    }
}