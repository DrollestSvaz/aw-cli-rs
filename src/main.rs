use scraper::{ Selector, Html };
use std::process::{Command, Stdio};
use std::io::Write;
use colored::*;
enum State {
    SearchInput,
    SearchResults(Option<String>),
    EpisodeList(String, String),
    Playing(String, usize, Vec<String>),
    PostPlay(String, usize, Vec<String>),
}

#[tokio::main]
async fn main() {


    let args: Vec<String> = std::env::args().collect();

    let initial_state = if args.contains(&"--news".to_string()) {
        State::SearchResults(Option::from(None))
    } else if let Some(i) = args.iter().position(|a| a == "--genre") {
        match args.get(i + 1) {
            Some(genre) => State::SearchResults(Option::from(genre.to_string())),
            None => {
                eprintln!("Specificare un genere dopo --genre");
                std::process::exit(1);
            }
        }
    } else {
        State::SearchInput
    };


    let mut state = initial_state;
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

    State::SearchResults(Option::from(title.trim().to_string()))
}

async fn search_results(query: Option<String>) -> State {

    // let correct_query = query.unwrap().to_lowercase().replace(" ", "+").to_string();
    let url;
    if let Some(query) = query.clone() {
        url = format!("https://www.animeworld.ac/search?keyword={}", query.clone().to_lowercase().replace(" ", "+").to_string())
    } else {
        url = "https://www.animeworld.ac".to_string();
    }
    print!("\x1B[2J\x1B[1;1H");

    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
    print!("\x1B[2J\x1B[1;1H");
    let doc = Html::parse_document(&resp);
    let selector;
    if query.is_some() {
        selector = Selector::parse("div.film-list div.item div.inner a.name").unwrap();
    } else {
        selector = Selector::parse("div.content:not(.hidden)[data-name=\"all\"] div.page div.film-list div.item div.inner a.name").unwrap();
    }
    let mut titles: Vec<String> = Vec::new();
    let mut addresses: Vec<String> = Vec::new();
    for element in doc.select(&selector) {
        titles.push(element.inner_html());
        addresses.push(element.attr("href").unwrap().to_string());
    }
    if titles.is_empty() {
        return State::SearchInput
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
    State::EpisodeList(query.unwrap_or_default(), indirizzo.to_string())
}

async fn episode_list(_anime: String, link: String) -> State {
    let url = format!("https://www.animeworld.ac{}", link);
    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
    print!("\x1B[2J\x1B[1;1H");
    let doc = Html::parse_document(&resp);
    let selector = Selector::parse("div.active ul.episodes li.episode a").unwrap();
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

    State::PostPlay(indirizzi[index].clone(), index, indirizzi)
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