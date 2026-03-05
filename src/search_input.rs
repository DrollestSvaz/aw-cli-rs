use colored::Colorize;
use crate::State;

pub async fn search_input() -> State {
    let mut title: String = String::new();
    println!("{}", "Cerca un anime per titolo:".bold().on_white().truecolor(37, 37, 37));
    std::io::stdin() // Get the standard input stream
        .read_line(&mut title) // The read_line function reads data until it reaches a '\n' character
        .expect("Unable to read Stdin"); // In case the read operation fails, it panics with the given message

    State::SearchResults(Option::from(title.trim().to_string()))
}