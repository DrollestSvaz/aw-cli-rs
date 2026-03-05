use std::io::Write;
use std::process::{Command, Stdio};
use colored::Colorize;
use crossterm::execute;
use scraper::{Html, Selector};
use crate::State;
use crossterm::terminal::{Clear, ClearType};
use crossterm::cursor::MoveTo;
pub(crate) async fn episode_list(_anime: String, link: String) -> State {
    let url = format!("https://www.animeworld.ac{}", link);
    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
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