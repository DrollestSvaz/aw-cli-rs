use std::process::{Command, Stdio};
use colored::Colorize;
use crossterm::execute;
use scraper::{Html, Selector};
use crate::{vlc_path, State};
use crossterm::terminal::{Clear, ClearType};
use crossterm::cursor::MoveTo;

pub async fn play(url: String, index: usize, indirizzi: Vec<String>) -> State {
    let url = format!("https://www.animeworld.ac{}", url);
    println!("{}", "Caricamento...".bold().on_white().truecolor(37, 37, 37));
    let resp = reqwest::get(url.clone()).await.unwrap().text().await.unwrap();
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
    let doc = Html::parse_document(&resp);
    let selector = Selector::parse("a#alternativeDownloadLink").unwrap();
    let video_url = doc.select(&selector)
        .next()
        .unwrap()
        .attr("href")
        .unwrap()
        .to_string();

    Command::new(vlc_path())
        .arg(&video_url)
        .arg("--width=576")
        .arg("--height=324")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    State::PostPlay(indirizzi[index].clone(), index, indirizzi)
}