use std::io::Write;
use std::process::{Command, Stdio};
use colored::Colorize;
use crossterm::execute;
use scraper::{Html, Selector};
use crate::State;
use crossterm::cursor::MoveTo;
use crossterm::terminal::{Clear, ClearType};

pub async fn search_results(query: Option<String>) -> State {

    // let correct_query = query.unwrap().to_lowercase().replace(" ", "+").to_string();
    let url;
    if let Some(query) = query.clone() {
        url = format!("https://www.animeworld.ac/search?keyword={}", query.clone().to_lowercase().replace(" ", "+").to_string())
    } else {
        url = "https://www.animeworld.ac".to_string();
    }
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();

    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
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