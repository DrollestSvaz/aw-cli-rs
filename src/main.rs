mod play;
mod search_input;
mod search_results;
mod episode_list;
mod post_play;



use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use crossterm::cursor::MoveTo;
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

    let initial_state = if args.contains(&"--news".to_string()) || args.contains(&"-n".to_string()) {
        State::SearchResults(Option::from(None))
    } else if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("usage: aw-cli-rs [-h] [--news]");
        println!("Guarda anime dal terminale!");
        println!();
        println!("Informazioni:");
        println!("  -h, --help       mostra questo messaggio");
        println!();
        println!("Opzioni:");
        println!("  --news           mostra gli ultimi anime usciti su AnimeWorld");
        std::process::exit(0);
    } else if args.iter().skip(1).any(|a| a.starts_with('-')) {
        eprintln!("Argomento non riconosciuto. Usa --help per la lista dei comandi.");
        std::process::exit(1);
    } else {
        State::SearchInput
    };


    let mut state = initial_state;
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();

    loop {
        state = match state {
            State::SearchInput => search_input::search_input().await,
            State::SearchResults(query) => search_results::search_results(query).await,
            State::EpisodeList(anime, href) => episode_list::episode_list(anime, href).await,
            State::Playing(url, index, indirizzi) => play::play(url, index, indirizzi).await,
            State::PostPlay(url, index_ep , indirizzi) => post_play::post_play(url, index_ep, indirizzi).await,
        }
    }
}
fn vlc_path() -> &'static str {
    if cfg!(target_os = "windows") {
        r"C:\Program Files\VideoLAN\VLC\vlc.exe"
    } else {
        "vlc"
    }
}